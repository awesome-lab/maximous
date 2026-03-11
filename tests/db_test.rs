use rusqlite::Connection;

use maximous::db;

#[test]
fn test_init_db_creates_all_tables() {
    let conn = Connection::open_in_memory().unwrap();
    db::init_db(&conn).unwrap();

    let tables: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    assert!(tables.contains(&"memory".to_string()));
    assert!(tables.contains(&"tasks".to_string()));
    assert!(tables.contains(&"agents".to_string()));
    assert!(tables.contains(&"changes".to_string()));
    assert!(tables.contains(&"config".to_string()));
    assert!(tables.contains(&"agent_definitions".to_string()));
    assert!(tables.contains(&"teams".to_string()));
    assert!(tables.contains(&"team_members".to_string()));
    assert!(tables.contains(&"tickets".to_string()));
    assert!(tables.contains(&"launches".to_string()));
}

#[test]
fn test_wal_mode_enabled() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let conn = Connection::open(&db_path).unwrap();
    db::init_db(&conn).unwrap();

    let mode: String = conn
        .pragma_query_value(None, "journal_mode", |row| row.get(0))
        .unwrap();
    assert_eq!(mode, "wal");
}

#[test]
fn test_trigger_populates_changes_on_memory_insert() {
    let conn = Connection::open_in_memory().unwrap();
    db::init_db(&conn).unwrap();

    conn.execute(
        "INSERT INTO memory (namespace, key, value, created_at, updated_at) VALUES (?1, ?2, ?3, strftime('%s','now'), strftime('%s','now'))",
        rusqlite::params!["test-ns", "test-key", r#"{"hello":"world"}"#],
    ).unwrap();

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM changes WHERE table_name = 'memory'", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_migration_adds_new_tables() {
    let conn = Connection::open_in_memory().unwrap();
    // Simulate old schema: create only the original tables (no agent_definitions, teams, tickets, launches)
    conn.execute_batch("
        CREATE TABLE memory (namespace TEXT NOT NULL, key TEXT NOT NULL, value TEXT NOT NULL, ttl_seconds INTEGER, created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL, PRIMARY KEY (namespace, key));
        CREATE TABLE messages (id INTEGER PRIMARY KEY AUTOINCREMENT, channel TEXT NOT NULL, sender TEXT NOT NULL, priority INTEGER NOT NULL DEFAULT 2, content TEXT NOT NULL, acknowledged INTEGER NOT NULL DEFAULT 0, created_at INTEGER NOT NULL);
        CREATE TABLE tasks (id TEXT PRIMARY KEY, title TEXT NOT NULL, status TEXT NOT NULL DEFAULT 'pending', priority INTEGER NOT NULL DEFAULT 2, assigned_to TEXT, dependencies TEXT, result TEXT, created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL);
        CREATE TABLE agents (id TEXT PRIMARY KEY, name TEXT NOT NULL, status TEXT NOT NULL DEFAULT 'idle', capabilities TEXT, metadata TEXT, last_heartbeat INTEGER NOT NULL);
        CREATE TABLE changes (id INTEGER PRIMARY KEY AUTOINCREMENT, table_name TEXT NOT NULL, row_id TEXT NOT NULL, action TEXT NOT NULL, summary TEXT, created_at INTEGER NOT NULL);
        CREATE TABLE config (key TEXT PRIMARY KEY, value TEXT NOT NULL);
        CREATE TABLE sessions (id TEXT PRIMARY KEY, agent_id TEXT, status TEXT NOT NULL DEFAULT 'active', metadata TEXT, summary TEXT, started_at INTEGER NOT NULL, ended_at INTEGER);
    ").unwrap();

    // Run init_db which should add new tables via migration
    db::init_db(&conn).unwrap();

    // Verify new tables exist and are empty (migration ran successfully)
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM agent_definitions", [], |r| r.get(0)).unwrap();
    assert_eq!(count, 0);
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM teams", [], |r| r.get(0)).unwrap();
    assert_eq!(count, 0);
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM tickets", [], |r| r.get(0)).unwrap();
    assert_eq!(count, 0);
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM launches", [], |r| r.get(0)).unwrap();
    assert_eq!(count, 0);
}
