use chrono::{Datelike, Local, NaiveDate, Duration};
use rusqlite::Connection;
use crate::models::ai_search::{DatePeriod, ExtractedEntities};

// ── Top-level extraction ──────────────────────────────────────────

pub fn extract_entities(query: &str, conn: &Connection) -> ExtractedEntities {
    let q = query.trim().to_lowercase().replace('?', "");

    let (category_id, category_name) = match_category(&q, conn);
    let (account_id, account_name) = match_account(&q, conn);
    let date_period = parse_date_period(&q);
    let comparison_period = parse_comparison_period(&q);
    let (amount_threshold, amount_operator) = parse_amount(&q);
    let merchant = match_merchant(&q, conn);

    ExtractedEntities {
        category_id,
        category_name,
        account_id,
        account_name,
        date_period,
        comparison_period,
        amount_threshold,
        amount_operator,
        merchant,
    }
}

// ── Category fuzzy matcher ────────────────────────────────────────

/// Common words that users say → category names they likely mean.
/// This bridges the gap between natural language and category names.
const CATEGORY_ALIASES: &[(&str, &[&str])] = &[
    ("utilities", &["electricity", "electric", "power", "water", "gas", "utility", "internet", "wifi", "cable", "phone", "cell"]),
    ("food", &["groceries", "grocery", "eating", "meals", "supermarket", "food"]),
    ("dining", &["restaurants", "restaurant", "eating out", "takeout", "delivery", "dining", "lunch", "dinner", "brunch"]),
    ("housing", &["rent", "mortgage", "housing", "apartment", "home"]),
    ("transportation", &["gas", "fuel", "uber", "lyft", "transit", "bus", "subway", "transport", "car", "auto", "parking"]),
    ("entertainment", &["movies", "games", "gaming", "netflix", "spotify", "streaming", "fun", "entertainment", "music"]),
    ("shopping", &["amazon", "clothes", "clothing", "retail", "shopping", "online"]),
    ("health", &["medical", "doctor", "pharmacy", "health", "dental", "vision", "hospital", "prescription", "gym", "fitness"]),
    ("insurance", &["insurance", "premium", "coverage"]),
    ("education", &["tuition", "school", "books", "courses", "education", "learning"]),
    ("subscriptions", &["subscription", "subscriptions", "monthly", "recurring"]),
    ("travel", &["hotel", "flight", "airfare", "vacation", "travel", "trip"]),
    ("personal", &["haircut", "personal", "beauty", "spa", "salon"]),
    ("pets", &["pet", "pets", "vet", "veterinary", "dog", "cat"]),
    ("gifts", &["gift", "gifts", "donation", "charity", "giving"]),
    ("salary", &["salary", "paycheck", "wages", "pay"]),
    ("freelance", &["freelance", "contract", "consulting", "gig", "side hustle"]),
    ("investments", &["investment", "investments", "stocks", "dividend", "interest", "returns"]),
];

fn match_category(query: &str, conn: &Connection) -> (Option<String>, Option<String>) {
    let mut stmt = match conn.prepare("SELECT id, name FROM categories") {
        Ok(s) => s,
        Err(_) => return (None, None),
    };
    let cats: Vec<(String, String)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .ok()
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_default();

    // 1. Exact substring match (case-insensitive)
    for (id, name) in &cats {
        let lower = name.to_lowercase();
        if query.contains(&lower) {
            return (Some(id.clone()), Some(name.clone()));
        }
    }

    // 2. Alias matching — user said "electricity", we find "Utilities"
    let query_words: Vec<&str> = query.split_whitespace().collect();
    for (cat_name, aliases) in CATEGORY_ALIASES {
        for alias in *aliases {
            // Check if any alias appears in the query
            if query.contains(alias) || query_words.iter().any(|w| *w == *alias) {
                // Find this category in the user's actual categories
                for (id, name) in &cats {
                    if name.to_lowercase() == *cat_name
                        || name.to_lowercase().contains(cat_name)
                        || cat_name.contains(&name.to_lowercase())
                    {
                        return (Some(id.clone()), Some(name.clone()));
                    }
                }
            }
        }
    }

    // 3. Fuzzy match (Levenshtein distance) — catches typos
    let mut best_match: Option<(String, String, usize)> = None;
    for (id, name) in &cats {
        let lower = name.to_lowercase();
        for word in &query_words {
            if word.len() < 3 { continue; } // skip short words
            let dist = levenshtein(word, &lower);
            let threshold = if lower.len() <= 4 { 1 } else if lower.len() <= 6 { 2 } else { 3 };
            if dist <= threshold {
                if best_match.is_none() || dist < best_match.as_ref().unwrap().2 {
                    best_match = Some((id.clone(), name.clone(), dist));
                }
            }
        }
        // Also try matching each word against multi-word category names
        let cat_words: Vec<&str> = lower.split_whitespace().collect();
        for cat_word in &cat_words {
            for word in &query_words {
                if word.len() < 3 { continue; }
                let dist = levenshtein(word, cat_word);
                if dist <= 1 {
                    if best_match.is_none() || dist < best_match.as_ref().unwrap().2 {
                        best_match = Some((id.clone(), name.clone(), dist));
                    }
                }
            }
        }
    }

    match best_match {
        Some((id, name, _)) => (Some(id), Some(name)),
        None => (None, None),
    }
}

