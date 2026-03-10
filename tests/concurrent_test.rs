use std::sync::{Arc, Barrier};
use std::thread;

use maximous::db;
use maximous::tools;

#[test]
fn test_concurrent_wal_writes() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("concurrent.db");
    let db_path_str = db_path.to_str().unwrap().to_string();

    let conn = db::open_db(&db_path_str).unwrap();
    drop(conn);

    let num_threads = 4;
    let ops_per_thread = 500;
    let barrier = Arc::new(Barrier::new(num_threads));

    let handles: Vec<_> = (0..num_threads)
        .map(|t| {
            let barrier = Arc::clone(&barrier);
            let path = db_path_str.clone();
            thread::spawn(move || {
                let conn = db::open_db(&path).unwrap();
                barrier.wait();
                let mut successes = 0;
                for i in 0..ops_per_thread {
                    let result = tools::memory::set(
                        &serde_json::json!({
                            "namespace": format!("thread-{}", t),
                            "key": format!("key-{}", i),
                            "value": format!("{{\"thread\":{},\"op\":{}}}", t, i)
                        }),
                        &conn,
                    );
                    if result.ok {
                        successes += 1;
                    }
                }
                successes
            })
        })
        .collect();

    let total_successes: usize = handles.into_iter().map(|h| h.join().unwrap()).sum();
    let expected = num_threads * ops_per_thread;
    assert_eq!(
        total_successes, expected,
        "Expected {} successful writes, got {} ({} failed)",
        expected, total_successes, expected - total_successes
    );

    let conn = db::open_db(&db_path_str).unwrap();
    for t in 0..num_threads {
        let result = tools::memory::get(
            &serde_json::json!({"namespace": format!("thread-{}", t)}),
            &conn,
        );
        let keys = result.data.unwrap()["keys"].as_array().unwrap().len();
        assert_eq!(keys, ops_per_thread, "Thread {} should have {} keys", t, ops_per_thread);
    }
}

#[test]
fn test_concurrent_read_write_mix() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("mixed.db");
    let db_path_str = db_path.to_str().unwrap().to_string();

    let conn = db::open_db(&db_path_str).unwrap();
    for i in 0..100 {
        tools::memory::set(
            &serde_json::json!({"namespace": "shared", "key": format!("pre-{}", i), "value": "initial"}),
            &conn,
        );
    }
    drop(conn);

    let barrier = Arc::new(Barrier::new(4));
    let mut handles = vec![];

    for t in 0..2 {
        let barrier = Arc::clone(&barrier);
        let path = db_path_str.clone();
        handles.push(thread::spawn(move || {
            let conn = db::open_db(&path).unwrap();
            barrier.wait();
            for i in 0..200 {
                tools::memory::set(
                    &serde_json::json!({"namespace": "shared", "key": format!("w{}-{}", t, i), "value": "written"}),
                    &conn,
                );
            }
            true
        }));
    }

    for _ in 0..2 {
        let barrier = Arc::clone(&barrier);
        let path = db_path_str.clone();
        handles.push(thread::spawn(move || {
            let conn = db::open_db(&path).unwrap();
            barrier.wait();
            for _ in 0..200 {
                tools::memory::get(
                    &serde_json::json!({"namespace": "shared"}),
                    &conn,
                );
                tools::changes::poll(
                    &serde_json::json!({"since_id": 0, "limit": 10}),
                    &conn,
                );
            }
            true
        }));
    }

    for h in handles {
        assert!(h.join().unwrap(), "Thread should complete without panic");
    }
}
