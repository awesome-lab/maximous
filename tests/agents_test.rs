use rusqlite::Connection;
use maximous::db;
use maximous::tools;

fn setup() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    db::init_db(&conn).unwrap();
    conn
}

#[test]
fn test_agent_register_and_list() {
    let conn = setup();
    let result = tools::agents::register(
        &serde_json::json!({"id": "agent-1", "name": "Parser", "capabilities": ["parsing", "analysis"]}),
        &conn,
    );
    assert!(result.ok);
    let result = tools::agents::list(&serde_json::json!({}), &conn);
    assert!(result.ok);
    let agents = result.data.unwrap();
    assert_eq!(agents["agents"].as_array().unwrap().len(), 1);
    assert_eq!(agents["agents"][0]["name"], "Parser");
}

#[test]
fn test_agent_heartbeat() {
    let conn = setup();
    tools::agents::register(&serde_json::json!({"id": "a1", "name": "Agent"}), &conn);
    let result = tools::agents::heartbeat(&serde_json::json!({"id": "a1", "status": "active"}), &conn);
    assert!(result.ok);
}

#[test]
fn test_agent_register_upsert() {
    let conn = setup();
    tools::agents::register(&serde_json::json!({"id": "a1", "name": "V1"}), &conn);
    tools::agents::register(&serde_json::json!({"id": "a1", "name": "V2", "capabilities": ["new"]}), &conn);
    let result = tools::agents::list(&serde_json::json!({"include_stale": true}), &conn);
    let agents = result.data.unwrap();
    let agents = agents["agents"].as_array().unwrap();
    assert_eq!(agents.len(), 1);
    assert_eq!(agents[0]["name"], "V2");
}

#[test]
fn test_agent_stale_filtering() {
    let conn = setup();
    conn.execute(
        "INSERT INTO agents (id, name, status, last_heartbeat) VALUES ('stale', 'Old', 'idle', 0)",
        [],
    ).unwrap();
    tools::agents::register(&serde_json::json!({"id": "fresh", "name": "New"}), &conn);
    let result = tools::agents::list(&serde_json::json!({}), &conn);
    let agents = result.data.unwrap();
    let agents = agents["agents"].as_array().unwrap();
    assert_eq!(agents.len(), 1);
    assert_eq!(agents[0]["id"], "fresh");
    let result = tools::agents::list(&serde_json::json!({"include_stale": true}), &conn);
    let agents = result.data.unwrap();
    assert_eq!(agents["agents"].as_array().unwrap().len(), 2);
}
