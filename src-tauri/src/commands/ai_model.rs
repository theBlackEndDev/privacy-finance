use std::sync::{Arc, Mutex};
use tauri::{State, AppHandle, Emitter};
use crate::error::AppResult;
use crate::helpers::ai_local_model::{self, ModelStatus};

/// Managed state for the inference server process
pub struct InferenceServerState {
    pub process: Arc<Mutex<Option<std::process::Child>>>,
    pub app_data_dir: std::path::PathBuf,
}

/// Check the current status of the AI model
#[tauri::command]
pub fn ai_model_status(state: State<InferenceServerState>) -> AppResult<ModelStatus> {
    Ok(ai_local_model::check_model_status(&state.app_data_dir))
}

/// Download the AI model (runs in background, emits progress events, auto-starts server)
#[tauri::command]
pub fn ai_model_download(app: AppHandle, state: State<InferenceServerState>) -> AppResult<()> {
    let app_data_dir = state.app_data_dir.clone();
    let app_clone = app.clone();
    let process = state.process.clone();

    std::thread::spawn(move || {
        let result = ai_local_model::download_model(&app_data_dir, move |downloaded, total| {
            let _ = app_clone.emit("ai-model-download-progress", serde_json::json!({
                "downloaded": downloaded,
                "total": total,
                "percent": if total > 0 { (downloaded as f64 / total as f64 * 100.0) as u32 } else { 0 },
            }));
        });

        match result {
            Ok(()) => {
                // Auto-start the inference server after successful download
                match ai_local_model::start_inference_server(&app_data_dir) {
                    Ok(child) => {
                        if let Ok(mut proc) = process.lock() {
                            *proc = Some(child);
                        }
                        eprintln!("[AI Search] Server auto-started after install");
                    }
                    Err(e) => {
                        eprintln!("[AI Search] Server auto-start failed: {e}");
                    }
                }
                let _ = app.emit("ai-model-download-complete", serde_json::json!({ "success": true }));
            }
            Err(e) => {
                let _ = app.emit("ai-model-download-complete", serde_json::json!({
                    "success": false,
                    "error": e.to_string(),
                }));
            }
        }
    });

    Ok(())
}

/// Start the local inference server
#[tauri::command]
pub fn ai_model_start(state: State<InferenceServerState>) -> AppResult<()> {
    let mut process = state.process.lock().unwrap();

    // Already running?
    if let Some(ref mut child) = *process {
        match child.try_wait() {
            Ok(None) => return Ok(()), // still running
            _ => {} // exited, restart
        }
    }

    match ai_local_model::start_inference_server(&state.app_data_dir) {
        Ok(child) => {
            *process = Some(child);
            Ok(())
        }
        Err(e) => Err(e),
    }
}

/// Stop the local inference server
#[tauri::command]
pub fn ai_model_stop(state: State<InferenceServerState>) -> AppResult<()> {
    let mut process = state.process.lock().unwrap();
    if let Some(ref mut child) = *process {
        let _ = child.kill();
        let _ = child.wait();
    }
    *process = None;
    Ok(())
}

/// Check if the inference server is running and responsive
#[tauri::command]
pub fn ai_model_health(state: State<InferenceServerState>) -> AppResult<bool> {
    let process = state.process.lock().unwrap();
    if process.is_none() {
        return Ok(false);
    }

    // Try to ping the server
    match ureq::get("http://127.0.0.1:8847/health").call() {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}
