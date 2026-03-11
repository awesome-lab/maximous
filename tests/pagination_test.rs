use rusqlite::Connection;
use maximous::db;
use maximous::tools;

fn setup() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    db::init_db(&conn).unwrap();
    conn
}

#[test]
fn test_task_list_pagination() {
    let conn = setup();
    for i in 0..10 {
        tools::tasks::create(
            &serde_json::json!({"title": format!("Task {}", i)}),
            &conn,
        );
    }
    let result = tools::tasks::list(
        &serde_json::json!({"limit": 3, "offset": 0}),
        &conn,
    );
    assert!(result.ok);
    let data = result.data.unwrap();
    assert_eq!(data["tasks"].as_array().unwrap().len(), 3);
    assert_eq!(data["limit"], 3);
    assert_eq!(data["offset"], 0);

    let result = tools::tasks::list(
        &serde_json::json!({"limit": 3, "offset": 3}),
        &conn,
    );
    assert!(result.ok);
    let data = result.data.unwrap();
    assert_eq!(data["tasks"].as_array().unwrap().len(), 3);
    assert_eq!(data["offset"], 3);
}

#[test]
fn test_agent_list_pagination() {
    let conn = setup();
    for i in 0..5 {
        tools::agents::register(
            &serde_json::json!({"id": format!("agent-{}", i), "name": format!("Agent {}", i)}),
            &conn,
        );
    }
    let result = tools::agents::list(
        &serde_json::json!({"include_stale": true, "limit": 2, "offset": 0}),
        &conn,
    );
    assert!(result.ok);
    let data = result.data.unwrap();
    assert_eq!(data["agents"].as_array().unwrap().len(), 2);
}

#[test]
fn test_ticket_list_pagination() {
    let conn = setup();
    for i in 0..5 {
        tools::tickets::cache(
            &serde_json::json!({"id": format!("t{}", i), "source": "linear", "external_id": format!("LIN-{}", i), "title": format!("Ticket {}", i), "status": "todo"}),
            &conn,
        );
    }
    let result = tools::tickets::list(
        &serde_json::json!({"limit": 2, "offset": 0}),
        &conn,
    );
    assert!(result.ok);
    let data = result.data.unwrap();
    assert_eq!(data["tickets"].as_array().unwrap().len(), 2);
}
