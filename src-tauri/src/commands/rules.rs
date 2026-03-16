use regex::Regex;
use rusqlite::params;
use tauri::State;
use uuid::Uuid;

use crate::db::DbState;
use crate::error::{AppError, AppResult};
use crate::helpers::timestamps::now_utc;
use crate::models::rule::{CategorizationRule, CreateRuleInput, RuleTestResult, UpdateRuleInput};

fn row_to_rule(row: &rusqlite::Row) -> rusqlite::Result<CategorizationRule> {
    Ok(CategorizationRule {
        id: row.get(0)?,
        name: row.get(1)?,
        match_field: row.get(2)?,
        match_type: row.get(3)?,
        pattern: row.get(4)?,
        category_id: row.get(5)?,
        priority: row.get(6)?,
        is_active: row.get::<_, i64>(7)? != 0,
        apply_count: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

const RULE_COLS: &str = "id, name, match_field, match_type, pattern, category_id, priority, is_active, apply_count, created_at, updated_at";

fn validate_match_field(field: &str) -> AppResult<()> {
    match field {
        "payee" | "description" | "amount" => Ok(()),
        _ => Err(AppError::Validation(format!(
            "Invalid match_field '{}'. Must be payee, description, or amount",
            field
        ))),
    }
}

fn validate_match_type(mt: &str) -> AppResult<()> {
    match mt {
        "contains" | "starts_with" | "ends_with" | "exact" | "regex" => Ok(()),
        _ => Err(AppError::Validation(format!(
            "Invalid match_type '{}'. Must be contains, starts_with, ends_with, exact, or regex",
            mt
        ))),
    }
}

fn validate_regex_pattern(pattern: &str) -> AppResult<()> {
    Regex::new(pattern).map_err(|e| {
        AppError::Validation(format!("Invalid regex pattern '{}': {}", pattern, e))
    })?;
    Ok(())
}

/// A rule with its regex pre-compiled (if applicable) for efficient batch matching.
struct CompiledRule {
    rule: CategorizationRule,
    compiled_regex: Option<Regex>,
    pattern_lower: String,
}

impl CompiledRule {
    fn from_rules(rules: &[CategorizationRule]) -> Vec<CompiledRule> {
        rules
            .iter()
            .map(|r| {
                let compiled_regex = if r.match_type == "regex" {
                    Regex::new(&format!("(?i){}", r.pattern)).ok()
                } else {
                    None
                };
                CompiledRule {
                    rule: r.clone(),
                    compiled_regex,
                    pattern_lower: r.pattern.to_lowercase(),
                }
            })
            .collect()
    }

    fn matches(&self, value: &str) -> bool {
        let value_lower = value.to_lowercase();
        match self.rule.match_type.as_str() {
            "contains" => value_lower.contains(&self.pattern_lower),
            "starts_with" => value_lower.starts_with(&self.pattern_lower),
            "ends_with" => value_lower.ends_with(&self.pattern_lower),
            "exact" => value_lower == self.pattern_lower,
            "regex" => self.compiled_regex.as_ref().map_or(false, |re| re.is_match(value)),
            _ => false,
        }
    }
}

fn matches_value(value: &str, match_type: &str, pattern: &str) -> bool {
    let value_lower = value.to_lowercase();
    let pattern_lower = pattern.to_lowercase();
    match match_type {
        "contains" => value_lower.contains(&pattern_lower),
        "starts_with" => value_lower.starts_with(&pattern_lower),
        "ends_with" => value_lower.ends_with(&pattern_lower),
        "exact" => value_lower == pattern_lower,
        "regex" => {
            let pattern_ci = format!("(?i){}", pattern);
            Regex::new(&pattern_ci).map_or(false, |re| re.is_match(value))
        }
        _ => false,
    }
}

fn get_field_value(payee: &str, description: &str, amount: i64, match_field: &str) -> String {
    match match_field {
        "payee" => payee.to_string(),
        "description" => description.to_string(),
        "amount" => amount.abs().to_string(),
        _ => String::new(),
    }
}

#[tauri::command]
pub fn create_rule(
    input: CreateRuleInput,
    state: State<DbState>,
) -> AppResult<CategorizationRule> {
    validate_match_field(&input.match_field)?;
    validate_match_type(&input.match_type)?;
    if input.match_type == "regex" {
        validate_regex_pattern(&input.pattern)?;
    }
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("Rule name cannot be empty".into()));
    }
    if input.pattern.trim().is_empty() {
        return Err(AppError::Validation("Pattern cannot be empty".into()));
    }

    let conn = state.write_conn.lock().unwrap();
    let id = Uuid::new_v4().to_string();
    let now = now_utc();

    // Auto-assign priority: max existing + 1
    let max_priority: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(priority), -1) FROM categorization_rules",
            [],
            |row| row.get(0),
        )
        .unwrap_or(-1);
    let priority = max_priority + 1;

    conn.execute(
        "INSERT INTO categorization_rules (id, name, match_field, match_type, pattern, category_id, priority, is_active, apply_count, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1, 0, ?8, ?9)",
        params![
            id,
            input.name.trim(),
            input.match_field,
            input.match_type,
            input.pattern,
            input.category_id,
            priority,
            now,
            now,
        ],
    )?;

    Ok(CategorizationRule {
        id,
        name: input.name.trim().to_string(),
        match_field: input.match_field,
        match_type: input.match_type,
        pattern: input.pattern,
        category_id: input.category_id,
        priority,
        is_active: true,
        apply_count: 0,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub fn get_all_rules(state: State<DbState>) -> AppResult<Vec<CategorizationRule>> {
    let conn = state.read_conn.lock().unwrap();
    let mut stmt = conn.prepare(&format!(
        "SELECT {} FROM categorization_rules ORDER BY priority ASC",
        RULE_COLS
    ))?;
    let rules = stmt
        .query_map([], |row| row_to_rule(row))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rules)
}

#[tauri::command]
pub fn update_rule(
    id: String,
    input: UpdateRuleInput,
    state: State<DbState>,
) -> AppResult<CategorizationRule> {
    if let Some(ref f) = input.match_field {
        validate_match_field(f)?;
    }
    if let Some(ref mt) = input.match_type {
        validate_match_type(mt)?;
    }

    let conn = state.write_conn.lock().unwrap();
    let now = now_utc();

    // Fetch existing
    let existing = conn.query_row(
        &format!("SELECT {} FROM categorization_rules WHERE id = ?1", RULE_COLS),
        params![id],
        |row| row_to_rule(row),
    ).map_err(|_| AppError::NotFound(format!("Rule {} not found", id)))?;

    let name = input.name.unwrap_or(existing.name);
    let match_field = input.match_field.unwrap_or(existing.match_field);
    let match_type = input.match_type.unwrap_or(existing.match_type);
    let pattern = input.pattern.unwrap_or(existing.pattern);
    let category_id = input.category_id.unwrap_or(existing.category_id);
    let is_active = input.is_active.unwrap_or(existing.is_active);

    if match_type == "regex" {
        validate_regex_pattern(&pattern)?;
    }

    conn.execute(
        "UPDATE categorization_rules SET name=?1, match_field=?2, match_type=?3, pattern=?4, category_id=?5, is_active=?6, updated_at=?7 WHERE id=?8",
        params![name, match_field, match_type, pattern, category_id, is_active as i64, now, id],
    )?;

    Ok(CategorizationRule {
        id,
        name,
        match_field,
        match_type,
        pattern,
        category_id,
        priority: existing.priority,
        is_active,
        apply_count: existing.apply_count,
        created_at: existing.created_at,
        updated_at: now,
    })
}

#[tauri::command]
pub fn delete_rule(id: String, state: State<DbState>) -> AppResult<()> {
    let conn = state.write_conn.lock().unwrap();
    let affected = conn.execute("DELETE FROM categorization_rules WHERE id = ?1", params![id])?;
    if affected == 0 {
        return Err(AppError::NotFound(format!("Rule {} not found", id)));
    }
    Ok(())
}

#[tauri::command]
pub fn reorder_rules(ids: Vec<String>, state: State<DbState>) -> AppResult<()> {
    let conn = state.write_conn.lock().unwrap();
    let now = now_utc();
    conn.execute_batch("BEGIN")?;
    for (i, id) in ids.iter().enumerate() {
        conn.execute(
            "UPDATE categorization_rules SET priority = ?1, updated_at = ?2 WHERE id = ?3",
            params![i as i64, now, id],
        )?;
    }
    conn.execute_batch("COMMIT")?;
    Ok(())
}

/// Given a transaction's payee/description/amount, iterate active rules by priority
/// and return the first matching category_id (or None).
pub fn match_transaction_inner(
    conn: &rusqlite::Connection,
    payee: &str,
    description: &str,
    amount: i64,
) -> AppResult<Option<String>> {
    let rules = load_active_rules(conn)?;
    let compiled = CompiledRule::from_rules(&rules);

    for cr in &compiled {
        let value = get_field_value(payee, description, amount, &cr.rule.match_field);
        if cr.matches(&value) {
            return Ok(Some(cr.rule.category_id.clone()));
        }
    }
    Ok(None)
}

#[tauri::command]
pub fn match_transaction(
    payee: String,
    description: String,
    amount: i64,
    state: State<DbState>,
) -> AppResult<Option<String>> {
    let conn = state.read_conn.lock().unwrap();
    match_transaction_inner(&conn, &payee, &description, amount)
}

/// Shared helper: match transactions against compiled rules, update categories and apply counts.
/// Wraps all updates in a transaction for atomicity.
fn apply_rules_to_rows(
    conn: &rusqlite::Connection,
    txns: &[(String, String, String, i64)],
    compiled_rules: &[CompiledRule],
) -> AppResult<i64> {
    if txns.is_empty() || compiled_rules.is_empty() {
        return Ok(0);
    }

    let now = now_utc();
    let mut updated = 0i64;
    let mut rule_apply_counts: std::collections::HashMap<String, i64> = std::collections::HashMap::new();

    conn.execute_batch("BEGIN")?;

    for (tx_id, payee, description, amount) in txns {
        for cr in compiled_rules {
            let value = get_field_value(payee, description, *amount, &cr.rule.match_field);
            if cr.matches(&value) {
                conn.execute(
                    "UPDATE transactions SET category_id = ?1, updated_at = ?2 WHERE id = ?3",
                    params![cr.rule.category_id, now, tx_id],
                )?;
                *rule_apply_counts.entry(cr.rule.id.clone()).or_insert(0) += 1;
                updated += 1;
                break;
            }
        }
    }

    // Batch update apply_count for each rule
    for (rule_id, count) in &rule_apply_counts {
        conn.execute(
            "UPDATE categorization_rules SET apply_count = apply_count + ?1, updated_at = ?2 WHERE id = ?3",
            params![count, now, rule_id],
        )?;
    }

    conn.execute_batch("COMMIT")?;
    Ok(updated)
}

fn load_active_rules(conn: &rusqlite::Connection) -> AppResult<Vec<CategorizationRule>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {} FROM categorization_rules WHERE is_active = 1 ORDER BY priority ASC",
        RULE_COLS
    ))?;
    let rules = stmt
        .query_map([], |row| row_to_rule(row))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rules)
}

