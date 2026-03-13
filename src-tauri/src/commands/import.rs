use std::fs;
use std::collections::HashMap;
use chrono::NaiveDate;
use encoding_rs::Encoding;
use sha2::{Digest, Sha256};
use tauri::State;
use uuid::Uuid;

use crate::db::DbState;
use crate::error::{AppError, AppResult};
use crate::helpers::timestamps::now_utc;
use crate::models::import::{
    ColumnMapping, CsvPreview, ImportBatch, ImportInput, ImportProfile, ImportResult,
    ImportRowError,
};

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Strip UTF-8 / UTF-16 BOM bytes from raw bytes.
fn strip_bom(bytes: &[u8]) -> &[u8] {
    if bytes.starts_with(b"\xEF\xBB\xBF") {
        return &bytes[3..]; // UTF-8 BOM
    }
    if bytes.starts_with(b"\xFF\xFE") || bytes.starts_with(b"\xFE\xFF") {
        return &bytes[2..]; // UTF-16 BOM
    }
    bytes
}

/// Decode raw bytes to String, trying UTF-8 first then encoding_rs detection.
fn decode_bytes(raw: &[u8]) -> String {
    // Fast path: valid UTF-8
    if let Ok(s) = std::str::from_utf8(raw) {
        return s.to_string();
    }
    // Use encoding_rs detector (windows-1252, latin-1, etc.)
    let (enc, _bom, _confident) = Encoding::for_bom(raw)
        .map(|(e, _)| (e, false, true))
        .unwrap_or_else(|| {
            // Default fallback: windows-1252 handles most Western bank exports
            (encoding_rs::WINDOWS_1252, false, false)
        });
    let (cow, _had_errors) = enc.decode_without_bom_handling(raw);
    cow.into_owned()
}

/// Auto-detect CSV delimiter by counting commas vs semicolons vs tabs in first line.
fn detect_delimiter(first_line: &str) -> u8 {
    let commas = first_line.matches(',').count();
    let semis = first_line.matches(';').count();
    let tabs = first_line.matches('\t').count();
    if tabs > commas && tabs > semis {
        return b'\t';
    }
    if semis > commas {
        return b';';
    }
    b','
}

/// SHA-256 hash exposed for testing.
#[cfg(test)]
pub fn row_hash_pub(account_id: &str, date: &str, amount: i64, payee: &str, description: &str) -> String {
    row_hash(account_id, date, amount, payee, description)
}

/// Parse an amount string in various formats → integer cents.
/// Handles: "$1,234.56", "(123.45)", "-123.45", "1.234,56" (European), "1234"
pub fn parse_amount(s: &str) -> Result<i64, String> {
    let s = s.trim();
    // Detect parentheses as negative
    let (negative, s) = if s.starts_with('(') && s.ends_with(')') {
        (true, &s[1..s.len() - 1])
    } else {
        (false, s)
    };

    // Detect leading minus before stripping symbols
    let has_leading_minus = s.trim_start().starts_with('-');

    // Strip currency symbols, spaces, and sign — keep digits, dot, comma
    let s: String = s
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.' || *c == ',')
        .collect();

    let was_negative = negative || has_leading_minus;

    // Detect European format: "1.234,56" → the last separator is comma
    let normalized = if s.contains(',') && s.contains('.') {
        // whichever comes last is the decimal separator
        let last_comma = s.rfind(',').unwrap_or(0);
        let last_dot = s.rfind('.').unwrap_or(0);
        if last_comma > last_dot {
            // European: remove dots (thousands), replace comma with dot
            s.replace('.', "").replace(',', ".")
        } else {
            // US: remove commas (thousands)
            s.replace(',', "")
        }
    } else if s.contains(',') {
        // Only commas — could be European decimal or US thousands
        // Heuristic: if exactly one comma and ≤2 digits after → decimal
        let parts: Vec<&str> = s.splitn(2, ',').collect();
        if parts.len() == 2 && parts[1].len() <= 2 {
            s.replace(',', ".")
        } else {
            s.replace(',', "")
        }
    } else {
        s.to_string()
    };

    let amount: f64 = normalized
        .parse()
        .map_err(|_| format!("Cannot parse amount: {}", s))?;

    let cents = (amount * 100.0).round() as i64;
    if was_negative || negative {
        Ok(-cents)
    } else {
        Ok(cents)
    }
}

