use std::fs;
use std::collections::HashMap;
use chrono::{Datelike, NaiveDate};
use encoding_rs::Encoding;
use sha2::{Digest, Sha256};
use tauri::State;
use uuid::Uuid;

use crate::db::DbState;
use crate::error::{AppError, AppResult};
use crate::helpers::timestamps::now_utc;
use crate::models::import::{
    ColumnMapping, CsvPreview, ImportInput, ImportProfile, ImportResult,
    ImportRowError,
};

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Decode raw bytes to String, checking BOM first then UTF-8 then Windows-1252.
fn decode_bytes(raw: &[u8]) -> String {
    if let Some((enc, bom_len)) = Encoding::for_bom(raw) {
        let (cow, _) = enc.decode_without_bom_handling(&raw[bom_len..]);
        return cow.into_owned();
    }
    if let Ok(s) = std::str::from_utf8(raw) {
        return s.to_string();
    }
    let (cow, _) = encoding_rs::WINDOWS_1252.decode_without_bom_handling(raw);
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
    row_hash(account_id, date, amount, payee, description, 0)
}

fn strip_currency(s: &str) -> (&str, bool) {
    const CURRENCIES: [&str; 7] = ["USD", "EUR", "GBP", "$", "€", "£", "¥"];
    for cur in CURRENCIES {
        if let Some(rest) = s.strip_prefix(cur) {
            return (rest.trim(), true);
        }
        if let Some(rest) = s.strip_suffix(cur) {
            return (rest.trim(), true);
        }
    }
    (s, false)
}

fn strip_sign(s: &str) -> (&str, bool, bool) {
    if s.starts_with('(') && s.ends_with(')') {
        return (s[1..s.len() - 1].trim(), true, true);
    }
    if let Some(rest) = s.strip_prefix('-') {
        return (rest.trim(), true, true);
    }
    if let Some(rest) = s.strip_suffix('-') {
        return (rest.trim(), true, true);
    }
    if let Some(rest) = s.strip_prefix('+') {
        return (rest.trim(), false, true);
    }
    (s, false, false)
}

