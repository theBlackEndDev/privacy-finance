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
    let server_path = app_data_dir.join("bin").join("llama-server");
    let model_installed = model_path.exists();
    let server_installed = server_path.exists() || which::which("llama-server").is_ok();
    let size = if model_installed {
        std::fs::metadata(&model_path).ok().map(|m| m.len())
    } else {
        None
    };

    ModelStatus {
        installed: model_installed && server_installed,
        model_name: MODEL_NAME.to_string(),
        model_path: if model_installed { Some(model_path.to_string_lossy().to_string()) } else { None },
        model_size_bytes: size,
        ready: model_installed && server_installed,
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

/// Get the platform-specific archive asset name suffix for llama.cpp releases.
/// Returns (asset_contains, archive_ext) — e.g. ("macos-arm64", ".tar.gz")
fn get_platform_asset_pattern() -> (&'static str, &'static str) {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    { ("macos-arm64", ".tar.gz") }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    { ("macos-x64", ".tar.gz") }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    { ("ubuntu-x64", ".tar.gz") }
    #[cfg(not(any(
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "macos", target_arch = "x86_64"),
        all(target_os = "linux", target_arch = "x86_64"),
    )))]
    { ("", "") }
}

/// Query GitHub API for the latest llama.cpp release and find the right archive URL.
fn resolve_server_archive_url() -> AppResult<String> {
    let (pattern, ext) = get_platform_asset_pattern();
    if pattern.is_empty() {
        return Err(AppError::Validation("unsupported platform for AI Search".to_string()));
    }

    let response = ureq::get("https://api.github.com/repos/ggml-org/llama.cpp/releases/latest")
        .set("User-Agent", "privacy-finance")
        .call()
        .map_err(|e| AppError::Validation(format!("failed to query GitHub releases: {e}")))?;

    let body: serde_json::Value = response.into_json()
        .map_err(|e| AppError::Validation(format!("failed to parse GitHub response: {e}")))?;

    let assets = body["assets"].as_array()
        .ok_or_else(|| AppError::Validation("no assets in GitHub release".to_string()))?;

    // Find the archive that matches our platform — must contain pattern, end with ext,
    // and NOT contain cuda/rocm/vulkan/opencl/sycl/hip/openvino/aclgraph (CPU-only)
    let excluded = ["cuda", "rocm", "vulkan", "opencl", "sycl", "hip", "openvino", "aclgraph"];
    for asset in assets {
        if let Some(name) = asset["name"].as_str() {
            if name.contains(pattern) && name.ends_with(ext)
                && !excluded.iter().any(|ex| name.contains(ex))
            {
                if let Some(url) = asset["browser_download_url"].as_str() {
                    return Ok(url.to_string());
                }
            }
        }
    }

    Err(AppError::Validation(format!(
        "no llama-server archive found for platform '{}' in latest release", pattern
    )))
}

