use rusqlite::Connection;
use maximous::db;
use maximous::tools;

fn setup() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    db::init_db(&conn).unwrap();
    conn
}

#[test]
fn test_define_creates_agent() {
    let conn = setup();
    let result = tools::definitions::define(
        &serde_json::json!({"id": "agent-1", "name": "Researcher"}),
        &conn,
    );
    assert!(result.ok);
    let data = result.data.unwrap();
    assert_eq!(data["agent"]["id"], "agent-1");
    assert_eq!(data["agent"]["name"], "Researcher");
    assert_eq!(data["agent"]["model"], "sonnet");
    assert_eq!(data["agent"]["prompt_hint"], "");
    assert_eq!(data["agent"]["capabilities"], "[]");
}

#[test]
fn test_define_with_all_fields() {
    let conn = setup();
    let result = tools::definitions::define(
        &serde_json::json!({
            "id": "agent-2",
            "name": "Coder",
            "capabilities": ["rust", "python"],
            "model": "opus",
            "prompt_hint": "You are an expert coder."
        }),
        &conn,
    );
    assert!(result.ok);
    let data = result.data.unwrap();
    assert_eq!(data["agent"]["model"], "opus");
    assert_eq!(data["agent"]["prompt_hint"], "You are an expert coder.");
}

#[test]
fn test_define_upsert() {
    let conn = setup();
    // Create initial definition
    tools::definitions::define(
        &serde_json::json!({"id": "agent-1", "name": "V1", "model": "haiku"}),
        &conn,
    );
    // Upsert with new values
    let result = tools::definitions::define(
        &serde_json::json!({"id": "agent-1", "name": "V2", "model": "opus"}),
        &conn,
    );
    assert!(result.ok);
    let data = result.data.unwrap();
    assert_eq!(data["agent"]["name"], "V2");
    assert_eq!(data["agent"]["model"], "opus");

    // Catalog should show only one entry
    let catalog = tools::definitions::catalog(&serde_json::json!({}), &conn);
    let catalog_data = catalog.data.unwrap();
    assert_eq!(catalog_data["count"], 1);
}

#[test]
fn test_catalog_returns_all_agents() {
    let conn = setup();
    tools::definitions::define(&serde_json::json!({"id": "b-agent", "name": "Bravo"}), &conn);
    tools::definitions::define(&serde_json::json!({"id": "a-agent", "name": "Alpha"}), &conn);
    tools::definitions::define(&serde_json::json!({"id": "c-agent", "name": "Charlie"}), &conn);

    let result = tools::definitions::catalog(&serde_json::json!({}), &conn);
    assert!(result.ok);
    let data = result.data.unwrap();
    assert_eq!(data["count"], 3);
    let agents = data["agents"].as_array().unwrap();
    // Should be ordered by name
    assert_eq!(agents[0]["name"], "Alpha");
    assert_eq!(agents[1]["name"], "Bravo");
    assert_eq!(agents[2]["name"], "Charlie");
}

#[test]
fn test_catalog_pagination() {
    let conn = setup();
    for i in 0..5 {
        tools::definitions::define(
            &serde_json::json!({"id": format!("agent-{}", i), "name": format!("Agent {}", i)}),
            &conn,
        );
    }

    let result = tools::definitions::catalog(&serde_json::json!({"limit": 2, "offset": 0}), &conn);
    assert!(result.ok);
    let data = result.data.unwrap();
    assert_eq!(data["count"], 2);

    let result2 = tools::definitions::catalog(&serde_json::json!({"limit": 2, "offset": 2}), &conn);
    assert!(result2.ok);
    let data2 = result2.data.unwrap();
    assert_eq!(data2["count"], 2);
}

#[test]
fn test_remove_existing_agent() {
    let conn = setup();
    tools::definitions::define(&serde_json::json!({"id": "agent-1", "name": "ToDelete"}), &conn);

    let result = tools::definitions::remove(&serde_json::json!({"id": "agent-1"}), &conn);
    assert!(result.ok);
    let data = result.data.unwrap();
    assert_eq!(data["removed"], true);

    // Verify it's gone
    let catalog = tools::definitions::catalog(&serde_json::json!({}), &conn);
    let catalog_data = catalog.data.unwrap();
    assert_eq!(catalog_data["count"], 0);
}

#[test]
fn test_remove_nonexistent_agent() {
    let conn = setup();
    let result = tools::definitions::remove(&serde_json::json!({"id": "no-such-agent"}), &conn);
    assert!(result.ok);
    let data = result.data.unwrap();
    assert_eq!(data["removed"], false);
}

#[test]
fn test_define_missing_id() {
    let conn = setup();
    let result = tools::definitions::define(&serde_json::json!({"name": "NoId"}), &conn);
    assert!(!result.ok);
    assert!(result.error.unwrap().contains("id"));
}

#[test]
fn test_define_missing_name() {
    let conn = setup();
    let result = tools::definitions::define(&serde_json::json!({"id": "agent-1"}), &conn);
    assert!(!result.ok);
    assert!(result.error.unwrap().contains("name"));
}

#[test]
fn test_remove_missing_id() {
    let conn = setup();
    let result = tools::definitions::remove(&serde_json::json!({}), &conn);
    assert!(!result.ok);
    assert!(result.error.unwrap().contains("id"));
}

#[test]
fn test_dispatch_agent_define() {
    let conn = setup();
    let result = tools::dispatch_tool(
        "agent_define",
        &serde_json::json!({"id": "d1", "name": "Dispatcher"}),
        &conn,
    );
    assert!(result.ok);
}

#[test]
fn test_dispatch_agent_catalog() {
    let conn = setup();
    let result = tools::dispatch_tool("agent_catalog", &serde_json::json!({}), &conn);
    assert!(result.ok);
}

#[test]
fn test_dispatch_agent_remove() {
    let conn = setup();
    tools::definitions::define(&serde_json::json!({"id": "r1", "name": "ToRemove"}), &conn);
    let result = tools::dispatch_tool("agent_remove", &serde_json::json!({"id": "r1"}), &conn);
    assert!(result.ok);
    let data = result.data.unwrap();
    assert_eq!(data["removed"], true);
}
