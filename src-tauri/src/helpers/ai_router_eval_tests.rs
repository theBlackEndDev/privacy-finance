//! Router eval fixture and its guard test.

use std::path::Path;
use std::time::Instant;

use chrono::NaiveDate;
use rusqlite::Connection;
use serde::Deserialize;

use crate::commands::ai_search::try_payee_fallback;
use crate::db::migrations::{run_migrations, seed_test_data};
use crate::helpers::ai_entities::extract_entities;
use crate::helpers::ai_engine::Engine;
use crate::helpers::ai_result_formatter::execute_and_format;
use crate::helpers::ai_router::{route_from_json, router_inputs, to_intent_match, RouterOutput};
use crate::helpers::ai_sql_builder::build_query;
use crate::models::ai_search::IntentMatch;
use crate::helpers::ai_templates::detect_intent;

/// `expected` in the fixture: a D2 object, or the string `"other"`.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub(crate) enum Expected {
    Other(String),
    Route(RouterOutput),
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct FixtureItem {
    pub question: String,
    pub expected: Expected,
}

pub(crate) fn load_fixture() -> Vec<FixtureItem> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/ai-router-eval.json");
    serde_json::from_str(&std::fs::read_to_string(path).expect("read fixture"))
        .expect("parse fixture")
}

const INTENTS: [&str; 12] = [
    "spending_total", "spending_by_category", "spending_comparison", "income_total",
    "balance_check", "budget_status", "transaction_search", "top_merchants",
    "top_expenses", "recurring_check", "net_worth", "category_detail",
];

const PERIODS: [&str; 12] = [
    "today", "yesterday", "this_week", "last_week", "this_month", "last_month",
    "this_quarter", "this_year", "last_year", "last_7_days", "last_30_days", "last_90_days",
];

fn seeded() -> Connection {
    let conn = Connection::open_in_memory().expect("in-memory db");
    run_migrations(&conn).expect("migrations");
    seed_test_data(&conn).expect("seed test data");
    conn
}

fn name_exists(conn: &Connection, table: &str, name: &str) -> bool {
    let sql = format!("SELECT COUNT(*) FROM {table} WHERE name = ?1");
    conn.query_row(&sql, [name], |r| r.get::<_, i64>(0)).unwrap() > 0
}

fn payee_exists(conn: &Connection, merchant: &str) -> bool {
    conn.query_row(
        "SELECT COUNT(*) FROM transactions WHERE LOWER(payee) LIKE ?1",
        [format!("%{}%", merchant.to_lowercase())],
        |r| r.get::<_, i64>(0),
    )
    .unwrap()
        > 0
}

fn valid_period(p: &str) -> bool {
    PERIODS.contains(&p)
        || (p.len() == 7
            && p.as_bytes()[4] == b'-'
            && chrono::NaiveDate::parse_from_str(&format!("{p}-01"), "%Y-%m-%d").is_ok())
}

#[test]
fn router_fixture_misses_templates() {
    let conn = seeded();
    let items = load_fixture();

    let supported = items.iter().filter(|i| matches!(i.expected, Expected::Route(_))).count();
    let unsupported = items.len() - supported;
    assert!(supported >= 40, "need at least 40 supported questions, have {supported}");
    assert!(unsupported >= 10, "need at least 10 unsupported questions, have {unsupported}");

    let mut problems: Vec<String> = Vec::new();

    for item in &items {
        let q = &item.question;

        let entities = extract_entities(q, &conn);
        if detect_intent(q, &entities).is_some() {
            problems.push(format!("template matches: {q}"));
        }
        let mut fallback_entities = entities.clone();
        if try_payee_fallback(q, &mut fallback_entities, &conn).is_some() {
            problems.push(format!("payee fallback matches: {q}"));
        }

        match &item.expected {
            Expected::Other(s) => {
                if s != "other" {
                    problems.push(format!("expected string must be \"other\" ({q})"));
                }
            }
            Expected::Route(r) => {
                if !INTENTS.contains(&r.intent.as_str()) {
                    problems.push(format!("unknown intent {:?}: {q}", r.intent));
                }
                if let Some(c) = &r.category {
                    if !name_exists(&conn, "categories", c) {
                        problems.push(format!("category {c:?} not seeded: {q}"));
                    }
                }
                if let Some(a) = &r.account {
                    if !name_exists(&conn, "accounts", a) {
                        problems.push(format!("account {a:?} not seeded: {q}"));
                    }
                }
                if let Some(m) = &r.merchant {
                    if !payee_exists(&conn, m) {
                        problems.push(format!("payee {m:?} not seeded: {q}"));
                    }
                }
                if let Some(p) = &r.period {
                    if !valid_period(p) {
                        problems.push(format!("invalid period {p:?}: {q}"));
                    }
                }
                if let Some(o) = &r.operator {
                    if o != "over" && o != "under" {
                        problems.push(format!("invalid operator {o:?}: {q}"));
                    }
                }
            }
        }
    }

    assert!(problems.is_empty(), "fixture problems:\n{}", problems.join("\n"));
}

