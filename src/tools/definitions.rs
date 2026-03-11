use rusqlite::Connection;
use serde_json::Value;
use super::ToolResult;

fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

pub fn define(args: &Value, conn: &Connection) -> ToolResult {
    let id = match args["id"].as_str() {
        Some(s) => s,
        None => return ToolResult::fail("missing required field: id"),
    };
    let name = match args["name"].as_str() {
        Some(s) => s,
        None => return ToolResult::fail("missing required field: name"),
    };
    let capabilities = args.get("capabilities")
        .filter(|c| c.is_array())
        .map(|c| serde_json::to_string(c).unwrap())
        .unwrap_or_else(|| "[]".to_string());
    let model = args["model"].as_str().unwrap_or("sonnet").to_string();
    let prompt_hint = args["prompt_hint"].as_str().unwrap_or("").to_string();
    let ts = now();

    match conn.execute(
        "INSERT INTO agent_definitions (id, name, capabilities, model, prompt_hint, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)
         ON CONFLICT(id) DO UPDATE SET name=?2, capabilities=?3, model=?4, prompt_hint=?5, updated_at=?6",
        rusqlite::params![id, name, capabilities, model, prompt_hint, ts],
    ) {
        Ok(_) => ToolResult::success(serde_json::json!({
            "agent": {
                "id": id,
                "name": name,
                "capabilities": capabilities,
                "model": model,
                "prompt_hint": prompt_hint,
            }
        })),
        Err(e) => ToolResult::fail(&format!("db error: {}", e)),
    }
}

pub fn catalog(args: &Value, conn: &Connection) -> ToolResult {
    let limit = args["limit"].as_i64().unwrap_or(100);
    let offset = args["offset"].as_i64().unwrap_or(0);

    let mut stmt = match conn.prepare(
        "SELECT id, name, capabilities, model, prompt_hint FROM agent_definitions ORDER BY name LIMIT ?1 OFFSET ?2"
    ) {
        Ok(s) => s,
        Err(e) => return ToolResult::fail(&format!("db error: {}", e)),
    };

    let agents: Vec<Value> = stmt
        .query_map(rusqlite::params![limit, offset], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "name": row.get::<_, String>(1)?,
                "capabilities": row.get::<_, String>(2)?,
                "model": row.get::<_, String>(3)?,
                "prompt_hint": row.get::<_, String>(4)?,
            }))
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    let count = agents.len();
    ToolResult::success(serde_json::json!({"agents": agents, "count": count}))
}

pub fn remove(args: &Value, conn: &Connection) -> ToolResult {
    let id = match args["id"].as_str() {
        Some(s) => s,
        None => return ToolResult::fail("missing required field: id"),
    };

    match conn.execute(
        "DELETE FROM agent_definitions WHERE id = ?1",
        rusqlite::params![id],
    ) {
        Ok(deleted) => ToolResult::success(serde_json::json!({"removed": deleted > 0})),
        Err(e) => ToolResult::fail(&format!("db error: {}", e)),
    }
}
