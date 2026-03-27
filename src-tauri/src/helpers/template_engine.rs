use rusqlite::Connection;
use serde::Serialize;
use crate::error::{AppError, AppResult};

/// Result of a template-matched query.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InferenceResult {
    pub matched: bool,
    pub template_name: Option<String>,
    pub answer: Option<String>,
    pub raw_value: Option<f64>,
    pub query_sql: Option<String>,
    pub suggestions: Vec<String>,
}

/// Template definition: a natural language pattern → SQL mapping.
struct Template {
    name: &'static str,
    patterns: &'static [&'static str],
    sql: &'static str,
    format_fn: fn(f64) -> String,
}

const TEMPLATES: &[Template] = &[
    Template {
        name: "total_spending_category",
        patterns: &[
            "how much did i spend on",
            "what did i spend on",
            "total spending on",
            "spending on",
        ],
        sql: "SELECT COALESCE(SUM(ABS(t.amount)), 0) FROM transactions t
              JOIN categories c ON t.category_id = c.id
              WHERE LOWER(c.name) = LOWER(?1) AND t.kind = 'expense'",
        format_fn: format_currency,
    },
    Template {
        name: "total_income",
        patterns: &[
            "how much did i earn",
            "total income",
            "how much income",
            "what did i earn",
        ],
        sql: "SELECT COALESCE(SUM(ABS(t.amount)), 0) FROM transactions t WHERE t.kind = 'income'",
        format_fn: format_currency,
    },
    Template {
        name: "total_spending",
        patterns: &[
            "how much did i spend",
            "total spending",
            "total expenses",
            "what did i spend total",
        ],
        sql: "SELECT COALESCE(SUM(ABS(t.amount)), 0) FROM transactions t WHERE t.kind = 'expense'",
        format_fn: format_currency,
    },
    Template {
        name: "transaction_count",
        patterns: &[
            "how many transactions",
            "number of transactions",
            "transaction count",
        ],
        sql: "SELECT COUNT(*) FROM transactions",
        format_fn: format_count,
    },
    Template {
        name: "account_balance",
        patterns: &[
            "what is my balance",
            "account balance",
            "how much do i have",
            "what's my balance",
        ],
        sql: "SELECT COALESCE(SUM(CASE WHEN t.kind = 'income' THEN t.amount
              WHEN t.kind = 'expense' THEN -ABS(t.amount) ELSE 0 END), 0)
              + COALESCE((SELECT SUM(opening_balance) FROM accounts WHERE is_archived = 0), 0)
              FROM transactions t
              JOIN accounts a ON t.account_id = a.id WHERE a.is_archived = 0",
        format_fn: format_currency,
    },
    Template {
        name: "top_category",
        patterns: &[
            "what do i spend the most on",
            "biggest expense category",
            "top spending category",
            "where does my money go",
        ],
        sql: "SELECT c.name FROM transactions t
              JOIN categories c ON t.category_id = c.id
              WHERE t.kind = 'expense'
              GROUP BY c.id ORDER BY SUM(ABS(t.amount)) DESC LIMIT 1",
        format_fn: |_| String::new(), // special handling
    },
    Template {
        name: "transactions_over",
        patterns: &[
            "transactions over",
            "spending over",
            "charges over",
        ],
        sql: "SELECT COUNT(*) FROM transactions WHERE ABS(amount) > ?1",
        format_fn: format_count,
    },
];

/// Try to match a natural language query against templates and execute.
pub fn infer(query: &str, conn: &Connection) -> AppResult<InferenceResult> {
    let normalized = query.trim().to_lowercase();

    for template in TEMPLATES {
        for pattern in template.patterns {
            if normalized.starts_with(pattern) || normalized.contains(pattern) {
                let remainder = extract_remainder(&normalized, pattern);
                return execute_template(template, &remainder, conn);
            }
        }
    }

    // No match — return suggestions
    Ok(InferenceResult {
        matched: false,
        template_name: None,
        answer: None,
        raw_value: None,
        query_sql: None,
        suggestions: vec![
            "How much did I spend on [category]?".to_string(),
            "How much did I earn?".to_string(),
            "What is my balance?".to_string(),
            "What do I spend the most on?".to_string(),
            "How many transactions do I have?".to_string(),
        ],
    })
}

