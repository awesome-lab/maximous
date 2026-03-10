use rusqlite::Connection;
use serde_json::Value;
use super::ToolResult;

pub fn set(args: &Value, conn: &Connection) -> ToolResult {
    let namespace = match args["namespace"].as_str() {
        Some(s) => s,
        None => return ToolResult::fail("missing required field: namespace"),
    };
    let key = match args["key"].as_str() {
        Some(s) => s,
        None => return ToolResult::fail("missing required field: key"),
    };
    let value = match args["value"].as_str() {
        Some(s) => s,
        None => return ToolResult::fail("missing required field: value"),
    };
    let ttl = args["ttl_seconds"].as_i64();

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    match conn.execute(
        "INSERT INTO memory (namespace, key, value, ttl_seconds, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?5)
         ON CONFLICT(namespace, key) DO UPDATE SET value=?3, ttl_seconds=?4, updated_at=?5",
        rusqlite::params![namespace, key, value, ttl, now],
    ) {
        Ok(_) => ToolResult::success(serde_json::json!({"stored": true})),
        Err(e) => ToolResult::fail(&format!("db error: {}", e)),
    }
}

pub fn get(args: &Value, conn: &Connection) -> ToolResult {
    let namespace = match args["namespace"].as_str() {
        Some(s) => s,
        None => return ToolResult::fail("missing required field: namespace"),
    };

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let _ = conn.execute(
        "DELETE FROM memory WHERE namespace = ?1 AND ttl_seconds IS NOT NULL AND (created_at + ttl_seconds) < ?2",
        rusqlite::params![namespace, now],
    );

    match args["key"].as_str() {
        Some(key) => {
            let result: Result<(String, Option<i64>, i64), _> = conn.query_row(
                "SELECT value, ttl_seconds, updated_at FROM memory WHERE namespace = ?1 AND key = ?2",
                rusqlite::params![namespace, key],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            );
            match result {
                Ok((value, ttl, updated_at)) => ToolResult::success(serde_json::json!({
                    "namespace": namespace,
                    "key": key,
                    "value": value,
                    "ttl_seconds": ttl,
                    "updated_at": updated_at,
                })),
                Err(_) => ToolResult::success(serde_json::json!({
                    "namespace": namespace,
                    "key": key,
                    "value": null,
                })),
            }
        }
        None => {
            let mut stmt = conn.prepare(
                "SELECT key, updated_at FROM memory WHERE namespace = ?1 ORDER BY key"
            ).unwrap();
            let keys: Vec<Value> = stmt
                .query_map(rusqlite::params![namespace], |row| {
                    Ok(serde_json::json!({
                        "key": row.get::<_, String>(0)?,
                        "updated_at": row.get::<_, i64>(1)?,
                    }))
                })
                .unwrap()
                .filter_map(|r| r.ok())
                .collect();
            ToolResult::success(serde_json::json!({"namespace": namespace, "keys": keys}))
        }
    }
}

pub fn search(args: &Value, conn: &Connection) -> ToolResult {
    let query = match args["query"].as_str() {
        Some(s) => s,
        None => return ToolResult::fail("missing required field: query"),
    };
    let namespace = args["namespace"].as_str();
    let limit = args["limit"].as_i64().unwrap_or(50);
    let offset = args["offset"].as_i64().unwrap_or(0);

    // Try FTS5 first, fall back to LIKE if the FTS table doesn't exist
    match search_fts(query, namespace, limit, offset, conn) {
        Ok(result) => result,
        Err(_) => search_like(query, namespace, limit, offset, conn),
    }
}