/// Parse an amount string in various formats → integer cents.
/// Handles: "$1,234.56", "(123.45)", "-123.45", "1.234,56" (European), "1234"
pub fn parse_amount(raw: &str) -> Result<i64, String> {
    let mapped = raw.replace('\u{2212}', "-");
    let s = mapped.trim();

    let (s, cur_stripped) = strip_currency(s);
    let (s, mut negative, sign_stripped) = strip_sign(s);
    let s = if !cur_stripped {
        let (s, _) = strip_currency(s);
        s
    } else {
        s
    };
    let s = if !sign_stripped {
        let (s, neg2, _) = strip_sign(s);
        negative = negative || neg2;
        s
    } else {
        s
    };

    if s.is_empty()
        || !s.chars().any(|c| c.is_ascii_digit())
        || !s.chars().all(|c| c.is_ascii_digit() || c == '.' || c == ',')
    {
        return Err(format!("Cannot parse amount: {}", raw));
    }

    let normalized = if s.contains(',') && s.contains('.') {
        let last_comma = s.rfind(',').unwrap_or(0);
        let last_dot = s.rfind('.').unwrap_or(0);
        if last_comma > last_dot {
            // European: dots are thousands separators, single trailing comma is decimal
            let comma_parts: Vec<&str> = s.split(',').collect();
            if comma_parts.len() != 2 || !(1..=2).contains(&comma_parts[1].len()) {
                return Err(format!("Cannot parse amount: {}", raw));
            }
            let dot_parts: Vec<&str> = comma_parts[0].split('.').collect();
            if dot_parts.len() < 2
                || !(1..=3).contains(&dot_parts[0].len())
                || !dot_parts[1..].iter().all(|p| p.len() == 3)
            {
                return Err(format!("Cannot parse amount: {}", raw));
            }
            format!("{}.{}", dot_parts.join(""), comma_parts[1])
        } else {
            // US: commas are thousands separators, single trailing dot is decimal
            let dot_parts: Vec<&str> = s.split('.').collect();
            if dot_parts.len() != 2 || !(1..=2).contains(&dot_parts[1].len()) {
                return Err(format!("Cannot parse amount: {}", raw));
            }
            let comma_parts: Vec<&str> = dot_parts[0].split(',').collect();
            if comma_parts.len() < 2
                || !(1..=3).contains(&comma_parts[0].len())
                || !comma_parts[1..].iter().all(|p| p.len() == 3)
            {
                return Err(format!("Cannot parse amount: {}", raw));
            }
            format!("{}.{}", comma_parts.join(""), dot_parts[1])
        }
    } else if s.contains(',') {
        let parts: Vec<&str> = s.split(',').collect();
        // A leading "0" or empty integer part is always a decimal ("0,125", ",50").
        if parts.len() == 2
            && !parts[1].is_empty()
            && ((1..=2).contains(&parts[1].len()) || parts[0].is_empty() || parts[0] == "0")
        {
            format!("{}.{}", parts[0], parts[1])
        } else if parts.len() >= 2
            && (1..=3).contains(&parts[0].len())
            && parts[1..].iter().all(|p| p.len() == 3)
        {
            parts.join("")
        } else {
            return Err(format!("Cannot parse amount: {}", raw));
        }
    } else if s.contains('.') {
        let parts: Vec<&str> = s.split('.').collect();
        // A leading "0" or empty integer part is always a decimal ("0.125", ".50").
        if parts.len() == 2
            && !parts[1].is_empty()
            && ((1..=2).contains(&parts[1].len()) || parts[0].is_empty() || parts[0] == "0")
        {
            format!("0{}", s)
        } else if parts.len() >= 2
            && (1..=3).contains(&parts[0].len())
            && parts[1..].iter().all(|p| p.len() == 3)
        {
            parts.join("")
        } else {
            return Err(format!("Cannot parse amount: {}", raw));
        }
    } else {
        s.to_string()
    };

    let amount: f64 = normalized
        .parse()
        .map_err(|_| format!("Cannot parse amount: {}", raw))?;

    let cents = (amount * 100.0).round() as i64;
    if negative {
        Ok(-cents)
    } else {
        Ok(cents)
    }
}