// ── Account matcher ───────────────────────────────────────────────

fn match_account(query: &str, conn: &Connection) -> (Option<String>, Option<String>) {
    let mut stmt = match conn.prepare("SELECT id, name FROM accounts WHERE is_archived = 0") {
        Ok(s) => s,
        Err(_) => return (None, None),
    };
    let accounts: Vec<(String, String)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .ok()
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_default();

    for (id, name) in &accounts {
        if query.contains(&name.to_lowercase()) {
            return (Some(id.clone()), Some(name.clone()));
        }
    }
    (None, None)
}

// ── Date period parser ────────────────────────────────────────────

fn parse_date_period(query: &str) -> Option<DatePeriod> {
    let today = Local::now().date_naive();

    // Rolling windows
    if let Some(days) = extract_rolling_days(query) {
        let start = today - Duration::days(days);
        return Some(DatePeriod {
            start_date: start.format("%Y-%m-%d").to_string(),
            end_date: today.format("%Y-%m-%d").to_string(),
            label: format!("Last {} days", days),
        });
    }

    // Relative periods
    if query.contains("today") {
        return Some(date_period_single_day(today, "Today"));
    }
    if query.contains("yesterday") {
        let d = today - Duration::days(1);
        return Some(date_period_single_day(d, "Yesterday"));
    }
    if query.contains("this week") {
        let start = today - Duration::days(today.weekday().num_days_from_monday() as i64);
        return Some(DatePeriod {
            start_date: start.format("%Y-%m-%d").to_string(),
            end_date: today.format("%Y-%m-%d").to_string(),
            label: "This week".to_string(),
        });
    }
    if query.contains("last week") {
        let this_monday = today - Duration::days(today.weekday().num_days_from_monday() as i64);
        let start = this_monday - Duration::days(7);
        let end = this_monday - Duration::days(1);
        return Some(DatePeriod {
            start_date: start.format("%Y-%m-%d").to_string(),
            end_date: end.format("%Y-%m-%d").to_string(),
            label: "Last week".to_string(),
        });
    }
    if query.contains("this month") {
        let start = NaiveDate::from_ymd_opt(today.year(), today.month(), 1)?;
        return Some(DatePeriod {
            start_date: start.format("%Y-%m-%d").to_string(),
            end_date: today.format("%Y-%m-%d").to_string(),
            label: "This month".to_string(),
        });
    }
    if query.contains("last month") {
        let (y, m) = prev_month(today.year(), today.month());
        let start = NaiveDate::from_ymd_opt(y, m, 1)?;
        let end = NaiveDate::from_ymd_opt(today.year(), today.month(), 1)? - Duration::days(1);
        return Some(DatePeriod {
            start_date: start.format("%Y-%m-%d").to_string(),
            end_date: end.format("%Y-%m-%d").to_string(),
            label: "Last month".to_string(),
        });
    }
    if query.contains("this quarter") {
        let q = (today.month() - 1) / 3;
        let start_month = q * 3 + 1;
        let start = NaiveDate::from_ymd_opt(today.year(), start_month, 1)?;
        return Some(DatePeriod {
            start_date: start.format("%Y-%m-%d").to_string(),
            end_date: today.format("%Y-%m-%d").to_string(),
            label: format!("Q{} {}", q + 1, today.year()),
        });
    }
    if query.contains("this year") {
        let start = NaiveDate::from_ymd_opt(today.year(), 1, 1)?;
        return Some(DatePeriod {
            start_date: start.format("%Y-%m-%d").to_string(),
            end_date: today.format("%Y-%m-%d").to_string(),
            label: format!("{}", today.year()),
        });
    }
    if query.contains("last year") {
        let y = today.year() - 1;
        let start = NaiveDate::from_ymd_opt(y, 1, 1)?;
        let end = NaiveDate::from_ymd_opt(y, 12, 31)?;
        return Some(DatePeriod {
            start_date: start.format("%Y-%m-%d").to_string(),
            end_date: end.format("%Y-%m-%d").to_string(),
            label: format!("{}", y),
        });
    }

    // Quarter references: Q1, Q2, Q3, Q4
    for qn in 1..=4 {
        if query.contains(&format!("q{}", qn)) {
            let start_month = (qn - 1) * 3 + 1;
            let end_month = qn * 3;
            let year = today.year();
            let start = NaiveDate::from_ymd_opt(year, start_month, 1)?;
            let end = end_of_month(year, end_month)?;
            return Some(DatePeriod {
                start_date: start.format("%Y-%m-%d").to_string(),
                end_date: end.format("%Y-%m-%d").to_string(),
                label: format!("Q{} {}", qn, year),
            });
        }
    }

    // Named months
    let months = [
        ("january", 1), ("february", 2), ("march", 3), ("april", 4),
        ("may", 5), ("june", 6), ("july", 7), ("august", 8),
        ("september", 9), ("october", 10), ("november", 11), ("december", 12),
        ("jan", 1), ("feb", 2), ("mar", 3), ("apr", 4),
        ("jun", 6), ("jul", 7), ("aug", 8), ("sep", 9),
        ("oct", 10), ("nov", 11), ("dec", 12),
    ];
    for (name, m) in months {
        if query.contains(name) {
            let year = if m > today.month() { today.year() - 1 } else { today.year() };
            let start = NaiveDate::from_ymd_opt(year, m, 1)?;
            let end = end_of_month(year, m)?;
            return Some(DatePeriod {
                start_date: start.format("%Y-%m-%d").to_string(),
                end_date: end.format("%Y-%m-%d").to_string(),
                label: format!("{} {}", start.format("%B"), year),
            });
        }
    }

    None
}

