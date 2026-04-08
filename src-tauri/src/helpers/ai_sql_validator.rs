use crate::error::{AppError, AppResult};

/// Validate that AI-generated SQL is safe to execute.
/// Only SELECT statements are allowed. All mutation statements are rejected.
pub fn validate_sql(sql: &str) -> AppResult<String> {
    let trimmed = sql.trim();

    // Must not be empty
    if trimmed.is_empty() {
        return Err(AppError::Validation("empty SQL query".to_string()));
    }

    // Extract just the SQL if the model included explanation text
    let sql_only = extract_sql(trimmed);

    let upper = sql_only.to_uppercase();

    // Must start with SELECT or WITH (for CTEs)
    if !upper.starts_with("SELECT") && !upper.starts_with("WITH") {
        return Err(AppError::Validation(format!(
            "only SELECT queries are allowed, got: {}",
            &sql_only[..sql_only.len().min(50)]
        )));
    }

    // Strip string literals before checking for forbidden keywords
    // This prevents false positives like WHERE name = 'Drop Zone'
    let without_strings = strip_string_literals(&sql_only);
    let upper_stripped = without_strings.to_uppercase();

    // Reject dangerous keywords (checked against string-stripped version)
    let forbidden = [
        "DROP ", "DELETE ", "INSERT ", "UPDATE ", "ALTER ", "CREATE ",
        "TRUNCATE ", "REPLACE ", "ATTACH ", "DETACH ",
        "PRAGMA ", "VACUUM", "REINDEX",
        "LOAD_EXTENSION", "EXEC ", "GRANT ", "WRITEFILE", "READFILE",
        "; DROP", ";DROP", "; DELETE", ";DELETE",
        "; INSERT", ";INSERT", "; UPDATE", ";UPDATE",
    ];

    for keyword in forbidden {
        if upper_stripped.contains(keyword) {
            return Err(AppError::Validation(format!(
                "forbidden keyword in SQL: {}",
                keyword.trim()
            )));
        }
    }

    // Reject multiple statements
    if without_strings.matches(';').count() > 1 {
        return Err(AppError::Validation("multiple SQL statements not allowed".to_string()));
    }

    Ok(sql_only)
}

/// Extract SQL from model output that may include explanation text.
/// Models sometimes output "Here is the SQL:\n```sql\nSELECT...\n```"
fn extract_sql(text: &str) -> String {
    // Try to find SQL in code block
    if let Some(start) = text.find("```sql") {
        let after = &text[start + 6..];
        if let Some(end) = after.find("```") {
            return after[..end].trim().to_string();
        }
    }
    if let Some(start) = text.find("```") {
        let after = &text[start + 3..];
        if let Some(end) = after.find("```") {
            let block = after[..end].trim();
            if block.to_uppercase().starts_with("SELECT") || block.to_uppercase().starts_with("WITH") {
                return block.to_string();
            }
        }
    }

    // Try to find a line starting with SELECT
    for line in text.lines() {
        let trimmed = line.trim();
        let upper = trimmed.to_uppercase();
        if upper.starts_with("SELECT") || upper.starts_with("WITH") {
            // Collect this and subsequent lines until we hit an empty line or non-SQL
            let idx = text.find(trimmed).unwrap_or(0);
            let rest = &text[idx..];
            let mut sql_lines = Vec::new();
            for l in rest.lines() {
                let t = l.trim();
                if t.is_empty() && !sql_lines.is_empty() {
                    break;
                }
                if !sql_lines.is_empty() || t.to_uppercase().starts_with("SELECT") || t.to_uppercase().starts_with("WITH") {
                    sql_lines.push(t);
                }
            }
            if !sql_lines.is_empty() {
                return sql_lines.join(" ");
            }
        }
    }

    // Fallback: return the whole thing trimmed
    text.trim().to_string()
}

