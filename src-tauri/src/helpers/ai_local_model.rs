use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use serde::Serialize;
use crate::error::{AppError, AppResult};

/// Status of the local AI model
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelStatus {
    pub installed: bool,
    pub model_name: String,
    pub model_path: Option<String>,
    pub model_size_bytes: Option<u64>,
    pub ready: bool,
}

/// Configuration for the local model
pub const MODEL_NAME: &str = "qwen2.5-coder-1.5b-instruct";
pub const MODEL_FILENAME: &str = "qwen2.5-coder-1.5b-instruct-q4_k_m.gguf";
pub const MODEL_URL: &str = "https://huggingface.co/Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF/resolve/main/qwen2.5-coder-1.5b-instruct-q4_k_m.gguf";
pub const MODEL_SIZE_APPROX: u64 = 1_000_000_000; // ~1GB

/// System prompt that teaches the model our DB schema
pub const SYSTEM_PROMPT: &str = r#"You are a SQL query generator for a personal finance SQLite database.
Given a natural language question about the user's finances, generate a single SQL SELECT query that answers it.

Database schema:
- transactions (id TEXT, account_id TEXT, date TEXT 'YYYY-MM-DD', amount INTEGER in cents, kind TEXT 'income'|'expense'|'transfer', category_id TEXT, payee TEXT, description TEXT)
- categories (id TEXT, name TEXT, color TEXT, type TEXT 'income'|'expense')
- accounts (id TEXT, name TEXT, type TEXT 'checking'|'savings'|'credit_card'|'cash'|'investment'|'other', currency_code TEXT, opening_balance INTEGER cents, is_archived INTEGER 0|1)
- budgets (id TEXT, month TEXT 'YYYY-MM')
- budget_items (id TEXT, budget_id TEXT, category_id TEXT, amount INTEGER cents)
- recurring_transactions (id TEXT, payee TEXT, amount INTEGER cents, frequency TEXT, next_date TEXT, is_active INTEGER 0|1)

Important rules:
- All amounts are stored in cents (e.g., $10.50 = 1050). Use ABS(amount) for totals, amount/100.0 for dollar display.
- Dates are TEXT in YYYY-MM-DD format. Use date() and strftime() for date math.
- JOIN categories ON transactions.category_id = categories.id for category names.
- JOIN accounts ON transactions.account_id = accounts.id for account names.
- Use LOWER() for case-insensitive matching on payee and category names.
- For "last month": WHERE date >= date('now','start of month','-1 month') AND date < date('now','start of month')
- For "this month": WHERE date >= date('now','start of month')
- For "this year": WHERE date >= date('now','start of year')
- Only generate SELECT statements. Never generate INSERT, UPDATE, DELETE, DROP, or any other mutation.

Respond with ONLY the SQL query. No explanation, no markdown, no comments."#;

/// Get the model storage directory
pub fn get_models_dir(app_data_dir: &PathBuf) -> PathBuf {
    app_data_dir.join("models")
}

/// Get the full path to the model file
pub fn get_model_path(app_data_dir: &PathBuf) -> PathBuf {
    get_models_dir(app_data_dir).join(MODEL_FILENAME)
}

/// Check if the model is installed
pub fn check_model_status(app_data_dir: &PathBuf) -> ModelStatus {
    let model_path = get_model_path(app_data_dir);
    let installed = model_path.exists();
    let size = if installed {
        std::fs::metadata(&model_path).ok().map(|m| m.len())
    } else {
        None
    };

    ModelStatus {
        installed,
        model_name: MODEL_NAME.to_string(),
        model_path: if installed { Some(model_path.to_string_lossy().to_string()) } else { None },
        model_size_bytes: size,
        ready: installed,
    }
}

/// Shared state for the loaded model inference engine
pub struct InferenceEngine {
    pub loaded: bool,
    pub app_data_dir: PathBuf,
}

impl InferenceEngine {
    pub fn new(app_data_dir: PathBuf) -> Self {
        Self { loaded: false, app_data_dir }
    }

    /// Run inference — send a question to the local model, get SQL back
    pub fn infer(&self, question: &str) -> AppResult<String> {
        if !self.loaded {
            let model_path = get_model_path(&self.app_data_dir);
            if !model_path.exists() {
                return Err(AppError::Validation(
                    "AI model not installed. Go to Settings → AI Search to install.".to_string()
                ));
            }
        }

        // Call the local model via HTTP (llama-server running as managed process)
        // or via direct llama-cpp-2 bindings
        let result = call_inference_server(question)?;
        Ok(result)
    }
}

