use tauri::State;
use crate::db::DbState;
use crate::error::AppResult;
use crate::helpers::{ai_entities, ai_templates, ai_sql_builder, ai_result_formatter};
use crate::models::ai_search::{AISearchResult, SearchIntent, CardType, IntentMatch, ExtractedEntities};

/// Natural language search — user asks a question about their finances,
/// the engine extracts entities, detects intent, builds SQL, executes,
/// and returns a rich answer card.
#[tauri::command]
pub fn ai_search(query: String, state: State<DbState>) -> AppResult<AISearchResult> {
    let start = std::time::Instant::now();
    let conn = state.read_conn.lock().unwrap();

    // 1. Extract entities (categories, dates, amounts, merchants)
    let mut entities = ai_entities::extract_entities(&query, &conn);

    // 2. Detect intent from templates
    let (intent_match, card_type) = match ai_templates::detect_intent(&query, &entities) {
        Some(result) => result,
        None => {
            // Fallback: if the query looks like a spending question but no category matched,
            // try to extract the subject as a payee search
            if let Some(fallback) = try_payee_fallback(&query, &mut entities, &conn) {
                fallback
            } else {
                let mut result = AISearchResult::no_match();
                result.query_time_ms = start.elapsed().as_millis() as u64;
                return Ok(result);
            }
        }
    };

    // 3. Build parameterized SQL
    let queries = ai_sql_builder::build_query(&intent_match);

    // 4. Execute and format into card data
    let (card_data, text_answer) = ai_result_formatter::execute_and_format(
        &queries, &intent_match, &conn
    )?;

    let elapsed = start.elapsed().as_millis() as u64;

    Ok(AISearchResult {
        matched: true,
        intent: Some(intent_match.intent),
        card_type: Some(card_type),
        card_data: Some(card_data),
        text_answer: Some(text_answer),
        suggestions: vec![],
        query_time_ms: elapsed,
    })
}

/// When intent detection fails, try to extract a subject word from the query
/// and treat it as a merchant/payee search. This handles cases like
/// "how much did I spend on electricity" where "electricity" isn't a category
/// but might match payees like "Electric Company".
fn try_payee_fallback(query: &str, entities: &mut ExtractedEntities, conn: &rusqlite::Connection) -> Option<(IntentMatch, CardType)> {
    let q = query.trim().to_lowercase().replace('?', "");

    // Extract the subject — typically after "on", "at", "for"
    let subject = extract_subject(&q)?;

    // Search payees for this subject
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

    // Even if no payee match, if the query looks like a spending question,
    // do a general text search across payees and descriptions
    if q.contains("spend") || q.contains("spent") || q.contains("how much") || q.contains("show") {
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
    // Look for words after "on", "at", "for", "to"
    let prepositions = ["on ", "at ", "for ", "to "];
    for prep in prepositions {
        if let Some(idx) = query.rfind(prep) {
            let after = query[idx + prep.len()..].trim();
            // Remove trailing date/period words
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
