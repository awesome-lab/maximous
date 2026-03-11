use rusqlite::Connection;
use maximous::db;
use maximous::tools;

fn setup() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    db::init_db(&conn).unwrap();
    conn
}

#[test]
fn test_poll_changes_empty() {
    let conn = setup();
    let result = tools::changes::poll(&serde_json::json!({}), &conn);
    assert!(result.ok);
    let data = result.data.unwrap();
    assert_eq!(data["changes"].as_array().unwrap().len(), 0);
}

#[test]
fn test_poll_changes_after_operations() {
    let conn = setup();
    tools::memory::set(&serde_json::json!({"namespace": "ns", "key": "k", "value": "v"}), &conn);
    tools::tasks::create(&serde_json::json!({"title": "test task"}), &conn);
    let result = tools::changes::poll(&serde_json::json!({"since_id": 0}), &conn);
    let data = result.data.unwrap();
    let changes = data["changes"].as_array().unwrap();
    assert!(changes.len() >= 2);
}

#[test]
fn test_poll_changes_since_id() {
    let conn = setup();
    tools::memory::set(&serde_json::json!({"namespace": "ns", "key": "k1", "value": "v1"}), &conn);
    let result = tools::changes::poll(&serde_json::json!({"since_id": 0}), &conn);
    let data = result.data.unwrap();
    let changes = data["changes"].as_array().unwrap();
    let last_id = changes.last().unwrap()["id"].as_i64().unwrap();
    tools::memory::set(&serde_json::json!({"namespace": "ns", "key": "k2", "value": "v2"}), &conn);
    let result = tools::changes::poll(&serde_json::json!({"since_id": last_id}), &conn);
    let data = result.data.unwrap();
    let changes = data["changes"].as_array().unwrap();
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0]["table_name"], "memory");
}

#[test]
fn test_poll_changes_filter_by_table() {
    let conn = setup();
    tools::memory::set(&serde_json::json!({"namespace": "ns", "key": "k", "value": "v"}), &conn);
    tools::tasks::create(&serde_json::json!({"title": "test task"}), &conn);
    let result = tools::changes::poll(&serde_json::json!({"table_name": "tasks"}), &conn);
    let data = result.data.unwrap();
    let changes = data["changes"].as_array().unwrap();
    assert!(changes.iter().all(|c| c["table_name"] == "tasks"));
}

#[test]
fn test_poll_changes_limit() {
    let conn = setup();
    for i in 0..10 {
        tools::memory::set(&serde_json::json!({"namespace": "ns", "key": format!("k{}", i), "value": "v"}), &conn);
    }
    let result = tools::changes::poll(&serde_json::json!({"limit": 3}), &conn);
    let data = result.data.unwrap();
    assert_eq!(data["changes"].as_array().unwrap().len(), 3);
}
