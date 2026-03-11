use rusqlite::Connection;
use maximous::db;
use maximous::tools;

fn setup() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    db::init_db(&conn).unwrap();
    conn
}

#[test]
fn test_full_workflow_define_agents_create_team_cache_ticket_launch() {
    let conn = setup();

    // 1. Define agents
    let r = tools::definitions::define(&serde_json::json!({
        "id": "dev", "name": "Developer", "capabilities": ["rust"], "model": "sonnet"
    }), &conn);
    assert!(r.ok);

    let r = tools::definitions::define(&serde_json::json!({
        "id": "reviewer", "name": "Reviewer", "model": "opus"
    }), &conn);
    assert!(r.ok);

    // 2. Create team
    let team = tools::teams::create(&serde_json::json!({
        "name": "rust-team",
        "members": [
            {"agent_id": "dev", "role": "implementer"},
            {"agent_id": "reviewer", "role": "reviewer"}
        ]
    }), &conn);
    assert!(team.ok);
    let team_id = team.data.unwrap()["team"]["id"].as_str().unwrap().to_string();

    // 3. Cache ticket
    let r = tools::tickets::cache(&serde_json::json!({
        "id": "ticket-1", "source": "linear", "external_id": "LIN-42",
        "title": "Add auth middleware", "status": "todo",
        "url": "https://linear.app/team/LIN-42"
    }), &conn);
    assert!(r.ok);

    // 4. Create launch
    let launch = tools::launches::create(&serde_json::json!({
        "ticket_id": "ticket-1", "team_id": team_id, "branch": "feat/auth-middleware"
    }), &conn);
    assert!(launch.ok);
    let launch_id = launch.data.unwrap()["launch"]["id"].as_str().unwrap().to_string();

    // 5. Update to running
    let r = tools::launches::update(&serde_json::json!({
        "id": launch_id, "status": "running", "worktree_path": "/tmp/wt/auth"
    }), &conn);
    assert!(r.ok);

    // 6. Complete with PR
    let r = tools::launches::update(&serde_json::json!({
        "id": launch_id, "status": "pr_created",
        "pr_url": "https://github.com/org/repo/pull/42"
    }), &conn);
    assert!(r.ok);

    // 7. Verify launch list
    let result = tools::launches::list(&serde_json::json!({}), &conn);
    assert!(result.ok);
    let data = result.data.unwrap();
    let launches = data["launches"].as_array().unwrap();
    assert_eq!(launches.len(), 1);
    assert_eq!(launches[0]["status"], "pr_created");
    assert_eq!(launches[0]["pr_url"], "https://github.com/org/repo/pull/42");
    assert_eq!(launches[0]["ticket_title"], "Add auth middleware");
    assert_eq!(launches[0]["team_name"], "rust-team");

    // 8. Verify changes captured events
    let result = tools::changes::poll(&serde_json::json!({"since_id": 0}), &conn);
    assert!(result.ok);
    let changes = result.data.unwrap()["changes"].as_array().unwrap().len();
    assert!(changes >= 6, "Expected at least 6 changes, got {}", changes);
}
