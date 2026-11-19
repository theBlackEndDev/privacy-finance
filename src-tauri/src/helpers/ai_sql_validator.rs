/// Extract SQL from model output that may include explanation text.
/// Models sometimes output "Here is the SQL:\n```sql\nSELECT...\n```"
pub fn extract_sql(text: &str) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_sql_from_code_block() {
        let text = "Here is the SQL:\n```sql\nSELECT COUNT(*) FROM transactions\n```";
        assert_eq!(extract_sql(text), "SELECT COUNT(*) FROM transactions");
    }

    #[test]
    fn test_extract_sql_from_mixed_text() {
        let text = "I'll query that for you.\nSELECT SUM(amount) FROM transactions WHERE kind = 'expense'";
        assert_eq!(extract_sql(text), "SELECT SUM(amount) FROM transactions WHERE kind = 'expense'");
    }

    #[test]
    fn test_extract_sql_plain() {
        assert_eq!(extract_sql("  SELECT 1  "), "SELECT 1");
    }
}
