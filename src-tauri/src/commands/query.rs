use tauri::State;
use crate::db::DbState;
use crate::error::AppResult;
use crate::helpers::template_engine::{infer, InferenceResult};

/// Natural language query — user asks a question about their finances,
/// the template engine matches it to SQL and returns a plain-English answer.
#[tauri::command]
pub fn ask_question(question: String, state: State<DbState>) -> AppResult<InferenceResult> {
    let conn = state.read_conn.lock().unwrap();
    infer(&question, &conn)
}
