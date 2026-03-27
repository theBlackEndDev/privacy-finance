use tauri::State;
use crate::db::DbState;
use crate::error::AppResult;
use crate::helpers::{ai_entities, ai_templates, ai_sql_builder, ai_result_formatter};
use crate::models::ai_search::AISearchResult;

/// Natural language search — user asks a question about their finances,
/// the engine extracts entities, detects intent, builds SQL, executes,
/// and returns a rich answer card.
#[tauri::command]
pub fn ai_search(query: String, state: State<DbState>) -> AppResult<AISearchResult> {
    let start = std::time::Instant::now();
    let conn = state.read_conn.lock().unwrap();

    // 1. Extract entities (categories, dates, amounts, merchants)
    let entities = ai_entities::extract_entities(&query, &conn);

    // 2. Detect intent from templates
    let (intent_match, card_type) = match ai_templates::detect_intent(&query, &entities) {
        Some(result) => result,
        None => {
            let mut result = AISearchResult::no_match();
            result.query_time_ms = start.elapsed().as_millis() as u64;
            return Ok(result);
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