/// SHA-256 hash of account_id + length-prefixed row fields + per-file occurrence index for deduplication.
fn row_hash(
    account_id: &str,
    date: &str,
    amount: i64,
    payee: &str,
    description: &str,
    occurrence: usize,
) -> String {
    let mut hasher = Sha256::new();
    let amount_str = amount.to_string();
    let occ_str = occurrence.to_string();
    for field in [
        account_id,
        date,
        amount_str.as_str(),
        payee,
        description,
        occ_str.as_str(),
    ] {
        hasher.update(&(field.len() as u64).to_le_bytes());
        hasher.update(field.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

/// SHA-256 of file bytes (for duplicate-file detection).
fn file_hash(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn swapped_day_month_format(fmt: &str) -> Option<&'static str> {
    match fmt {
        "%m/%d/%Y" => Some("%d/%m/%Y"),
        "%d/%m/%Y" => Some("%m/%d/%Y"),
        "%m-%d-%Y" => Some("%d-%m-%Y"),
        "%d-%m-%Y" => Some("%m-%d-%Y"),
        _ => None,
    }
}

fn parse_date_checked(date_str: &str, fmt: &str) -> Result<NaiveDate, String> {
    let d = NaiveDate::parse_from_str(date_str, fmt)
        .map_err(|_| format!("Cannot parse date '{}' with format '{}'", date_str, fmt))?;
    if d.year() < 1900 || d.year() > 2100 {
        return Err(format!(
            "Date '{}' has year {} with format '{}': 2-digit vs 4-digit year mismatch",
            date_str,
            d.year(),
            fmt
        ));
    }
    Ok(d)
}

// ── Commands ──────────────────────────────────────────────────────────────────

/// Read first 10 rows of a CSV file; return headers + preview rows + detected delimiter.
#[tauri::command]
pub fn preview_csv(file_path: String) -> AppResult<CsvPreview> {
    let raw = fs::read(&file_path).map_err(AppError::Io)?;
    let text = decode_bytes(&raw);

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
    let conn = state.write_conn.lock().unwrap();
    import_csv_inner(&conn, input)
}

pub(crate) fn import_csv_inner(conn: &rusqlite::Connection, input: ImportInput) -> AppResult<ImportResult> {
    let raw = fs::read(&input.file_path).map_err(AppError::Io)?;
    let fhash = file_hash(&raw);
    let text = decode_bytes(&raw);

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
    let mut imported_ids: Vec<String> = Vec::new();

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
    let mut seen_counts: HashMap<(String, i64, String, String), usize> = HashMap::new();
    let swapped_fmt = swapped_day_month_format(&input.date_format);
    let mut any_chosen_date_failed = false;
    let mut all_swapped_dates_ok = true;

    for (row_idx, result) in rdr.records().enumerate() {
        let record = match result {
            Ok(r) => r,
            Err(e) => {
                all_swapped_dates_ok = false;
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
        if let Some(swapped) = swapped_fmt {
            if date_str.is_empty() || parse_date_checked(date_str, swapped).is_err() {
                all_swapped_dates_ok = false;
            }
        }
        if date_str.is_empty() {
            any_chosen_date_failed = true;
            errors.push(ImportRowError { row: row_idx + 2, message: "Missing date".into() });
            continue;
        }
        let date = match parse_date_checked(date_str, &input.date_format) {
            Ok(d) => d.format("%Y-%m-%d").to_string(),
            Err(msg) => {
                any_chosen_date_failed = true;
                errors.push(ImportRowError {
                    row: row_idx + 2,
                    message: msg,
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
        let count = seen_counts
            .entry((date.clone(), amount, payee.clone(), description.clone()))
            .or_insert(0);
        let occurrence = *count;
        *count += 1;
        let hash = row_hash(
            &input.account_id,
            &date,
            amount,
            &payee,
            &description,
            occurrence,
        );

        valid_rows.push(Row {
            id: Uuid::new_v4().to_string(),
            date,
            amount,
            payee,
            description,
            hash,
        });
    }

    if let Some(swapped) = swapped_fmt {
        if any_chosen_date_failed && all_swapped_dates_ok {
            return Err(AppError::Validation(format!(
                "Date format mismatch: dates match '{}' instead of '{}' (day and month swapped)",
                swapped, input.date_format
            )));
        }
    }

    // Insert all valid rows in a single DB transaction
    {
        let tx = conn.unchecked_transaction()?;
        tx.execute_batch("PRAGMA defer_foreign_keys = ON;")?;

        for row in &valid_rows {
            // Amounts are stored positive; kind carries the direction.
            let kind = if row.amount >= 0 { "income" } else { "expense" };
            let result = tx.execute(
                "INSERT OR IGNORE INTO transactions
                 (id, account_id, date, amount, kind, payee, description,
                  is_cleared, import_batch_id, import_row_hash, created_at, updated_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,1,?8,?9,?10,?10)",
                rusqlite::params![
                    row.id, input.account_id, row.date, row.amount.abs(), kind,
                    row.payee, row.description, batch_id, row.hash, now
                ],
            )?;
            if result == 0 {
                duplicates += 1;
            } else {
                imported += 1;
                imported_ids.push(row.id.clone());
            }
        }

        if imported > 0 {
            tx.execute(
                "INSERT INTO import_batches (id, filename, file_hash, account_id, profile_id, row_count, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![
                    batch_id,
                    filename,
                    fhash,
                    input.account_id,
                    input.profile_id,
                    imported as i64,
                    now
                ],
            )?;
        }

        tx.commit()?;
    }

    // Auto-categorize newly imported uncategorized transactions
    if !imported_ids.is_empty() {
        let _ = super::rules::apply_rules_to_transactions(conn, &imported_ids);
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
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "DELETE FROM transactions WHERE import_batch_id = ?1",
        rusqlite::params![batch_id],
    )?;
    tx.execute(
        "DELETE FROM import_batches WHERE id = ?1",
        rusqlite::params![batch_id],
    )?;
    tx.commit()?;
    Ok(())
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