fn parse_comparison_period(query: &str) -> Option<DatePeriod> {
    // For "this month vs last month" — we parse the second period
    if (query.contains(" vs ") || query.contains(" versus ") || query.contains("compared to") || query.contains("compare"))
        && query.contains("last month")
    {
        let today = Local::now().date_naive();
        let (y, m) = prev_month(today.year(), today.month());
        let start = NaiveDate::from_ymd_opt(y, m, 1)?;
        let end = NaiveDate::from_ymd_opt(today.year(), today.month(), 1)? - Duration::days(1);
        return Some(DatePeriod {
            start_date: start.format("%Y-%m-%d").to_string(),
            end_date: end.format("%Y-%m-%d").to_string(),
            label: "Last month".to_string(),
        });
    }
    None
}

// ── Amount parser ─────────────────────────────────────────────────

fn parse_amount(query: &str) -> (Option<i64>, Option<String>) {
    if let Ok(re_over) = regex::Regex::new(r"(?:over|above|more than|greater than|exceeding)\s*\$?([\d,]+(?:\.\d{1,2})?)") {
        if let Some(caps) = re_over.captures(query) {
            if let Some(val) = parse_dollar_amount(&caps[1]) {
                return (Some(val), Some("over".to_string()));
            }
        }
    }

    if let Ok(re_under) = regex::Regex::new(r"(?:under|below|less than|cheaper than)\s*\$?([\d,]+(?:\.\d{1,2})?)") {
        if let Some(caps) = re_under.captures(query) {
            if let Some(val) = parse_dollar_amount(&caps[1]) {
                return (Some(val), Some("under".to_string()));
            }
        }
    }

    // Standalone dollar amount (e.g., "transactions $200")
    if let Ok(re_amount) = regex::Regex::new(r"\$\s*([\d,]+(?:\.\d{1,2})?)") {
        if let Some(caps) = re_amount.captures(query) {
            if let Some(val) = parse_dollar_amount(&caps[1]) {
                return (Some(val), Some("over".to_string()));
            }
        }
    }

    (None, None)
}

fn parse_dollar_amount(s: &str) -> Option<i64> {
    let cleaned = s.replace(',', "");
    let f: f64 = cleaned.parse().ok()?;
    Some((f * 100.0) as i64) // convert to cents
}

// ── Merchant matcher ──────────────────────────────────────────────

fn match_merchant(query: &str, conn: &Connection) -> Option<String> {
    let mut stmt = conn.prepare(
        "SELECT DISTINCT payee FROM transactions WHERE payee != '' AND payee IS NOT NULL LIMIT 500"
    ).ok()?;
    let payees: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .ok()?
        .filter_map(|r| r.ok())
        .collect();

    // Exact substring match
    for payee in &payees {
        if query.contains(&payee.to_lowercase()) {
            return Some(payee.clone());
        }
    }

    // Fuzzy match on query words
    let words: Vec<&str> = query.split_whitespace().collect();
    for payee in &payees {
        let lower = payee.to_lowercase();
        for word in &words {
            if word.len() >= 3 && levenshtein(word, &lower) <= 2 {
                return Some(payee.clone());
            }
        }
    }

    None
}