/// Download both the llama-server binary and the GGUF model.
/// Progress callback receives (downloaded_bytes, total_bytes) for the combined download.
pub fn download_model(
    app_data_dir: &PathBuf,
    progress_callback: impl Fn(u64, u64) + Send + 'static,
) -> AppResult<()> {
    let models_dir = get_models_dir(app_data_dir);
    let bin_dir = app_data_dir.join("bin");
    std::fs::create_dir_all(&models_dir).map_err(AppError::Io)?;
    std::fs::create_dir_all(&bin_dir).map_err(AppError::Io)?;

    let server_path = bin_dir.join("llama-server");
    let model_path = get_model_path(app_data_dir);

    // Estimate total size: ~50MB server archive + ~1GB model
    let total_estimated = MODEL_SIZE_APPROX + 50_000_000;
    let mut total_downloaded: u64 = 0;

    // Step 1: Download llama-server from GitHub release archive (if not already present)
    if !server_path.exists() {
        let archive_url = resolve_server_archive_url()?;
        eprintln!("[AI Search] Downloading llama-server archive from {}", archive_url);

        let response = ureq::get(&archive_url)
            .call()
            .map_err(|e| AppError::Validation(format!("failed to download llama-server: {e}")))?;

        // Download archive to a temp file
        let archive_path = bin_dir.join("llama-server-archive.tar.gz");
        {
            let mut file = std::fs::File::create(&archive_path).map_err(AppError::Io)?;
            let mut reader = response.into_reader();
            let mut buffer = [0u8; 65536];

            loop {
                let bytes_read = std::io::Read::read(&mut reader, &mut buffer).map_err(AppError::Io)?;
                if bytes_read == 0 { break; }
                std::io::Write::write_all(&mut file, &buffer[..bytes_read]).map_err(AppError::Io)?;
                total_downloaded += bytes_read as u64;
                progress_callback(total_downloaded, total_estimated);
            }
        }

        // Extract llama-server binary from the archive using system tar
        // Archive structure: llama-bNNNN/llama-server
        let extract_result = std::process::Command::new("tar")
            .args(["xzf", &archive_path.to_string_lossy(), "--strip-components=1",
                   "-C", &bin_dir.to_string_lossy(), "--include=*/llama-server"])
            .output()
            .map_err(|e| AppError::Validation(format!("failed to extract llama-server: {e}")))?;

        if !extract_result.status.success() {
            // macOS tar doesn't support --include, try wildcard approach
            let extract_result2 = std::process::Command::new("tar")
                .args(["xzf", &archive_path.to_string_lossy(), "--strip-components=1",
                       "-C", &bin_dir.to_string_lossy()])
                .output()
                .map_err(|e| AppError::Validation(format!("failed to extract archive: {e}")))?;

            if !extract_result2.status.success() {
                let stderr = String::from_utf8_lossy(&extract_result2.stderr);
                return Err(AppError::Validation(format!("tar extraction failed: {stderr}")));
            }
        }

        // Clean up: remove archive and any extra extracted files (keep only llama-server)
        let _ = std::fs::remove_file(&archive_path);
        // Clean up non-llama-server files that may have been extracted
        if let Ok(entries) = std::fs::read_dir(&bin_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str != "llama-server" {
                    let _ = std::fs::remove_file(entry.path());
                }
            }
        }

        if !server_path.exists() {
            return Err(AppError::Validation("llama-server binary not found in archive".to_string()));
        }

        // Make executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&server_path).map_err(AppError::Io)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&server_path, perms).map_err(AppError::Io)?;
        }

        eprintln!("[AI Search] llama-server extracted to {:?}", server_path);
    }

    // Step 2: Download GGUF model (if not already present)
    if !model_path.exists() {
        eprintln!("[AI Search] Downloading model from {}", MODEL_URL);
        let response = ureq::get(MODEL_URL)
            .call()
            .map_err(|e| AppError::Validation(format!("failed to download model: {e}")))?;

        let model_total = response.header("Content-Length")
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(MODEL_SIZE_APPROX);

        let actual_total = total_downloaded + model_total;

        let mut file = std::fs::File::create(&model_path).map_err(AppError::Io)?;
        let mut reader = response.into_reader();
        let mut buffer = [0u8; 65536];

        loop {
            let bytes_read = std::io::Read::read(&mut reader, &mut buffer).map_err(AppError::Io)?;
            if bytes_read == 0 { break; }
            std::io::Write::write_all(&mut file, &buffer[..bytes_read]).map_err(AppError::Io)?;
            total_downloaded += bytes_read as u64;
            progress_callback(total_downloaded, actual_total);
        }

        eprintln!("[AI Search] Model downloaded to {:?}", model_path);
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
    // Check in app's bin directory (downloaded by install flow)
    let app_binary = app_data_dir.join("bin").join("llama-server");
    if app_binary.exists() {
        return Ok(app_binary);
    }

    // Check system PATH
    if let Ok(path) = which::which("llama-server") {
        return Ok(path);
    }

    Err(AppError::Validation(
        "llama-server not found. Please install AI Search from Settings.".to_string()
    ))
}