/// Call the local inference server (llama-server on port 8847) — public for ai_search
pub fn call_inference_server_public(question: &str) -> AppResult<String> {
    call_inference_server(question)
}

/// Call the local inference server (llama-server on port 8847)
fn call_inference_server(question: &str) -> AppResult<String> {
    let prompt = format!(
        "<|im_start|>system\n{}<|im_end|>\n<|im_start|>user\n{}<|im_end|>\n<|im_start|>assistant\n",
        SYSTEM_PROMPT, question
    );

    // Use a blocking HTTP client (ureq) to call the local server
    let response = ureq::post("http://127.0.0.1:8847/completion")
        .set("Content-Type", "application/json")
        .send_json(ureq::json!({
            "prompt": prompt,
            "n_predict": 512,
            "temperature": 0.1,
            "stop": ["<|im_end|>", "\n\n"],
        }));

    match response {
        Ok(resp) => {
            let body: serde_json::Value = resp.into_json()
                .map_err(|e| AppError::Validation(format!("failed to parse model response: {e}")))?;
            let content = body["content"].as_str()
                .ok_or_else(|| AppError::Validation("no content in model response".to_string()))?;
            Ok(content.trim().to_string())
        }
        Err(e) => {
            Err(AppError::Validation(format!(
                "AI model not running. Please install and enable AI Search in Settings. Error: {e}"
            )))
        }
    }
}

/// Download the model file with progress reporting via a callback
pub fn download_model(
    app_data_dir: &PathBuf,
    progress_callback: impl Fn(u64, u64) + Send + 'static,
) -> AppResult<()> {
    let models_dir = get_models_dir(app_data_dir);
    std::fs::create_dir_all(&models_dir)
        .map_err(|e| AppError::Io(e))?;

    let model_path = get_model_path(app_data_dir);

    // Download with progress
    let response = ureq::get(MODEL_URL)
        .call()
        .map_err(|e| AppError::Validation(format!("failed to download model: {e}")))?;

    let total_size = response.header("Content-Length")
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(MODEL_SIZE_APPROX);

    let mut file = std::fs::File::create(&model_path)
        .map_err(|e| AppError::Io(e))?;

    let mut reader = response.into_reader();
    let mut downloaded: u64 = 0;
    let mut buffer = [0u8; 65536]; // 64KB chunks

    loop {
        let bytes_read = std::io::Read::read(&mut reader, &mut buffer)
            .map_err(|e| AppError::Io(e))?;
        if bytes_read == 0 { break; }

        std::io::Write::write_all(&mut file, &buffer[..bytes_read])
            .map_err(|e| AppError::Io(e))?;

        downloaded += bytes_read as u64;
        progress_callback(downloaded, total_size);
    }

    Ok(())
}

/// Start the llama-server process for the local model
pub fn start_inference_server(app_data_dir: &PathBuf) -> AppResult<std::process::Child> {
    let model_path = get_model_path(app_data_dir);
    if !model_path.exists() {
        return Err(AppError::Validation("model not installed".to_string()));
    }

    // Try to find llama-server in the app's binaries directory or system PATH
    let server_binary = find_llama_server(app_data_dir)?;

    let child = std::process::Command::new(server_binary)
        .args([
            "-m", &model_path.to_string_lossy(),
            "--host", "127.0.0.1",
            "--port", "8847",
            "-ngl", "0",  // CPU only
            "-c", "2048", // context length
            "--log-disable",
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| AppError::Validation(format!("failed to start inference server: {e}")))?;

    // Wait a moment for the server to start
    std::thread::sleep(std::time::Duration::from_secs(2));

    Ok(child)
}

fn find_llama_server(app_data_dir: &PathBuf) -> AppResult<PathBuf> {
    // Check in app's binaries directory
    let app_binary = app_data_dir.join("bin").join("llama-server");
    if app_binary.exists() {
        return Ok(app_binary);
    }

    // Check system PATH
    if let Ok(path) = which::which("llama-server") {
        return Ok(path);
    }

    // Check common locations
    let common_paths = [
        "/usr/local/bin/llama-server",
        "/usr/bin/llama-server",
        "/opt/homebrew/bin/llama-server",
    ];
    for p in common_paths {
        let path = PathBuf::from(p);
        if path.exists() {
            return Ok(path);
        }
    }

    Err(AppError::Validation(
        "llama-server not found. The AI Search runtime needs to be installed.".to_string()
    ))
}