// ── Helpers ───────────────────────────────────────────────────────

fn extract_rolling_days(query: &str) -> Option<i64> {
    let re = regex::Regex::new(r"(?:last|past)\s+(\d+)\s+days?").ok()?;
    let caps = re.captures(query)?;
    caps[1].parse().ok()
}

fn prev_month(year: i32, month: u32) -> (i32, u32) {
    if month == 1 { (year - 1, 12) } else { (year, month - 1) }
}

fn end_of_month(year: i32, month: u32) -> Option<NaiveDate> {
    if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1).map(|d| d - Duration::days(1))
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1).map(|d| d - Duration::days(1))
    }
}

fn date_period_single_day(date: NaiveDate, label: &str) -> DatePeriod {
    DatePeriod {
        start_date: date.format("%Y-%m-%d").to_string(),
        end_date: date.format("%Y-%m-%d").to_string(),
        label: label.to_string(),
    }
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let (m, n) = (a.len(), b.len());
    let mut dp = vec![vec![0usize; n + 1]; m + 1];
    for i in 0..=m { dp[i][0] = i; }
    for j in 0..=n { dp[0][j] = j; }
    for i in 1..=m {
        for j in 1..=n {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            dp[i][j] = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + cost);
        }
    }
    dp[m][n]
}

// ── Tests ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein() {
        assert_eq!(levenshtein("dining", "dining"), 0);
        assert_eq!(levenshtein("dinning", "dining"), 1);
        assert_eq!(levenshtein("food", "good"), 1);
        assert_eq!(levenshtein("cat", "car"), 1);
    }

    #[test]
    fn test_parse_date_last_month() {
        let result = parse_date_period("how much did i spend last month");
        assert!(result.is_some());
        assert_eq!(result.as_ref().unwrap().label, "Last month");
    }

    #[test]
    fn test_parse_date_this_year() {
        let result = parse_date_period("total spending this year");
        assert!(result.is_some());
        let today = Local::now().date_naive();
        assert!(result.as_ref().unwrap().label.contains(&today.year().to_string()));
    }

    #[test]
    fn test_parse_date_rolling() {
        let result = parse_date_period("spending in the last 30 days");
        assert!(result.is_some());
        assert_eq!(result.as_ref().unwrap().label, "Last 30 days");
    }

    #[test]
    fn test_parse_amount_over() {
        let (amt, op) = parse_amount("transactions over $200");
        assert_eq!(amt, Some(20000)); // cents
        assert_eq!(op.as_deref(), Some("over"));
    }

    #[test]
    fn test_parse_amount_under() {
        let (amt, op) = parse_amount("purchases under $50");
        assert_eq!(amt, Some(5000));
        assert_eq!(op.as_deref(), Some("under"));
    }

    #[test]
    fn test_parse_amount_dollar_sign() {
        let (amt, op) = parse_amount("show me transactions $100");
        assert_eq!(amt, Some(10000));
        assert!(op.is_some());
    }

    #[test]
    fn test_category_match_exact() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE categories (id TEXT, name TEXT);
            INSERT INTO categories VALUES ('c1', 'Dining');
            INSERT INTO categories VALUES ('c2', 'Groceries');").unwrap();
        let (id, name) = match_category("spending on dining", &conn);
        assert_eq!(id.as_deref(), Some("c1"));
        assert_eq!(name.as_deref(), Some("Dining"));
    }

    #[test]
    fn test_category_fuzzy_match() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE categories (id TEXT, name TEXT);
            INSERT INTO categories VALUES ('c1', 'Dining');").unwrap();
        let (id, _) = match_category("how much on dinning", &conn);
        assert_eq!(id.as_deref(), Some("c1")); // Levenshtein 1
    }

    #[test]
    fn test_merchant_match() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE transactions (id TEXT, payee TEXT, amount INTEGER, kind TEXT, date TEXT, category_id TEXT, account_id TEXT, description TEXT);
            INSERT INTO transactions VALUES ('t1', 'Netflix', -1599, 'expense', '2026-01-01', NULL, 'a1', '');").unwrap();
        let result = match_merchant("how much at netflix", &conn);
        assert_eq!(result.as_deref(), Some("Netflix"));
    }
}
