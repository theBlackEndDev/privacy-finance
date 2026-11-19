use std::sync::Mutex;
use tauri::State;
use crate::db::DbState;
use crate::error::{AppError, AppResult};
use crate::helpers::{ai_entities, ai_templates, ai_sql_builder, ai_result_formatter, ai_sql_validator, ai_local_model};
use crate::helpers::ai_engine::Engine;
use crate::models::ai_search::*;
use crate::commands::ai_model::InferenceServerState;

/// Natural language search — user asks a question about their finances.
/// Tier 1: template matching (instant). Tier 2: local model fallback (2-5s).
/// Async command: runs on the thread pool so generation does not block the UI.
#[tauri::command(async)]
pub fn ai_search(
    query: String,
    state: State<DbState>,
    inference_state: State<InferenceServerState>,
    engine_state: State<Mutex<Option<Engine>>>,
) -> AppResult<AISearchResult> {
    let start = std::time::Instant::now();

    // ── Tiers 1 and payee fallback, under the read_conn lock ─────
    {
        let conn = state.read_conn.lock().unwrap();
        let mut entities = ai_entities::extract_entities(&query, &conn);

        let matched = match ai_templates::detect_intent(&query, &entities) {
            Some(m) => Some(m),
            None => try_payee_fallback(&query, &mut entities, &conn),
        };

        if let Some((intent_match, card_type)) = matched {
            let queries = ai_sql_builder::build_query(&intent_match);
            let (card_data, text_answer) = ai_result_formatter::execute_and_format(
                &queries, &intent_match, &conn
            )?;
            let elapsed = start.elapsed().as_millis() as u64;
            return Ok(AISearchResult {
                matched: true,
                intent: Some(intent_match.intent),
                card_type: Some(card_type),
                card_data: Some(card_data),
                text_answer: Some(text_answer),
                sql: None,
                suggestions: vec![],
                query_time_ms: elapsed,
            });
        }
    } // read_conn released here

    // ── Tier 2: Local model (if installed) ───────────────────────
    let model_status = ai_local_model::check_model_status(&inference_state.app_data_dir);
    if model_status.installed {
        // Generate without holding the DB lock; re-lock only to execute.
        let model_path = ai_local_model::get_model_path(&inference_state.app_data_dir);
        let system_prompt = {
            let conn = state.read_conn.lock().unwrap();
            ai_local_model::build_system_prompt(&conn)
        };
        let outcome = system_prompt
            .and_then(|prompt| generate_sql(&prompt, &query, &model_path, &engine_state))
            .and_then(|raw_sql| {
                let conn = state.read_conn.lock().unwrap();
                execute_tier2(&raw_sql, &conn)
            });

        match outcome {
            Ok(mut result) => {
                result.query_time_ms = start.elapsed().as_millis() as u64;
                return Ok(result);
            }
            Err(e) => {
                eprintln!("[AI Search] Tier 2 inference failed: {e}");
                // Fall through to no-match
            }
        }
    }

    // ── No match ─────────────────────────────────────────────────
    let mut result = AISearchResult::no_match();
    if !model_status.installed {
        result.suggestions.insert(0,
            "💡 Install AI Search in Settings for smarter answers to any question.".to_string()
        );
    }
    result.query_time_ms = start.elapsed().as_millis() as u64;
    Ok(result)
}

/// Load the engine on first use, then ask it for SQL. No DB lock is held here.
fn generate_sql(
    system_prompt: &str,
    query: &str,
    model_path: &std::path::Path,
    engine_state: &Mutex<Option<Engine>>,
) -> AppResult<String> {
    let mut guard = engine_state.lock().unwrap();
    if guard.is_none() {
        eprintln!("[AI Search] Loading model...");
        *guard = Some(Engine::load(model_path)?);
    }
    guard.as_ref().unwrap().complete(system_prompt, query)
}

const TIER2_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(3);
const TIER2_MAX_ROWS: usize = 50;

/// Tier 2: execute the model's SQL, read-only, with a time limit and a row cap
fn execute_tier2(raw_sql: &str, conn: &rusqlite::Connection) -> AppResult<AISearchResult> {
    execute_tier2_with_timeout(raw_sql, conn, TIER2_TIMEOUT)
}

