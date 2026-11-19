//! Manual accuracy eval for the local SQL model. Run with:
//! `MODEL_PATH=/path/to/model.gguf cargo test ai_eval -- --ignored --nocapture`

use std::path::Path;
use std::time::Instant;

use rusqlite::types::Value;
use rusqlite::Connection;
use serde::Deserialize;

use crate::db::migrations::{run_migrations, seed_test_data};
use crate::helpers::ai_engine::Engine;
use crate::helpers::ai_local_model::build_system_prompt;
use crate::helpers::ai_sql_validator::extract_sql;

#[derive(Deserialize)]
struct Pair {
    question: String,
    gold_sql: String,
}

/// Run a read-only query and return its rows as a sorted multiset.
/// Column names and row order are ignored; reals are rounded to 4 places.
fn result_multiset(conn: &Connection, sql: &str) -> Result<Vec<Vec<String>>, String> {
    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    if !stmt.readonly() {
        return Err("not read-only".to_string());
    }
    let cols = stmt.column_count();
    let mut rows_out = Vec::new();
    let mut rows = stmt.query([]).map_err(|e| e.to_string())?;
    while let Some(row) = rows.next().map_err(|e| e.to_string())? {
        let mut cells = Vec::with_capacity(cols);
        for i in 0..cols {
            let v: Value = row.get(i).map_err(|e| e.to_string())?;
            cells.push(match v {
                Value::Null => "NULL".to_string(),
                Value::Integer(n) => n.to_string(),
                Value::Real(f) => {
                    if f.fract() == 0.0 && f.abs() < 1e15 {
                        (f as i64).to_string()
                    } else {
                        format!("{f:.4}")
                    }
                }
                Value::Text(s) => s,
                Value::Blob(_) => "[blob]".to_string(),
            });
        }
        rows_out.push(cells);
    }
    rows_out.sort();
    Ok(rows_out)
}

#[test]
#[ignore]
fn ai_eval() {
    let path = std::env::var("MODEL_PATH").expect("set MODEL_PATH to a .gguf file");
    let conn = Connection::open_in_memory().expect("in-memory db");
    run_migrations(&conn).expect("migrations");
    seed_test_data(&conn).expect("seed test data");

    let engine = Engine::load(Path::new(&path)).expect("load model");
    let system = build_system_prompt(&conn).expect("system prompt");

    let fixture = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/ai-eval.json");
    let pairs: Vec<Pair> =
        serde_json::from_str(&std::fs::read_to_string(fixture).expect("read fixture"))
            .expect("parse fixture");
    assert!(!pairs.is_empty(), "fixture is empty");

    let mut passed = 0usize;
    let mut total_ms = 0u128;
    let mut failures: Vec<String> = Vec::new();

    for pair in &pairs {
        let start = Instant::now();
        let raw = engine.complete(&system, &pair.question);
        total_ms += start.elapsed().as_millis();

        let verdict = match raw {
            Err(e) => Err(format!("engine error: {e}")),
            Ok(raw) => {
                let sql = extract_sql(&raw);
                let gold = result_multiset(&conn, &pair.gold_sql)
                    .unwrap_or_else(|e| panic!("gold SQL failed for {:?}: {e}", pair.question));
                match result_multiset(&conn, &sql) {
                    Err(e) => Err(format!("{e}; sql: {sql}")),
                    Ok(got) if got == gold => Ok(()),
                    Ok(_) => Err(format!("result mismatch; sql: {sql}")),
                }
            }
        };
        match verdict {
            Ok(()) => passed += 1,
            Err(why) => failures.push(format!("{} -> {why}", pair.question)),
        }
    }

    let n = pairs.len();
    println!("accuracy: {:.1}% ({passed}/{n})", passed as f64 * 100.0 / n as f64);
    println!("mean latency: {} ms", total_ms / n as u128);
    println!("failing questions ({}):", failures.len());
    for f in &failures {
        println!("  - {f}");
    }
}