fn search_fts(
    query: &str,
    namespace: Option<&str>,
    limit: i64,
    offset: i64,
    conn: &Connection,
) -> Result<ToolResult, rusqlite::Error> {
    let (sql, params): (String, Vec<Box<dyn rusqlite::types::ToSql>>) = match namespace {
        Some(ns) => (
            "SELECT m.namespace, m.key, m.value, f.rank \
             FROM memory_fts f \
             JOIN memory m ON m.rowid = f.rowid \
             WHERE memory_fts MATCH ? AND m.namespace = ? \
             ORDER BY f.rank \
             LIMIT ? OFFSET ?"
                .to_string(),
            vec![
                Box::new(query.to_string()),
                Box::new(ns.to_string()),
                Box::new(limit),
                Box::new(offset),
            ],
        ),
        None => (
            "SELECT m.namespace, m.key, m.value, f.rank \
             FROM memory_fts f \
             JOIN memory m ON m.rowid = f.rowid \
             WHERE memory_fts MATCH ? \
             ORDER BY f.rank \
             LIMIT ? OFFSET ?"
                .to_string(),
            vec![
                Box::new(query.to_string()),
                Box::new(limit),
                Box::new(offset),
            ],
        ),
    };

    let mut stmt = conn.prepare(&sql)?;
    let params_ref: Vec<&dyn rusqlite::types::ToSql> =
        params.iter().map(|p| p.as_ref()).collect();
    let matches: Vec<Value> = stmt
        .query_map(params_ref.as_slice(), |row| {
            Ok(serde_json::json!({
                "namespace": row.get::<_, String>(0)?,
                "key": row.get::<_, String>(1)?,
                "value": row.get::<_, String>(2)?,
                "rank": row.get::<_, f64>(3)?,
            }))
        })?
        .filter_map(|r| r.ok())
        .collect();

    let count = matches.len() as i64;
    Ok(ToolResult::success(serde_json::json!({
        "matches": matches,
        "count": count,
        "offset": offset,
        "limit": limit,
    })))
}

fn search_like(
    query: &str,
    namespace: Option<&str>,
    limit: i64,
    offset: i64,
    conn: &Connection,
) -> ToolResult {
    let (sql, params): (String, Vec<Box<dyn rusqlite::types::ToSql>>) = match namespace {
        Some(ns) => (
            "SELECT namespace, key, value FROM memory WHERE namespace = ? AND value LIKE ? LIMIT ? OFFSET ?".to_string(),
            vec![
                Box::new(ns.to_string()),
                Box::new(format!("%{}%", query)),
                Box::new(limit),
                Box::new(offset),
            ],
        ),
        None => (
            "SELECT namespace, key, value FROM memory WHERE value LIKE ? LIMIT ? OFFSET ?".to_string(),
            vec![
                Box::new(format!("%{}%", query)),
                Box::new(limit),
                Box::new(offset),
            ],
        ),
    };

    let mut stmt = conn.prepare(&sql).unwrap();
    let params_ref: Vec<&dyn rusqlite::types::ToSql> =
        params.iter().map(|p| p.as_ref()).collect();
    let matches: Vec<Value> = stmt
        .query_map(params_ref.as_slice(), |row| {
            Ok(serde_json::json!({
                "namespace": row.get::<_, String>(0)?,
                "key": row.get::<_, String>(1)?,
                "value": row.get::<_, String>(2)?,
            }))
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    let count = matches.len() as i64;
    ToolResult::success(serde_json::json!({
        "matches": matches,
        "count": count,
        "offset": offset,
        "limit": limit,
    }))
}

pub fn delete(args: &Value, conn: &Connection) -> ToolResult {
    let namespace = match args["namespace"].as_str() {
        Some(s) => s,
        None => return ToolResult::fail("missing required field: namespace"),
    };

    match args["key"].as_str() {
        Some(key) => {
            let deleted = conn.execute(
                "DELETE FROM memory WHERE namespace = ?1 AND key = ?2",
                rusqlite::params![namespace, key],
            ).unwrap_or(0);
            ToolResult::success(serde_json::json!({"deleted": deleted}))
        }
        None => {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            let deleted = conn.execute(
                "DELETE FROM memory WHERE namespace = ?1 AND ttl_seconds IS NOT NULL AND (created_at + ttl_seconds) < ?2",
                rusqlite::params![namespace, now],
            ).unwrap_or(0);
            ToolResult::success(serde_json::json!({"expired": deleted}))
        }
    }
}