fn execute_tier2_with_timeout(
    raw_sql: &str,
    conn: &rusqlite::Connection,
    timeout: std::time::Duration,
) -> AppResult<AISearchResult> {
    let sql = ai_sql_validator::extract_sql(raw_sql);
    if sql.is_empty() {
        return Err(AppError::Validation("empty SQL query".to_string()));
    }

    let mut stmt = conn.prepare(&sql)
        .map_err(|e| AppError::Validation(format!("generated SQL error: {e}")))?;
    if !stmt.readonly() {
        return Err(AppError::Validation("generated SQL is not read-only".to_string()));
    }

    // Timer thread interrupts the query after `timeout`; dropping `done_tx` stops it early.
    let handle = conn.get_interrupt_handle();
    let (done_tx, done_rx) = std::sync::mpsc::channel::<()>();
    let timer = std::thread::spawn(move || {
        if let Err(std::sync::mpsc::RecvTimeoutError::Timeout) = done_rx.recv_timeout(timeout) {
            handle.interrupt();
        }
    });

    let result = collect_rows(&mut stmt);

    drop(done_tx);
    let _ = timer.join();

    let (columns, rows) = result?;

    // No rows or one cell: plain text. Anything bigger: a Table card.
    let (card_type, card_data, text_answer) = if rows.is_empty() {
        (None, None, "No results found".to_string())
    } else if rows.len() == 1 && columns.len() == 1 {
        (None, None, format_value(&rows[0][0]))
    } else {
        let text = format!("{} results found", rows.len());
        (Some(CardType::Table), Some(CardData::Table { columns, rows }), text)
    };

    Ok(AISearchResult {
        matched: true,
        intent: None,
        card_type,
        card_data,
        text_answer: Some(text_answer),
        sql: Some(sql),
        suggestions: vec![],
        query_time_ms: 0,
    })
}

fn collect_rows(stmt: &mut rusqlite::Statement) -> AppResult<(Vec<String>, Vec<Vec<serde_json::Value>>)> {
    let column_names: Vec<String> = (0..stmt.column_count())
        .map(|i| stmt.column_name(i).unwrap_or("?").to_string())
        .collect();

    let mut rows = Vec::new();
    let mut row_iter = stmt.query([])
        .map_err(|e| AppError::Validation(format!("query error: {e}")))?;

    while let Some(row) = row_iter.next()
        .map_err(|e| AppError::Validation(format!("row error: {e}")))? {
        let mut cells = Vec::with_capacity(column_names.len());
        for i in 0..column_names.len() {
            let val: rusqlite::types::Value = row.get(i).unwrap_or(rusqlite::types::Value::Null);
            let json_val = match val {
                rusqlite::types::Value::Null => serde_json::Value::Null,
                rusqlite::types::Value::Integer(n) => serde_json::Value::Number(n.into()),
                rusqlite::types::Value::Real(f) => serde_json::json!(f),
                rusqlite::types::Value::Text(s) => serde_json::Value::String(s),
                rusqlite::types::Value::Blob(_) => serde_json::Value::String("[blob]".to_string()),
            };
            cells.push(json_val);
        }
        rows.push(cells);
        if rows.len() >= TIER2_MAX_ROWS { break; }
    }
    Ok((column_names, rows))
}

fn format_value(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                if i.abs() > 100 {
                    // Likely cents — format as dollars
                    format!("${:.2}", i.abs() as f64 / 100.0)
                } else {
                    i.to_string()
                }
            } else {
                n.to_string()
            }
        }
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Null => "—".to_string(),
        other => other.to_string(),
    }
}

