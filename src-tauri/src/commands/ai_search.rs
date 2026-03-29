use tauri::State;
use crate::db::DbState;
use crate::error::{AppError, AppResult};
use crate::helpers::{ai_entities, ai_templates, ai_sql_builder, ai_result_formatter, ai_sql_validator, ai_local_model};
use crate::models::ai_search::*;
use crate::commands::ai_model::InferenceServerState;

/// Natural language search — user asks a question about their finances.
/// Tier 1: template matching (instant). Tier 2: local model fallback (2-5s).
#[tauri::command]
pub fn ai_search(
    query: String,
    state: State<DbState>,
    inference_state: State<InferenceServerState>,
) -> AppResult<AISearchResult> {
    let start = std::time::Instant::now();
    let conn = state.read_conn.lock().unwrap();

    // ── Tier 1: Template matching (instant) ──────────────────────
    let mut entities = ai_entities::extract_entities(&query, &conn);

    if let Some((intent_match, card_type)) = ai_templates::detect_intent(&query, &entities) {
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
            suggestions: vec![],
            query_time_ms: elapsed,
        });
    }

    // ── Payee fallback (still instant) ───────────────────────────
    if let Some((intent_match, card_type)) = try_payee_fallback(&query, &mut entities, &conn) {
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
            suggestions: vec![],
            query_time_ms: elapsed,
        });
    }

    // ── Tier 2: Local model (if installed) ───────────────────────
    let model_status = ai_local_model::check_model_status(&inference_state.app_data_dir);
    if model_status.installed {
        // Ensure the server is running before attempting inference
        ensure_server_running(&inference_state);

        match try_tier2_inference(&query, &conn) {
            Ok(result) => {
                let mut result = result;
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

/// Start the inference server if it's not already running
fn ensure_server_running(inference_state: &InferenceServerState) {
    // Quick health check first
    if ureq::get("http://127.0.0.1:8847/health").call().is_ok() {
        return;
    }

    // Server not responding — try to start it
    let mut process = inference_state.process.lock().unwrap();

    // Check if we have a process that's still alive
    if let Some(ref mut child) = *process {
        match child.try_wait() {
            Ok(None) => {
                // Process is running but not responding yet — give it a moment
                drop(process);
                std::thread::sleep(std::time::Duration::from_secs(2));
                return;
            }
            _ => {} // Process exited, restart below
        }
    }

    // Start the server
    match ai_local_model::start_inference_server(&inference_state.app_data_dir) {
        Ok(child) => {
            *process = Some(child);
            eprintln!("[AI Search] Auto-started inference server for Tier 2 query");
        }
        Err(e) => {
            eprintln!("[AI Search] Failed to auto-start server: {e}");
        }
    }
}

/// Tier 2: Send the question to the local model, get SQL, validate, execute
fn try_tier2_inference(query: &str, conn: &rusqlite::Connection) -> AppResult<AISearchResult> {
    // Call the local model
    let engine = ai_local_model::InferenceEngine::new(std::path::PathBuf::new());
    let raw_sql = ai_local_model::call_inference_server_public(query)?;

    // Validate the generated SQL
    let validated_sql = ai_sql_validator::validate_sql(&raw_sql)?;

    // Execute with a timeout
    let result = conn.query_row(
        &format!("SELECT json_group_array(json_object('row', *)) FROM ({validated_sql}) LIMIT 100"),
        [],
        |row| row.get::<_, String>(0),
    );

    // If json_group_array doesn't work, try direct execution
    let (rows_json, text_answer) = match result {
        Ok(json) => {
            let answer = format_tier2_answer(&validated_sql, &json);
            (json, answer)
        }
        Err(_) => {
            // Fallback: execute directly and format
            let mut stmt = conn.prepare(&validated_sql)
                .map_err(|e| AppError::Validation(format!("generated SQL error: {e}")))?;

            let column_count = stmt.column_count();
            let column_names: Vec<String> = (0..column_count)
                .map(|i| stmt.column_name(i).unwrap_or("?").to_string())
                .collect();

            let mut rows = Vec::new();
            let mut row_iter = stmt.query([])
                .map_err(|e| AppError::Validation(format!("query error: {e}")))?;

            while let Some(row) = row_iter.next()
                .map_err(|e| AppError::Validation(format!("row error: {e}")))? {
                let mut obj = serde_json::Map::new();
                for (i, name) in column_names.iter().enumerate() {
                    let val: rusqlite::types::Value = row.get(i).unwrap_or(rusqlite::types::Value::Null);
                    let json_val = match val {
                        rusqlite::types::Value::Null => serde_json::Value::Null,
                        rusqlite::types::Value::Integer(n) => serde_json::Value::Number(n.into()),
                        rusqlite::types::Value::Real(f) => serde_json::json!(f),
                        rusqlite::types::Value::Text(s) => serde_json::Value::String(s),
                        rusqlite::types::Value::Blob(_) => serde_json::Value::String("[blob]".to_string()),
                    };
                    obj.insert(name.clone(), json_val);
                }
                rows.push(serde_json::Value::Object(obj));
                if rows.len() >= 50 { break; }
            }

            let json = serde_json::to_string(&rows).unwrap_or_default();
            let answer = format_tier2_rows(&column_names, &rows);
            (json, answer)
        }
    };

    // Build a TransactionList card for Tier 2 results
    Ok(AISearchResult {
        matched: true,
        intent: None,
        card_type: Some(CardType::TransactionList),
        card_data: Some(CardData::TransactionList {
            transactions: vec![], // Tier 2 uses text_answer instead
            total: 0,
            count: 0,
            label: "AI Search Result".to_string(),
        }),
        text_answer: Some(text_answer),
        suggestions: vec![],
        query_time_ms: 0,
    })
}

fn format_tier2_answer(sql: &str, json: &str) -> String {
    // Try to extract a meaningful answer from the JSON result
    if let Ok(arr) = serde_json::from_str::<Vec<serde_json::Value>>(json) {
        if arr.len() == 1 {
            // Single row result — likely an aggregate
            if let Some(obj) = arr[0].as_object() {
                let parts: Vec<String> = obj.iter().map(|(k, v)| {
                    let formatted = format_value(v);
                    format!("{}: {}", k, formatted)
                }).collect();
                return parts.join(" | ");
            }
        }
        return format!("{} results found", arr.len());
    }
    "Query executed successfully".to_string()
}

fn format_tier2_rows(columns: &[String], rows: &[serde_json::Value]) -> String {
    if rows.is_empty() {
        return "No results found".to_string();
    }
    if rows.len() == 1 {
        if let Some(obj) = rows[0].as_object() {
            let parts: Vec<String> = obj.iter().map(|(k, v)| {
                format!("{}: {}", k, format_value(v))
            }).collect();
            return parts.join(" | ");
        }
    }
    format!("{} results found", rows.len())
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

    let mut stmt = conn.prepare(
        "SELECT DISTINCT payee FROM transactions WHERE payee != '' AND payee IS NOT NULL AND LOWER(payee) LIKE ?1 LIMIT 5"
    ).ok()?;
    let pattern = format!("%{}%", subject);
    let payees: Vec<String> = stmt.query_map(rusqlite::params![pattern], |row| row.get(0))
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
