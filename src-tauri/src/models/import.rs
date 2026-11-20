use serde::{Deserialize, Serialize};

/// One CSV column -> one transaction field mapping entry.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ColumnMapping {
    /// CSV header name
    pub csv_column: String,
    /// Target field: "date" | "payee" | "description" | "amount" | "debit" | "credit"
    pub field: String,
}

/// Saved import profile (reusable column mapping).
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportProfile {
    pub id: String,
    pub name: String,
    /// JSON-encoded Vec<ColumnMapping>
    pub mapping: String,
    pub created_at: String,
}

/// Input from the frontend to kick off a full import.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportInput {
    pub file_path: String,
    pub account_id: String,
    pub mappings: Vec<ColumnMapping>,
    /// strftime-compatible format string, e.g. "%m/%d/%Y"
    pub date_format: String,
    /// "single" | "debit_credit"
    pub amount_format: String,
    pub profile_id: Option<String>,
}

/// Per-row error during import.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportRowError {
    pub row: usize,
    pub message: String,
}

/// Result returned after a full import.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub batch_id: String,
    pub imported: usize,
    pub duplicates: usize,
    pub errors: Vec<ImportRowError>,
}

/// Preview returned by preview_csv (headers + first N rows).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CsvPreview {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub delimiter: char,
}