fn pct(n: usize, d: usize) -> f64 {
    if d == 0 { 0.0 } else { 100.0 * n as f64 / d as f64 }
}

/// The answer an `IntentMatch` gives on the seeded DB: `card_data` as JSON and `text_answer`.
fn answer_of(im: &IntentMatch, conn: &Connection) -> Result<(serde_json::Value, String), String> {
    let (card, text) = execute_and_format(&build_query(im), im, conn).map_err(|e| e.to_string())?;
    let card = serde_json::to_value(&card).map_err(|e| e.to_string())?;
    Ok((card, text))
}

fn text_of(a: &Result<(serde_json::Value, String), String>) -> String {
    match a {
        Ok((_, text)) => text.clone(),
        Err(e) => format!("<error: {e}>"),
    }
}

/// Scores the router per D14 (same answer), with the D11 slot-equality rule as
/// `strict_accuracy`. Run with
/// `MODEL_PATH=<file> cargo test --release ai_router_eval -- --ignored --nocapture`.
#[test]
#[ignore]
fn ai_router_eval() {
    let path = std::env::var("MODEL_PATH").expect("set MODEL_PATH to a .gguf file");
    let engine = Engine::load(Path::new(&path)).expect("load model");
    let conn = seeded();
    let today = NaiveDate::from_ymd_opt(2026, 3, 31).unwrap();
    let items = load_fixture();

    let (mut supported, mut correct, mut wrong, mut strict_correct) = (0usize, 0usize, 0usize, 0usize);
    let (mut unsupported, mut declined, mut parse_failures) = (0usize, 0usize, 0usize);
    let mut total_ms = 0u128;
    let mut failures: Vec<String> = Vec::new();

    for item in &items {
        let (grammar, prompt) = router_inputs(&conn, today).expect("router inputs");
        // Same steps as `route`, split so the raw model output can be reported.
        let started = Instant::now();
        let result = engine.complete_json(&prompt, &item.question, &grammar);
        total_ms += started.elapsed().as_millis();

        let raw = match result {
            Ok(raw) => raw,
            Err(e) => {
                parse_failures += 1;
                match &item.expected {
                    Expected::Route(_) => supported += 1,
                    Expected::Other(_) => unsupported += 1,
                }
                failures.push(format!("FAILED {:?}\n  model error: {e}", item.question));
                continue;
            }
        };
        let parsed: Option<RouterOutput> = serde_json::from_str(raw.trim()).ok();
        if parsed.is_none() {
            parse_failures += 1;
        }
        let actual_im = route_from_json(&raw, &conn, today);
        let mapped = actual_im.is_some();

        match &item.expected {
            Expected::Route(expected) => {
                supported += 1;
                if mapped && parsed.as_ref() == Some(expected) {
                    strict_correct += 1;
                }
                let expected_json = serde_json::to_string(expected).unwrap_or_default();
                if let Some(actual_im) = &actual_im {
                    let actual = answer_of(actual_im, &conn);
                    let want = to_intent_match(expected, &conn, today)
                        .ok_or_else(|| "expected output does not map".to_string())
                        .and_then(|im| answer_of(&im, &conn));
                    let same = matches!((&want, &actual), (Ok(w), Ok(a)) if w == a);
                    if same {
                        correct += 1;
                    } else {
                        wrong += 1;
                        failures.push(format!(
                            "WRONG {:?}\n  expected: {expected_json}\n  raw: {raw}\n  expected text_answer: {}\n  actual text_answer: {}",
                            item.question,
                            text_of(&want),
                            text_of(&actual)
                        ));
                    }
                } else {
                    failures.push(format!(
                        "DECLINED {:?}\n  expected: {expected_json}\n  raw: {raw}",
                        item.question
                    ));
                }
            }
            Expected::Other(_) => {
                unsupported += 1;
                // An unparseable answer is a failure, not a decline.
                if parsed.is_some() && !mapped {
                    declined += 1;
                } else {
                    failures.push(format!("NOT DECLINED {:?}\n  raw: {raw}", item.question));
                }
            }
        }
    }

    for f in &failures {
        println!("{f}");
    }
    println!(
        "router summary: accuracy={:.1}% wrong={:.1}% unsupported_declined={:.1}% parse_failures={} mean_ms={} strict_accuracy={:.1}%",
        pct(correct, supported),
        pct(wrong, supported),
        pct(declined, unsupported),
        parse_failures,
        total_ms / items.len().max(1) as u128,
        pct(strict_correct, supported),
    );
}