fn execute_template(template: &Template, remainder: &str, conn: &Connection) -> AppResult<InferenceResult> {
    match template.name {
        "total_spending_category" => {
            let category = remainder.trim().trim_end_matches('?');
            if category.is_empty() {
                return Ok(InferenceResult {
                    matched: false,
                    template_name: Some(template.name.to_string()),
                    answer: None,
                    raw_value: None,
                    query_sql: None,
                    suggestions: vec!["Please specify a category, e.g., 'How much did I spend on dining?'".to_string()],
                });
            }
            let value: f64 = conn.query_row(template.sql, rusqlite::params![category], |row| row.get(0))
                .unwrap_or(0.0);
            let cents = value;
            let dollars = cents / 100.0;
            Ok(InferenceResult {
                matched: true,
                template_name: Some(template.name.to_string()),
                answer: Some(format!("You spent {} on {}.", format_currency(cents), category)),
                raw_value: Some(dollars),
                query_sql: Some(template.sql.to_string()),
                suggestions: vec![],
            })
        }
        "transactions_over" => {
            let amount_str = remainder.trim().trim_start_matches('$').trim_end_matches('?');
            let amount: f64 = amount_str.parse().unwrap_or(0.0);
            let cents = (amount * 100.0) as i64;
            let count: f64 = conn.query_row(template.sql, rusqlite::params![cents], |row| row.get(0))
                .unwrap_or(0.0);
            Ok(InferenceResult {
                matched: true,
                template_name: Some(template.name.to_string()),
                answer: Some(format!("You have {} transactions over ${}.", count as i64, amount)),
                raw_value: Some(count),
                query_sql: Some(template.sql.to_string()),
                suggestions: vec![],
            })
        }
        "top_category" => {
            let name: String = conn.query_row(template.sql, [], |row| row.get(0))
                .unwrap_or_else(|_| "unknown".to_string());
            Ok(InferenceResult {
                matched: true,
                template_name: Some(template.name.to_string()),
                answer: Some(format!("Your biggest expense category is {}.", name)),
                raw_value: None,
                query_sql: Some(template.sql.to_string()),
                suggestions: vec![],
            })
        }
        _ => {
            // Generic: execute SQL, format result
            let value: f64 = conn.query_row(template.sql, [], |row| row.get(0))
                .unwrap_or(0.0);
            let formatted = (template.format_fn)(value);
            Ok(InferenceResult {
                matched: true,
                template_name: Some(template.name.to_string()),
                answer: Some(formatted),
                raw_value: Some(value),
                query_sql: Some(template.sql.to_string()),
                suggestions: vec![],
            })
        }
    }
}

fn extract_remainder(query: &str, pattern: &str) -> String {
    if let Some(idx) = query.find(pattern) {
        query[idx + pattern.len()..].trim().to_string()
    } else {
        String::new()
    }
}

fn format_currency(cents: f64) -> String {
    let dollars = cents / 100.0;
    if dollars >= 0.0 {
        format!("${:.2}", dollars)
    } else {
        format!("-${:.2}", dollars.abs())
    }
}

fn format_count(count: f64) -> String {
    format!("{}", count as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_spending_category() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("
            CREATE TABLE categories (id TEXT PRIMARY KEY, name TEXT);
            CREATE TABLE transactions (id TEXT, category_id TEXT, amount INTEGER, kind TEXT, account_id TEXT, date TEXT);
            INSERT INTO categories VALUES ('c1', 'Dining');
            INSERT INTO transactions VALUES ('t1', 'c1', -5000, 'expense', 'a1', '2026-01-01');
            INSERT INTO transactions VALUES ('t2', 'c1', -3000, 'expense', 'a1', '2026-01-02');
        ").unwrap();

        let result = infer("how much did i spend on dining", &conn).unwrap();
        assert!(result.matched);
        assert_eq!(result.template_name.as_deref(), Some("total_spending_category"));
        assert_eq!(result.raw_value, Some(80.0)); // 8000 cents / 100
    }

    #[test]
    fn matches_total_income() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("
            CREATE TABLE transactions (id TEXT, amount INTEGER, kind TEXT, account_id TEXT, date TEXT, category_id TEXT);
            INSERT INTO transactions VALUES ('t1', 100000, 'income', 'a1', '2026-01-01', NULL);
        ").unwrap();

        let result = infer("how much did i earn", &conn).unwrap();
        assert!(result.matched);
        assert_eq!(result.template_name.as_deref(), Some("total_income"));
    }

    #[test]
    fn no_match_returns_suggestions() {
        let conn = Connection::open_in_memory().unwrap();
        let result = infer("what color is the sky", &conn).unwrap();
        assert!(!result.matched);
        assert!(!result.suggestions.is_empty());
    }

    #[test]
    fn matches_transaction_count() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("
            CREATE TABLE transactions (id TEXT, amount INTEGER, kind TEXT);
            INSERT INTO transactions VALUES ('t1', 5000, 'expense');
            INSERT INTO transactions VALUES ('t2', 3000, 'expense');
        ").unwrap();

        let result = infer("how many transactions", &conn).unwrap();
        assert!(result.matched);
        assert_eq!(result.raw_value, Some(2.0));
    }
}