#[tauri::command]
pub fn apply_rules_to_uncategorized(state: State<DbState>) -> AppResult<i64> {
    let conn = state.write_conn.lock().unwrap();

    let rules = load_active_rules(&conn)?;
    if rules.is_empty() {
        return Ok(0);
    }
    let compiled = CompiledRule::from_rules(&rules);

    // Batch-fetch uncategorized transactions
    let mut tx_stmt = conn.prepare(
        "SELECT id, payee, description, amount FROM transactions WHERE category_id IS NULL"
    )?;
    let txns: Vec<(String, String, String, i64)> = tx_stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    apply_rules_to_rows(&conn, &txns, &compiled)
}

#[tauri::command]
pub fn test_rule(
    match_field: String,
    match_type: String,
    pattern: String,
    state: State<DbState>,
) -> AppResult<Vec<RuleTestResult>> {
    validate_match_field(&match_field)?;
    validate_match_type(&match_type)?;
    if match_type == "regex" {
        validate_regex_pattern(&pattern)?;
    }

    let conn = state.read_conn.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, date, payee, description, amount FROM transactions ORDER BY date DESC LIMIT 500"
    )?;
    let txns: Vec<(String, String, String, String, i64)> = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut results = Vec::new();
    for (id, date, payee, description, amount) in txns {
        let value = get_field_value(&payee, &description, amount, &match_field);
        if matches_value(&value, &match_type, &pattern) {
            results.push(RuleTestResult {
                transaction_id: id,
                date,
                payee: payee.clone(),
                description: description.clone(),
                amount,
                matched_field_value: value,
            });
            if results.len() >= 20 {
                break;
            }
        }
    }

    Ok(results)
}

