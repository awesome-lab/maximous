use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use rusqlite::Connection;

use maximous::db;
use maximous::tools;

fn setup() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    db::init_db(&conn).unwrap();
    conn
}

fn bench_memory_roundtrip(c: &mut Criterion) {
    let conn = setup();
    c.bench_function("memory_set+get", |b| {
        let mut i = 0u64;
        b.iter(|| {
            let key = format!("key-{}", i);
            tools::memory::set(
                &serde_json::json!({"namespace": "bench", "key": key, "value": "{\"data\":true}"}),
                &conn,
            );
            tools::memory::get(
                &serde_json::json!({"namespace": "bench", "key": key}),
                &conn,
            );
            i += 1;
        });
    });
}

fn bench_memory_write_throughput(c: &mut Criterion) {
    let conn = setup();
    c.bench_function("memory_set_throughput", |b| {
        let mut i = 0u64;
        b.iter(|| {
            let key = format!("tp-{}", i);
            tools::memory::set(
                &serde_json::json!({"namespace": "throughput", "key": key, "value": "{\"n\":1}"}),
                &conn,
            );
            i += 1;
        });
    });
}

fn bench_message_roundtrip(c: &mut Criterion) {
    let conn = setup();
    c.bench_function("message_send+read", |b| {
        b.iter(|| {
            tools::messages::send(
                &serde_json::json!({"channel": "bench", "sender": "agent", "content": "{\"ping\":true}"}),
                &conn,
            );
            tools::messages::read(
                &serde_json::json!({"channel": "bench", "limit": 1}),
                &conn,
            );
        });
    });
}

fn bench_poll_changes_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("poll_changes_scaling");
    for size in [100, 1_000, 10_000, 50_000].iter() {
        let conn = setup();
        for i in 0..*size {
            conn.execute(
                "INSERT INTO changes (table_name, row_id, action, summary, created_at) VALUES ('bench', ?1, 'insert', '{}', 0)",
                rusqlite::params![format!("row-{}", i)],
            ).unwrap();
        }
        let since_id = (*size as i64) - 100;
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            size,
            |b, _| {
                b.iter(|| {
                    tools::changes::poll(
                        &serde_json::json!({"since_id": since_id, "limit": 100}),
                        &conn,
                    );
                });
            },
        );
    }
    group.finish();
}

fn bench_task_with_deps(c: &mut Criterion) {
    let conn = setup();
    let dep = tools::tasks::create(&serde_json::json!({"title": "dep"}), &conn);
    let dep_id = dep.data.unwrap()["id"].as_str().unwrap().to_string();
    tools::tasks::update(&serde_json::json!({"id": dep_id, "status": "done"}), &conn);

    c.bench_function("task_create_with_dep_check", |b| {
        let mut i = 0u64;
        b.iter(|| {
            let r = tools::tasks::create(
                &serde_json::json!({"title": format!("task-{}", i), "dependencies": [dep_id]}),
                &conn,
            );
            let id = r.data.unwrap()["id"].as_str().unwrap().to_string();
            tools::tasks::update(&serde_json::json!({"id": id, "status": "ready"}), &conn);
            i += 1;
        });
    });
}

fn bench_memory_search_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_search_scaling");
    for size in [100, 1_000, 10_000].iter() {
        let conn = setup();
        for i in 0..*size {
            let value = if i % 100 == 0 {
                format!("{{\"data\":\"needle-{}\"}}", i)
            } else {
                format!("{{\"data\":\"haystack-{}\"}}", i)
            };
            conn.execute(
                "INSERT INTO memory (namespace, key, value, created_at, updated_at) VALUES ('search', ?1, ?2, 0, 0)",
                rusqlite::params![format!("k-{}", i), value],
            ).unwrap();
        }
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            size,
            |b, _| {
                b.iter(|| {
                    tools::memory::search(
                        &serde_json::json!({"query": "needle", "namespace": "search"}),
                        &conn,
                    );
                });
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_memory_roundtrip,
    bench_memory_write_throughput,
    bench_message_roundtrip,
    bench_poll_changes_scaling,
    bench_task_with_deps,
    bench_memory_search_scaling,
);
criterion_main!(benches);