/// Strip string literals from SQL to avoid false positives on forbidden keywords
/// inside string values (e.g., WHERE name = 'Drop Zone')
fn strip_string_literals(sql: &str) -> String {
    let mut result = String::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let chars: Vec<char> = sql.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let ch = chars[i];
        match ch {
            '\'' if !in_double_quote => {
                if in_single_quote {
                    // Check for escaped quote ('') inside a single-quoted string
                    if i + 1 < len && chars[i + 1] == '\'' {
                        // Escaped quote — stays inside the string
                        result.push('_');
                        result.push('_');
                        i += 2;
                        continue;
                    }
                    // Closing quote
                    in_single_quote = false;
                } else {
                    // Opening quote
                    in_single_quote = true;
                }
                result.push(ch);
            }
            '"' if !in_single_quote => {
                in_double_quote = !in_double_quote;
                result.push(ch);
            }
            _ if in_single_quote || in_double_quote => {
                result.push('_'); // Replace content with placeholder
            }
            _ => result.push(ch),
        }
        i += 1;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_select() {
        let result = validate_sql("SELECT * FROM transactions");
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_with_cte() {
        let result = validate_sql("WITH totals AS (SELECT SUM(amount) FROM transactions) SELECT * FROM totals");
        assert!(result.is_ok());
    }

    #[test]
    fn test_reject_drop() {
        let result = validate_sql("DROP TABLE transactions");
        assert!(result.is_err());
    }

    #[test]
    fn test_reject_delete() {
        let result = validate_sql("DELETE FROM transactions");
        assert!(result.is_err());
    }

    #[test]
    fn test_reject_insert() {
        let result = validate_sql("INSERT INTO transactions VALUES (1)");
        assert!(result.is_err());
    }

    #[test]
    fn test_reject_injection_in_select() {
        let result = validate_sql("SELECT * FROM transactions; DROP TABLE transactions");
        assert!(result.is_err());
    }

    #[test]
    fn test_allow_string_with_keyword() {
        // "Drop Zone" in a string should be fine
        let result = validate_sql("SELECT * FROM transactions WHERE payee = 'Drop Zone'");
        assert!(result.is_ok());
    }

    #[test]
    fn test_extract_sql_from_code_block() {
        let text = "Here is the SQL:\n```sql\nSELECT COUNT(*) FROM transactions\n```";
        let result = validate_sql(text);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "SELECT COUNT(*) FROM transactions");
    }

    #[test]
    fn test_extract_sql_from_mixed_text() {
        let text = "I'll query that for you.\nSELECT SUM(amount) FROM transactions WHERE kind = 'expense'";
        let result = validate_sql(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_reject_empty() {
        assert!(validate_sql("").is_err());
        assert!(validate_sql("   ").is_err());
    }

    #[test]
    fn test_reject_update() {
        assert!(validate_sql("UPDATE transactions SET amount = 0").is_err());
    }

    #[test]
    fn test_reject_load_extension() {
        let result = validate_sql("SELECT load_extension('evil.so')");
        assert!(result.is_err());
    }

    #[test]
    fn test_reject_writefile() {
        let result = validate_sql("SELECT writefile('/tmp/evil', data) FROM transactions");
        assert!(result.is_err());
    }

    #[test]
    fn test_reject_readfile() {
        let result = validate_sql("SELECT readfile('/etc/passwd')");
        assert!(result.is_err());
    }

    #[test]
    fn test_reject_grant() {
        let result = validate_sql("GRANT ALL ON transactions TO public");
        assert!(result.is_err());
    }

    #[test]
    fn test_escaped_quote_in_string() {
        // SQL escaped quote ('') inside a string should not break parsing
        let result = validate_sql("SELECT * FROM transactions WHERE payee = 'it''s okay'");
        assert!(result.is_ok());
    }

    #[test]
    fn test_escaped_quote_does_not_leak_forbidden() {
        // Ensure escaped quote doesn't cause the rest of the string to be unquoted
        // allowing a forbidden keyword to be detected
        let result = validate_sql("SELECT * FROM transactions WHERE payee = 'it''s DROP TABLE'");
        assert!(result.is_ok(), "Forbidden keyword inside a string with escaped quotes should be safe");
    }
}
