use rusqlite::Connection;
use serde_json::Value;
use super::ToolResult;

pub fn poll(args: &Value, conn: &Connection) -> ToolResult {
    let since_id = args["since_id"].as_i64().unwrap_or(0);
    let table_name = args["table_name"].as_str();
    let limit = args["limit"].as_i64().unwrap_or(100);

    let (sql, params): (String, Vec<Box<dyn rusqlite::types::ToSql>>) = match table_name {
        Some(tn) => (
            "SELECT id, table_name, row_id, action, summary, created_at FROM changes WHERE id > ? AND table_name = ? ORDER BY id ASC LIMIT ?".to_string(),
            vec![Box::new(since_id), Box::new(tn.to_string()), Box::new(limit)],
        ),
        None => (
            "SELECT id, table_name, row_id, action, summary, created_at FROM changes WHERE id > ? ORDER BY id ASC LIMIT ?".to_string(),
            vec![Box::new(since_id), Box::new(limit)],
        ),
    };

    let params_ref: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql).unwrap();
    let changes: Vec<Value> = stmt
        .query_map(params_ref.as_slice(), |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "table_name": row.get::<_, String>(1)?,
                "row_id": row.get::<_, String>(2)?,
                "action": row.get::<_, String>(3)?,
                "summary": row.get::<_, Option<String>>(4)?,
                "created_at": row.get::<_, i64>(5)?,
            }))
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    ToolResult::success(serde_json::json!({"changes": changes, "count": changes.len()}))
}
