use std::path::PathBuf;
use rusqlite::{Connection, OptionalExtension};
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
/// Expected SHA-256 checksum for the model file (verified after download)
pub const MODEL_SHA256: &str = "cc324af070c2ecbfd324a30884d2f951a7ff756aba85cb811a6ec436933bb046";

/// Tables whose live `CREATE TABLE` text goes into the system prompt
const PROMPT_TABLES: [&str; 10] = [
    "transactions", "categories", "category_groups", "accounts", "budgets",
    "budget_items", "recurring_transactions", "tags", "transaction_tags",
    "transaction_splits",
];

/// Build the system prompt from the live schema in `sqlite_master`
pub fn build_system_prompt(conn: &Connection) -> AppResult<String> {
    let mut stmt = conn.prepare("SELECT sql FROM sqlite_master WHERE type = 'table' AND name = ?1")?;
    let mut schema = String::new();
    for table in PROMPT_TABLES {
        let sql: Option<String> = stmt.query_row([table], |row| row.get(0)).optional()?;
        if let Some(sql) = sql {
            schema.push_str(sql.trim());
            schema.push_str(";\n\n");
        }
    }

    Ok(format!(
        "You are a SQL assistant for a personal finance SQLite database.\n\n\
Schema:\n\n{schema}\
Rules:\n\
- Amounts are integer cents. Income and expense amounts are positive; the outgoing leg of a transfer is negative. Do not divide by 100.\n\
- transactions.kind is one of income, expense, transfer.\n\
- Dates are TEXT in YYYY-MM-DD format.\n\
- Use the SQLite dialect.\n\
- Answer with one SELECT statement and nothing else: no explanation, no markdown, no comments."
    ))
}

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
    let model_installed = model_path.exists();
    let size = if model_installed {
        std::fs::metadata(&model_path).ok().map(|m| m.len())
    } else {
        None
    };

    ModelStatus {
        installed: model_installed,
        model_name: MODEL_NAME.to_string(),
        model_path: if model_installed { Some(model_path.to_string_lossy().to_string()) } else { None },
        model_size_bytes: size,
        ready: model_installed,
    }
}

/// Download the GGUF model (if not already present).
/// Progress callback receives (downloaded_bytes, total_bytes).
pub fn download_model(
    app_data_dir: &PathBuf,
    progress_callback: impl Fn(u64, u64) + Send + 'static,
) -> AppResult<()> {
    let models_dir = get_models_dir(app_data_dir);
    std::fs::create_dir_all(&models_dir).map_err(AppError::Io)?;

    let model_path = get_model_path(app_data_dir);

    if !model_path.exists() {
        eprintln!("[AI Search] Downloading model from {}", MODEL_URL);
        let response = ureq::get(MODEL_URL)
            .call()
            .map_err(|e| AppError::Validation(format!("failed to download model: {e}")))?;

        let model_total = response.header("Content-Length")
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(MODEL_SIZE_APPROX);

        let mut total_downloaded: u64 = 0;
        let mut file = std::fs::File::create(&model_path).map_err(AppError::Io)?;
        let mut reader = response.into_reader();
        let mut buffer = [0u8; 65536];

        loop {
            let bytes_read = std::io::Read::read(&mut reader, &mut buffer).map_err(AppError::Io)?;
            if bytes_read == 0 { break; }
            std::io::Write::write_all(&mut file, &buffer[..bytes_read]).map_err(AppError::Io)?;
            total_downloaded += bytes_read as u64;
            progress_callback(total_downloaded, model_total);
        }

        eprintln!("[AI Search] Model downloaded to {:?}", model_path);

        // Verify SHA-256 checksum of downloaded model
        eprintln!("[AI Search] Verifying model checksum...");
        if let Err(e) = verify_model_checksum(&model_path) {
            let _ = std::fs::remove_file(&model_path);
            return Err(e);
        }
        eprintln!("[AI Search] Model checksum verified");
    }

    Ok(())
}

/// Verify the SHA-256 checksum of a downloaded model file.
fn verify_model_checksum(path: &std::path::Path) -> AppResult<()> {
    use sha2::{Sha256, Digest};

    let mut file = std::fs::File::open(path).map_err(AppError::Io)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 65536];
    loop {
        let n = std::io::Read::read(&mut file, &mut buffer).map_err(AppError::Io)?;
        if n == 0 { break; }
        hasher.update(&buffer[..n]);
    }
    let hash = format!("{:x}", hasher.finalize());

    if hash != MODEL_SHA256 {
        return Err(AppError::Validation(format!(
            "Model checksum mismatch: expected {}, got {}. The download may be corrupted.",
            MODEL_SHA256, hash
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_contains_live_columns() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run_migrations(&conn).unwrap();
        let prompt = build_system_prompt(&conn).unwrap();
        assert!(prompt.contains("is_split"));
        assert!(prompt.contains("transfer_group_id"));
        for table in PROMPT_TABLES {
            assert!(prompt.contains(&format!("CREATE TABLE {table}"))
                || prompt.contains(&format!("CREATE TABLE \"{table}\"")), "missing {table}");
        }
    }
}
