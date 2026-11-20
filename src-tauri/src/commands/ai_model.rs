use tauri::{State, AppHandle, Emitter};
use crate::error::AppResult;
use crate::helpers::ai_local_model::{self, ModelStatus};

/// Managed state for the local AI model
pub struct InferenceServerState {
    pub app_data_dir: std::path::PathBuf,
}

/// Check the current status of the AI model
#[tauri::command]
pub fn ai_model_status(state: State<InferenceServerState>) -> AppResult<ModelStatus> {
    Ok(ai_local_model::check_model_status(&state.app_data_dir))
}

/// Download the AI model (emits progress events; runs on the async runtime, not the main thread)
#[tauri::command(async)]
pub fn ai_model_download(app: AppHandle, state: State<InferenceServerState>) -> AppResult<()> {
    let app_data_dir = state.app_data_dir.clone();
    let app_clone = app.clone();

    let result = ai_local_model::download_model(&app_data_dir, move |downloaded, total| {
        let _ = app_clone.emit("ai-model-download-progress", serde_json::json!({
            "downloaded": downloaded,
            "total": total,
            "percent": if total > 0 { (downloaded as f64 / total as f64 * 100.0) as u32 } else { 0 },
        }));
    });

    match result {
        Ok(()) => {
            let _ = app.emit("ai-model-download-complete", serde_json::json!({ "success": true }));
        }
        Err(e) => {
            let _ = app.emit("ai-model-download-complete", serde_json::json!({
                "success": false,
                "error": e.to_string(),
            }));
        }
    }

    Ok(())
}
