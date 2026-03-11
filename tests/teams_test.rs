use rusqlite::Connection;
use maximous::db;

fn setup() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    db::init_db(&conn).unwrap();
    conn
}

#[test]
fn test_agent_definitions_table_exists() {
    let conn = setup();
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='agent_definitions'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_teams_table_exists() {
    let conn = setup();
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='teams'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_team_members_table_exists() {
    let conn = setup();
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='team_members'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_tickets_table_exists() {
    let conn = setup();
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='tickets'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_launches_table_exists() {
    let conn = setup();
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='launches'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_agent_definitions_insert_and_query() {
    let conn = setup();
    conn.execute(
        "INSERT INTO agent_definitions (id, name, capabilities, model, prompt_hint) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params!["agent-1", "CodeAgent", r#"["code","review"]"#, "sonnet", "You are a coding expert."],
    ).unwrap();

    let name: String = conn
        .query_row(
            "SELECT name FROM agent_definitions WHERE id = 'agent-1'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(name, "CodeAgent");
}

#[test]
fn test_teams_insert_and_query() {
    let conn = setup();
    conn.execute(
        "INSERT INTO teams (id, name, description) VALUES (?1, ?2, ?3)",
        rusqlite::params!["team-1", "backend", "Backend engineering team"],
    ).unwrap();

    let name: String = conn
        .query_row(
            "SELECT name FROM teams WHERE id = 'team-1'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(name, "backend");
}

#[test]
fn test_team_members_insert_and_query() {
    let conn = setup();
    conn.execute(
        "INSERT INTO agent_definitions (id, name) VALUES (?1, ?2)",
        rusqlite::params!["agent-2", "ReviewAgent"],
    ).unwrap();
    conn.execute(
        "INSERT INTO teams (id, name) VALUES (?1, ?2)",
        rusqlite::params!["team-2", "frontend"],
    ).unwrap();
    conn.execute(
        "INSERT INTO team_members (team_id, agent_id, role) VALUES (?1, ?2, ?3)",
        rusqlite::params!["team-2", "agent-2", "reviewer"],
    ).unwrap();

    let role: String = conn
        .query_row(
            "SELECT role FROM team_members WHERE team_id = 'team-2' AND agent_id = 'agent-2'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(role, "reviewer");
}

#[test]
fn test_tickets_insert_and_query() {
    let conn = setup();
    conn.execute(
        "INSERT INTO tickets (id, source, external_id, title, status) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params!["ticket-1", "linear", "ENG-123", "Fix auth bug", "todo"],
    ).unwrap();

    let title: String = conn
        .query_row(
            "SELECT title FROM tickets WHERE id = 'ticket-1'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(title, "Fix auth bug");
}

#[test]
fn test_launches_insert_and_query() {
    let conn = setup();
    conn.execute(
        "INSERT INTO tickets (id, source, external_id, title, status) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params!["ticket-2", "linear", "ENG-456", "Add feature X", "in_progress"],
    ).unwrap();
    conn.execute(
        "INSERT INTO teams (id, name) VALUES (?1, ?2)",
        rusqlite::params!["team-3", "platform"],
    ).unwrap();
    conn.execute(
        "INSERT INTO launches (id, ticket_id, team_id, branch) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params!["launch-1", "ticket-2", "team-3", "feature/add-feature-x"],
    ).unwrap();

    let status: String = conn
        .query_row(
            "SELECT status FROM launches WHERE id = 'launch-1'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(status, "pending");
}

#[test]
fn test_agent_definitions_trigger_populates_changes() {
    let conn = setup();
    conn.execute(
        "INSERT INTO agent_definitions (id, name) VALUES (?1, ?2)",
        rusqlite::params!["agent-3", "TestAgent"],
    ).unwrap();

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM changes WHERE table_name = 'agent_definitions'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_teams_trigger_populates_changes() {
    let conn = setup();
    conn.execute(
        "INSERT INTO teams (id, name) VALUES (?1, ?2)",
        rusqlite::params!["team-4", "devops"],
    ).unwrap();

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM changes WHERE table_name = 'teams'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_tickets_unique_constraint() {
    let conn = setup();
    conn.execute(
        "INSERT INTO tickets (id, source, external_id, title, status) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params!["ticket-3", "linear", "ENG-789", "First insert", "todo"],
    ).unwrap();

    let result = conn.execute(
        "INSERT INTO tickets (id, source, external_id, title, status) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params!["ticket-4", "linear", "ENG-789", "Duplicate source+external_id", "todo"],
    );
    assert!(result.is_err(), "Should fail due to UNIQUE(source, external_id)");
}

#[test]
fn test_team_members_cascade_delete() {
    let conn = setup();
    conn.execute(
        "INSERT INTO agent_definitions (id, name) VALUES (?1, ?2)",
        rusqlite::params!["agent-4", "TempAgent"],
    ).unwrap();
    conn.execute(
        "INSERT INTO teams (id, name) VALUES (?1, ?2)",
        rusqlite::params!["team-5", "temp-team"],
    ).unwrap();
    conn.execute(
        "INSERT INTO team_members (team_id, agent_id, role) VALUES (?1, ?2, ?3)",
        rusqlite::params!["team-5", "agent-4", "member"],
    ).unwrap();

    // Deleting the team should cascade to team_members
    conn.execute("DELETE FROM teams WHERE id = 'team-5'", []).unwrap();

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM team_members WHERE team_id = 'team-5'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 0);
}
