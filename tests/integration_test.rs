use rusqlite::Connection;
use maximous::db;
use maximous::tools;

fn setup() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    db::init_db(&conn).unwrap();
    conn
}

#[test]
fn test_full_multi_agent_workflow() {
    let conn = setup();

    // 1. Register two agents
    tools::agents::register(
        &serde_json::json!({"id": "parser", "name": "Parser Agent", "capabilities": ["parsing"]}),
        &conn,
    );
    tools::agents::register(
        &serde_json::json!({"id": "builder", "name": "Builder Agent", "capabilities": ["building"]}),
        &conn,
    );

    // 2. Create tasks with dependency
    let r1 = tools::tasks::create(&serde_json::json!({"title": "Parse API spec", "priority": 1}), &conn);
    let parse_id = r1.data.unwrap()["id"].as_str().unwrap().to_string();

    let r2 = tools::tasks::create(&serde_json::json!({
        "title": "Build endpoints",
        "dependencies": [parse_id]
    }), &conn);
    let build_id = r2.data.unwrap()["id"].as_str().unwrap().to_string();

    // 3. Parser picks up task
    tools::tasks::update(&serde_json::json!({"id": parse_id, "status": "running", "assigned_to": "parser"}), &conn);

    // 4. Parser stores result in memory
    tools::memory::set(&serde_json::json!({
        "namespace": "task-results",
        "key": parse_id,
        "value": "{\"endpoints\":[\"/users\",\"/items\"]}"
    }), &conn);

    // 5. Parser completes task
    tools::tasks::update(&serde_json::json!({"id": parse_id, "status": "done", "result": "{\"success\":true}"}), &conn);

    // 6. Builder polls changes and sees completion
    let changes = tools::changes::poll(&serde_json::json!({"since_id": 0, "table_name": "tasks"}), &conn);
    let changes_data = changes.data.unwrap();
    let task_changes = changes_data["changes"].as_array().unwrap();
    assert!(task_changes.iter().any(|c| {
        c["row_id"].as_str() == Some(parse_id.as_str()) && c["action"] == "update"
    }));

    // 7. Builder can now set dependent task to ready
    let result = tools::tasks::update(&serde_json::json!({"id": build_id, "status": "ready"}), &conn);
    assert!(result.ok);

    // 8. Builder picks it up and reads upstream result
    tools::tasks::update(&serde_json::json!({"id": build_id, "status": "running", "assigned_to": "builder"}), &conn);
    let mem = tools::memory::get(&serde_json::json!({"namespace": "task-results", "key": parse_id}), &conn);
    assert!(mem.data.unwrap()["value"].as_str().unwrap().contains("/users"));

    // 9. Agents communicate
    tools::messages::send(&serde_json::json!({
        "channel": "team",
        "sender": "builder",
        "content": "{\"question\":\"REST or GraphQL?\"}",
        "priority": 1
    }), &conn);

    let msgs = tools::messages::read(&serde_json::json!({"channel": "team", "unacknowledged_only": true}), &conn);
    assert_eq!(msgs.data.unwrap()["count"], 1);

    // 10. Verify agent list
    let agents = tools::agents::list(&serde_json::json!({}), &conn);
    assert_eq!(agents.data.unwrap()["count"], 2);
}