fn try_payee_fallback(query: &str, entities: &mut ExtractedEntities, conn: &rusqlite::Connection) -> Option<(IntentMatch, CardType)> {
    let q = query.trim().to_lowercase().replace('?', "");
    let subject = extract_subject(&q)?;

    // Match with punctuation stripped so "trader joes" finds "Trader Joe's"
    let mut stmt = conn.prepare(
        "SELECT DISTINCT payee FROM transactions WHERE payee != '' AND payee IS NOT NULL
         AND (LOWER(payee) LIKE ?1 OR REPLACE(REPLACE(REPLACE(LOWER(payee), '''', ''), '.', ''), '-', '') LIKE ?2) LIMIT 5"
    ).ok()?;
    let pattern = format!("%{}%", subject);
    let pattern_stripped = format!("%{}%", ai_entities::strip_punctuation(&subject));
    let payees: Vec<String> = stmt.query_map(rusqlite::params![pattern, pattern_stripped], |row| row.get(0))
        .ok()?.filter_map(|r| r.ok()).collect();

    if !payees.is_empty() {
        entities.merchant = Some(payees[0].clone());
        return Some((
            IntentMatch {
                intent: SearchIntent::TransactionSearch,
                confidence: 0.7,
                entities: entities.clone(),
            },
            CardType::TransactionList,
        ));
    }

    if q.contains("spend") || q.contains("spent") || q.contains("how much") || q.contains("show") || q.contains("how many") || q.contains("did i go") {
        entities.merchant = Some(subject.clone());
        return Some((
            IntentMatch {
                intent: SearchIntent::TransactionSearch,
                confidence: 0.5,
                entities: entities.clone(),
            },
            CardType::TransactionList,
        ));
    }

    None
}

fn extract_subject(query: &str) -> Option<String> {
    let prepositions = ["on ", "at ", "for ", "to ", "in "];
    for prep in prepositions {
        if let Some(idx) = query.rfind(prep) {
            let after = query[idx + prep.len()..].trim();
            let cleaned = after
                .replace("last month", "").replace("this month", "")
                .replace("last year", "").replace("this year", "")
                .replace("this week", "").replace("last week", "")
                .replace("today", "").replace("yesterday", "");
            let subject = cleaned.trim().to_string();
            if !subject.is_empty() && subject.len() >= 3 {
                return Some(subject);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    fn seeded() -> rusqlite::Connection {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::db::migrations::run_migrations(&conn).unwrap();
        crate::db::migrations::seed_test_data(&conn).unwrap();
        conn
    }

    #[test]
    fn tier2_rejects_write_statement() {
        let conn = seeded();
        let before: i64 = conn.query_row("SELECT COUNT(*) FROM transactions", [], |r| r.get(0)).unwrap();
        assert!(before > 0);

        let res = execute_tier2("WITH a AS (SELECT 1) DELETE\nFROM transactions", &conn);
        assert!(res.unwrap_err().to_string().contains("not read-only"));

        let after: i64 = conn.query_row("SELECT COUNT(*) FROM transactions", [], |r| r.get(0)).unwrap();
        assert_eq!(before, after);
    }

    #[test]
    fn tier2_multi_row_returns_table() {
        let conn = seeded();
        let res = execute_tier2("SELECT id, name FROM categories LIMIT 3;", &conn).unwrap();
        assert_eq!(res.card_type, Some(CardType::Table));
        match res.card_data {
            Some(CardData::Table { columns, rows }) => {
                assert_eq!(columns, vec!["id", "name"]);
                assert_eq!(rows.len(), 3);
                assert!(rows.iter().all(|r| r.len() == 2));
            }
            other => panic!("expected Table card, got {other:?}"),
        }
        assert!(res.sql.unwrap().starts_with("SELECT id, name FROM categories"));

        let single = execute_tier2("SELECT COUNT(*) FROM transactions", &conn).unwrap();
        assert!(single.card_type.is_none());
        assert!(single.text_answer.is_some());
    }

    #[test]
    fn tier2_interrupts_slow_query() {
        let conn = seeded();
        let sql = "WITH RECURSIVE c(x) AS (SELECT 1 UNION ALL SELECT x + 1 FROM c) SELECT MAX(x) FROM c";
        let start = Instant::now();
        let res = execute_tier2_with_timeout(sql, &conn, Duration::from_millis(200));
        assert!(res.is_err());
        assert!(start.elapsed() < Duration::from_secs(3));
    }
}
