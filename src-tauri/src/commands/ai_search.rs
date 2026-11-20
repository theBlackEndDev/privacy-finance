use std::sync::Mutex;
use tauri::State;
use crate::db::DbState;
use crate::error::{AppError, AppResult};
use crate::helpers::{ai_entities, ai_templates, ai_sql_builder, ai_result_formatter, ai_local_model, ai_router};
use crate::helpers::ai_engine::Engine;
use crate::models::ai_search::*;
use crate::commands::ai_model::InferenceServerState;

/// Natural language search — user asks a question about their finances.
/// Template matching first (instant), then the local model router as fallback (2-5s).
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
            return answer_from_match(
                intent_match, card_type, AnswerSource::Template, &conn, start,
            );
        }
    } // read_conn released here

    let model_status = ai_local_model::check_model_status(&inference_state.app_data_dir);

    // ── Model router (if installed): the model picks an intent, hand-written SQL answers ──
    if model_status.installed {
        let model_path = ai_local_model::get_model_path(&inference_state.app_data_dir);
        match route_with_model(&query, &model_path, &state, &engine_state) {
            Ok(Some(intent_match)) => {
                // D17: a model mapping is only a proposal; nothing runs until the user confirms.
                let mut result = proposal_response(intent_match);
                result.query_time_ms = start.elapsed().as_millis() as u64;
                return Ok(result);
            }
            Ok(None) => {}
            Err(e) => eprintln!("[AI Search] Router failed: {e}"),
        }
    }

    // ── Decline ──────────────────────────────────────────────────
    let mut result = decline_response(model_status.installed);
    result.query_time_ms = start.elapsed().as_millis() as u64;
    Ok(result)
}

/// Run a router proposal the user confirmed. The intent match comes from the frontend,
/// so any category or account id is checked against the DB before the SQL runs.
#[tauri::command(async)]
pub fn ai_search_confirm(
    intent_match: IntentMatch,
    state: State<DbState>,
) -> AppResult<AISearchResult> {
    let conn = state.read_conn.lock().unwrap();
    confirm_proposal(intent_match, &conn)
}

fn confirm_proposal(intent_match: IntentMatch, conn: &rusqlite::Connection) -> AppResult<AISearchResult> {
    let checks = [
        ("categories", "category", &intent_match.entities.category_id),
        ("accounts", "account", &intent_match.entities.account_id),
    ];
    for (table, label, id) in checks {
        if let Some(id) = id {
            let found: bool = conn.query_row(
                &format!("SELECT EXISTS(SELECT 1 FROM {table} WHERE id = ?1)"),
                [id],
                |r| r.get(0),
            )?;
            if !found {
                return Err(AppError::NotFound(format!("{label} {id}")));
            }
        }
    }
    let card_type = ai_router::card_type_for(&intent_match.intent);
    answer_from_match(
        intent_match, card_type, AnswerSource::ModelRouter, conn, std::time::Instant::now(),
    )
}

/// The response for a router mapping: no card, only the proposal to confirm.
fn proposal_response(intent_match: IntentMatch) -> AISearchResult {
    let mut result = AISearchResult::no_match();
    result.source = AnswerSource::ModelRouter;
    result.suggestions = vec![];
    result.proposal = Some(RouterProposal {
        interpretation: ai_result_formatter::describe_intent(&intent_match),
        intent_match,
    });
    result
}

/// The answer when no template matches and the router declines, fails, or is not installed.
fn decline_response(model_installed: bool) -> AISearchResult {
    let mut result = AISearchResult::no_match();
    result.text_answer = Some("I can't answer that yet. Try one of these:".to_string());
    if !model_installed {
        result.suggestions.insert(0,
            "💡 Install AI Search in Settings for smarter answers to any question.".to_string()
        );
    }
    result
}

/// Run the hand-written SQL for a matched intent and build the answer, tagged with its
/// source and a plain-English interpretation.
fn answer_from_match(
    intent_match: IntentMatch,
    card_type: CardType,
    source: AnswerSource,
    conn: &rusqlite::Connection,
    start: std::time::Instant,
) -> AppResult<AISearchResult> {
    let queries = ai_sql_builder::build_query(&intent_match);
    let (card_data, text_answer) = ai_result_formatter::execute_and_format(
        &queries, &intent_match, conn
    )?;
    Ok(AISearchResult {
        matched: true,
        interpretation: Some(ai_result_formatter::describe_intent(&intent_match)),
        intent: Some(intent_match.intent),
        proposal: None,
        card_type: Some(card_type),
        card_data: Some(card_data),
        text_answer: Some(text_answer),
        source,
        suggestions: vec![],
        query_time_ms: start.elapsed().as_millis() as u64,
    })
}

/// Route a question to an intent with the model. The DB lock is held only to build
/// the grammar and prompt, and again to map the output; never during generation.
fn route_with_model(
    query: &str,
    model_path: &std::path::Path,
    state: &State<DbState>,
    engine_state: &Mutex<Option<Engine>>,
) -> AppResult<Option<IntentMatch>> {
    let today = chrono::Local::now().date_naive();
    let (grammar, prompt) = {
        let conn = state.read_conn.lock().unwrap();
        ai_router::router_inputs(&conn, today)?
    };
    let raw = {
        let mut guard = engine_state.lock().unwrap();
        if guard.is_none() {
            eprintln!("[AI Search] Loading model...");
            *guard = Some(Engine::load(model_path)?);
        }
        guard.as_ref().unwrap().complete_json(&prompt, query, &grammar)?
    };
    let conn = state.read_conn.lock().unwrap();
    Ok(ai_router::route_from_json(&raw, &conn, today))
}