/// Apply rules to a specific list of transaction IDs (used after import).
/// Batch-fetches transactions instead of N+1 individual queries.
pub fn apply_rules_to_transactions(
    conn: &rusqlite::Connection,
    transaction_ids: &[String],
) -> AppResult<i64> {
    if transaction_ids.is_empty() {
        return Ok(0);
    }

    let rules = load_active_rules(conn)?;
    if rules.is_empty() {
        return Ok(0);
    }
    let compiled = CompiledRule::from_rules(&rules);

    // Batch-fetch transactions by IDs (only uncategorized)
    let placeholders: Vec<String> = (1..=transaction_ids.len()).map(|i| format!("?{}", i)).collect();
    let sql = format!(
        "SELECT id, payee, description, amount FROM transactions WHERE id IN ({}) AND category_id IS NULL",
        placeholders.join(", ")
    );
    let mut stmt = conn.prepare(&sql)?;
    let id_params: Vec<&dyn rusqlite::types::ToSql> = transaction_ids
        .iter()
        .map(|id| id as &dyn rusqlite::types::ToSql)
        .collect();
    let txns: Vec<(String, String, String, i64)> = stmt
        .query_map(id_params.as_slice(), |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    apply_rules_to_rows(conn, &txns, &compiled)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_contains() {
        assert!(matches_value("NETFLIX SUBSCRIPTION", "contains", "netflix"));
        assert!(matches_value("netflix sub", "contains", "NETFLIX"));
        assert!(!matches_value("HULU", "contains", "netflix"));
    }

    #[test]
    fn test_matches_starts_with() {
        assert!(matches_value("NETFLIX SUBSCRIPTION", "starts_with", "netflix"));
        assert!(!matches_value("MY NETFLIX", "starts_with", "netflix"));
    }

    #[test]
    fn test_matches_ends_with() {
        assert!(matches_value("PAY NETFLIX", "ends_with", "netflix"));
        assert!(!matches_value("NETFLIX SUB", "ends_with", "netflix"));
    }

    #[test]
    fn test_matches_exact() {
        assert!(matches_value("Netflix", "exact", "netflix"));
        assert!(!matches_value("Netflix Sub", "exact", "netflix"));
    }

    #[test]
    fn test_matches_regex() {
        assert!(matches_value("NETFLIX 12345", "regex", r"netflix\s+\d+"));
        assert!(!matches_value("HULU 12345", "regex", r"netflix\s+\d+"));
    }

    #[test]
    fn test_matches_invalid_regex_returns_false() {
        assert!(!matches_value("test", "regex", r"[invalid"));
    }

    #[test]
    fn test_get_field_value() {
        assert_eq!(get_field_value("payee1", "desc1", -5000, "payee"), "payee1");
        assert_eq!(get_field_value("payee1", "desc1", -5000, "description"), "desc1");
        assert_eq!(get_field_value("payee1", "desc1", -5000, "amount"), "5000");
    }

    #[test]
    fn test_validate_match_field() {
        assert!(validate_match_field("payee").is_ok());
        assert!(validate_match_field("description").is_ok());
        assert!(validate_match_field("amount").is_ok());
        assert!(validate_match_field("invalid").is_err());
    }

    #[test]
    fn test_validate_match_type() {
        assert!(validate_match_type("contains").is_ok());
        assert!(validate_match_type("starts_with").is_ok());
        assert!(validate_match_type("ends_with").is_ok());
        assert!(validate_match_type("exact").is_ok());
        assert!(validate_match_type("regex").is_ok());
        assert!(validate_match_type("invalid").is_err());
    }

    #[test]
    fn test_validate_regex_pattern() {
        assert!(validate_regex_pattern(r"\d+").is_ok());
        assert!(validate_regex_pattern(r"[invalid").is_err());
    }
}