/// SHA-256 hash of account_id + row fields for deduplication.
fn row_hash(account_id: &str, date: &str, amount: i64, payee: &str, description: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(account_id.as_bytes());
    hasher.update(b"|");
    hasher.update(date.as_bytes());
    hasher.update(b"|");
    hasher.update(amount.to_string().as_bytes());
    hasher.update(b"|");
    hasher.update(payee.as_bytes());
    hasher.update(b"|");
    hasher.update(description.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// SHA-256 of file bytes (for duplicate-file detection).
fn file_hash(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

// ── Commands ──────────────────────────────────────────────────────────────────

/// Read first 10 rows of a CSV file; return headers + preview rows + detected delimiter.
#[tauri::command]
pub fn preview_csv(file_path: String) -> AppResult<CsvPreview> {
    let raw = fs::read(&file_path).map_err(AppError::Io)?;
    let stripped = strip_bom(&raw);
    let text = decode_bytes(stripped);

    let first_line = text.lines().next().unwrap_or("");
    let delimiter = detect_delimiter(first_line);

    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .flexible(true)
        .from_reader(text.as_bytes());

    let headers: Vec<String> = rdr
        .headers()
        .map_err(|e| AppError::Csv(e))?
        .iter()
        .map(|h| h.trim().to_string())
        .collect();

    let mut rows: Vec<Vec<String>> = Vec::new();
    for result in rdr.records().take(10) {
        let record = result.map_err(|e| AppError::Csv(e))?;
        rows.push(record.iter().map(|f| f.trim().to_string()).collect());
    }

    Ok(CsvPreview {
        headers,
        rows,
        delimiter: delimiter as char,
    })
}

/// Full CSV import: parse, map columns, dedupe, batch-insert.
#[tauri::command]
pub fn import_csv(input: ImportInput, state: State<'_, DbState>) -> AppResult<ImportResult> {
    let raw = fs::read(&input.file_path).map_err(AppError::Io)?;
    let fhash = file_hash(&raw);
    let stripped = strip_bom(&raw);
    let text = decode_bytes(stripped);

    let first_line = text.lines().next().unwrap_or("");
    let delimiter = detect_delimiter(first_line);

    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .flexible(true)
        .from_reader(text.as_bytes());

    let csv_headers: Vec<String> = rdr
        .headers()
        .map_err(|e| AppError::Csv(e))?
        .iter()
        .map(|h| h.trim().to_string())
        .collect();

    // Build index: csv column name → position
    let header_index: HashMap<String, usize> = csv_headers
        .iter()
        .enumerate()
        .map(|(i, h)| (h.clone(), i))
        .collect();

    // Build field → column index map from user mappings
    let mut field_col: HashMap<String, usize> = HashMap::new();
    for m in &input.mappings {
        if let Some(&idx) = header_index.get(&m.csv_column) {
            field_col.insert(m.field.clone(), idx);
        }
    }

    let filename = std::path::Path::new(&input.file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("import.csv")
        .to_string();

    let batch_id = Uuid::new_v4().to_string();
    let now = now_utc();

    let mut imported = 0usize;
    let mut duplicates = 0usize;
    let mut errors: Vec<ImportRowError> = Vec::new();

    // Collect all valid rows first, then insert in one transaction
    struct Row {
        id: String,
        date: String,
        amount: i64,
        payee: String,
        description: String,
        hash: String,
    }

    let mut valid_rows: Vec<Row> = Vec::new();

    for (row_idx, result) in rdr.records().enumerate() {
        let record = match result {
            Ok(r) => r,
            Err(e) => {
                errors.push(ImportRowError {
                    row: row_idx + 2,
                    message: e.to_string(),
                });
                continue;
            }
        };

        let fields: Vec<&str> = record.iter().collect();

        // Skip blank rows
        if fields.iter().all(|f| f.trim().is_empty()) {
            continue;
        }

        let get = |field: &str| -> &str {
            field_col
                .get(field)
                .and_then(|&i| fields.get(i).copied())
                .unwrap_or("")
                .trim()
        };

        // --- Date ---
        let date_str = get("date");
        if date_str.is_empty() {
            errors.push(ImportRowError { row: row_idx + 2, message: "Missing date".into() });
            continue;
        }
        let date = match NaiveDate::parse_from_str(date_str, &input.date_format) {
            Ok(d) => d.format("%Y-%m-%d").to_string(),
            Err(_) => {
                errors.push(ImportRowError {
                    row: row_idx + 2,
                    message: format!("Cannot parse date '{}' with format '{}'", date_str, input.date_format),
                });
                continue;
            }
        };

        // --- Amount ---
        let amount: i64 = match input.amount_format.as_str() {
            "debit_credit" => {
                let debit_str = get("debit");
                let credit_str = get("credit");
                let debit = if debit_str.is_empty() { 0 } else {
                    match parse_amount(debit_str) {
                        Ok(v) => v.abs(),
                        Err(e) => { errors.push(ImportRowError { row: row_idx + 2, message: e }); continue; }
                    }
                };
                let credit = if credit_str.is_empty() { 0 } else {
                    match parse_amount(credit_str) {
                        Ok(v) => v.abs(),
                        Err(e) => { errors.push(ImportRowError { row: row_idx + 2, message: e }); continue; }
                    }
                };
                credit as i64 - debit as i64
            }
            _ => {
                let amt_str = get("amount");
                if amt_str.is_empty() {
                    errors.push(ImportRowError { row: row_idx + 2, message: "Missing amount".into() });
                    continue;
                }
                match parse_amount(amt_str) {
                    Ok(v) => v,
                    Err(e) => { errors.push(ImportRowError { row: row_idx + 2, message: e }); continue; }
                }
            }
        };

        let payee = get("payee").to_string();
        let description = get("description").to_string();
        let hash = row_hash(&input.account_id, &date, amount, &payee, &description);

        valid_rows.push(Row {
            id: Uuid::new_v4().to_string(),
            date,
            amount,
            payee,
            description,
            hash,
        });
    }

    // Insert all valid rows in a single DB transaction
    {
        let conn = state.write_conn.lock().unwrap();
        conn.execute_batch("BEGIN")?;

        // Create batch record
        conn.execute(
            "INSERT INTO import_batches (id, filename, file_hash, account_id, profile_id, row_count, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6)",
            rusqlite::params![batch_id, filename, fhash, input.account_id, input.profile_id, now],
        )?;

        for row in &valid_rows {
            let kind = if row.amount >= 0 { "income" } else { "expense" };
            let result = conn.execute(
                "INSERT OR IGNORE INTO transactions
                 (id, account_id, date, amount, kind, payee, description,
                  is_cleared, import_batch_id, import_row_hash, created_at, updated_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,1,?8,?9,?10,?10)",
                rusqlite::params![
                    row.id, input.account_id, row.date, row.amount, kind,
                    row.payee, row.description, batch_id, row.hash, now
                ],
            )?;
            if result == 0 {
                duplicates += 1;
            } else {
                imported += 1;
            }
        }

        // Update batch row_count
        conn.execute(
            "UPDATE import_batches SET row_count = ?1 WHERE id = ?2",
            rusqlite::params![imported as i64, batch_id],
        )?;

        conn.execute_batch("COMMIT")?;
    }

    Ok(ImportResult {
        batch_id,
        imported,
        duplicates,
        errors,
    })
}

/// Delete all transactions in a batch and the batch record itself.
#[tauri::command]
pub fn rollback_import(batch_id: String, state: State<'_, DbState>) -> AppResult<()> {
    let conn = state.write_conn.lock().unwrap();
    conn.execute_batch("BEGIN")?;
    conn.execute(
        "DELETE FROM transactions WHERE import_batch_id = ?1",
        rusqlite::params![batch_id],
    )?;
    conn.execute(
        "DELETE FROM import_batches WHERE id = ?1",
        rusqlite::params![batch_id],
    )?;
    conn.execute_batch("COMMIT")?;
    Ok(())
}

/// List all import batches (most recent first).
#[tauri::command]
pub fn get_import_batches(state: State<'_, DbState>) -> AppResult<Vec<ImportBatch>> {
    let conn = state.read_conn.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, filename, file_hash, account_id, profile_id, row_count, created_at
         FROM import_batches ORDER BY created_at DESC",
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok(ImportBatch {
                id: row.get(0)?,
                filename: row.get(1)?,
                file_hash: row.get(2)?,
                account_id: row.get(3)?,
                profile_id: row.get(4)?,
                row_count: row.get(5)?,
                created_at: row.get(6)?,
            })
        })?
        .collect::<Result<_, _>>()?;
    Ok(rows)
}

/// Return all saved import profiles.
#[tauri::command]
pub fn get_import_profiles(state: State<'_, DbState>) -> AppResult<Vec<ImportProfile>> {
    let conn = state.read_conn.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, name, mapping, created_at FROM import_profiles ORDER BY name ASC",
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok(ImportProfile {
                id: row.get(0)?,
                name: row.get(1)?,
                mapping: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?
        .collect::<Result<_, _>>()?;
    Ok(rows)
}

/// Save (upsert by name) a column mapping profile.
#[tauri::command]
pub fn save_import_profile(
    name: String,
    mappings: Vec<ColumnMapping>,
    state: State<'_, DbState>,
) -> AppResult<ImportProfile> {
    let conn = state.write_conn.lock().unwrap();
    let mapping_json = serde_json::to_string(&mappings)
        .map_err(|e| AppError::Validation(e.to_string()))?;
    let now = now_utc();
    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO import_profiles (id, name, mapping, created_at)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(name) DO UPDATE SET mapping = excluded.mapping",
        rusqlite::params![id, name, mapping_json, now],
    )?;
    // Fetch back (id may differ if updated existing row)
    let profile = conn.query_row(
        "SELECT id, name, mapping, created_at FROM import_profiles WHERE name = ?1",
        rusqlite::params![name],
        |row| Ok(ImportProfile {
            id: row.get(0)?,
            name: row.get(1)?,
            mapping: row.get(2)?,
            created_at: row.get(3)?,
        }),
    )?;
    Ok(profile)
}

/// Delete a saved import profile by id.
#[tauri::command]
pub fn delete_import_profile(id: String, state: State<'_, DbState>) -> AppResult<()> {
    let conn = state.write_conn.lock().unwrap();
    let rows = conn.execute(
        "DELETE FROM import_profiles WHERE id = ?1",
        rusqlite::params![id],
    )?;
    if rows == 0 {
        return Err(AppError::NotFound(format!("Import profile {}", id)));
    }
    Ok(())
}