pub(crate) fn try_payee_fallback(query: &str, entities: &mut ExtractedEntities, conn: &rusqlite::Connection) -> Option<(IntentMatch, CardType)> {
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

    fn seeded() -> rusqlite::Connection {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::db::migrations::run_migrations(&conn).unwrap();
        crate::db::migrations::seed_test_data(&conn).unwrap();
        conn
    }

    #[test]
    fn ai_search_sets_source_and_interpretation() {
        let conn = seeded();
        let today = chrono::Local::now().date_naive();

        // Template path
        let query = "how much did I spend last month";
        let entities = ai_entities::extract_entities(query, &conn);
        let (im, card_type) = ai_templates::detect_intent(query, &entities).expect("template match");
        let res = answer_from_match(im, card_type, AnswerSource::Template, &conn, std::time::Instant::now()).unwrap();
        assert!(res.matched);
        assert_eq!(res.source, AnswerSource::Template);
        assert_eq!(res.interpretation.as_deref(), Some("Total spending, last month"));

        // Router path: the mapping is a proposal; confirming it produces the answer
        let raw = r#"{"intent": "income_total", "category": null, "account": null, "merchant": null, "period": "last_month", "amount": null, "operator": null}"#;
        let im = ai_router::route_from_json(raw, &conn, today).expect("router match");
        let res = confirm_proposal(im, &conn).unwrap();
        assert!(res.matched);
        assert!(res.proposal.is_none());
        assert_eq!(res.source, AnswerSource::ModelRouter);
        assert_eq!(res.interpretation.as_deref(), Some("Income, last month"));

        // No match
        let none = AISearchResult::no_match();
        assert_eq!(none.source, AnswerSource::None);
        assert!(none.interpretation.is_none());
    }

    #[test]
    fn router_match_returns_proposal_without_card() {
        let conn = seeded();
        let today = chrono::Local::now().date_naive();
        let raw = r#"{"intent": "income_total", "category": null, "account": null, "merchant": null, "period": "last_month", "amount": null, "operator": null}"#;
        let im = ai_router::route_from_json(raw, &conn, today).expect("router match");

        let res = proposal_response(im);
        assert!(!res.matched);
        assert_eq!(res.source, AnswerSource::ModelRouter);
        assert!(res.card_type.is_none());
        assert!(res.card_data.is_none());
        assert!(res.text_answer.is_none());
        assert!(res.interpretation.is_none());
        let proposal = res.proposal.expect("proposal");
        assert_eq!(proposal.interpretation, "Income, last month");
        assert_eq!(proposal.intent_match.intent, SearchIntent::IncomeTotal);
    }

    #[test]
    fn confirm_runs_proposal() {
        let conn = seeded();
        let today = chrono::Local::now().date_naive();
        let raw = r#"{"intent": "income_total", "category": null, "account": null, "merchant": null, "period": "last_month", "amount": null, "operator": null}"#;
        let im = ai_router::route_from_json(raw, &conn, today).expect("router match");

        // Same IntentMatch through the template path's runner gives the same answer.
        let card_type = ai_router::card_type_for(&im.intent);
        let expected = answer_from_match(
            im.clone(), card_type, AnswerSource::Template, &conn, std::time::Instant::now(),
        ).unwrap();
        // The frontend sends the proposal back as JSON.
        let wire: IntentMatch = serde_json::from_value(serde_json::to_value(&im).unwrap()).unwrap();
        let got = confirm_proposal(wire, &conn).unwrap();
        assert!(got.matched);
        assert_eq!(got.source, AnswerSource::ModelRouter);
        assert_eq!(got.interpretation, expected.interpretation);
        assert_eq!(got.text_answer, expected.text_answer);
        assert_eq!(
            serde_json::to_value(&got.card_data).unwrap(),
            serde_json::to_value(&expected.card_data).unwrap(),
        );

        // An id that is not in the DB is an error, not an answer.
        let mut bad = im.clone();
        bad.entities.category_id = Some("no-such-category".to_string());
        assert!(confirm_proposal(bad, &conn).is_err());
        let mut bad = im;
        bad.entities.account_id = Some("no-such-account".to_string());
        assert!(confirm_proposal(bad, &conn).is_err());
    }

    #[test]
    fn declined_question_returns_suggestions() {
        let conn = seeded();
        let today = chrono::Local::now().date_naive();
        let raw = r#"{"intent": "other", "category": null, "account": null, "merchant": null, "period": null, "amount": null, "operator": null}"#;
        assert!(ai_router::route_from_json(raw, &conn, today).is_none());

        let res = decline_response(true);
        assert!(!res.matched);
        assert_eq!(res.source, AnswerSource::None);
        assert_eq!(res.text_answer.as_deref(), Some("I can't answer that yet. Try one of these:"));
        assert_eq!(res.suggestions.len(), 6);
        assert!(res.suggestions[0].starts_with("How much did I spend on"));

        // Model not installed: the install hint comes first, ahead of the six.
        let res = decline_response(false);
        assert_eq!(res.suggestions.len(), 7);
        assert!(res.suggestions[0].contains("Install AI Search"));
    }
}
