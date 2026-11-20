use chrono::{Datelike, Duration, NaiveDate};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::error::AppResult;
use crate::helpers::ai_engine::Engine;
use crate::helpers::ai_entities::{date_period_single_day, end_of_month, prev_month, strip_punctuation};
use crate::models::ai_search::{CardType, DatePeriod, ExtractedEntities, IntentMatch, SearchIntent};

/// The JSON object the model router emits (Locked decision D2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RouterOutput {
    pub intent: String,
    pub category: Option<String>,
    pub account: Option<String>,
    pub merchant: Option<String>,
    pub period: Option<String>,
    pub amount: Option<i64>,
    pub operator: Option<String>,
}

const ROUTER_CONFIDENCE: f64 = 0.8;

/// The 12 D2 intents with a one-line description and one example question (D5).
/// The examples must not appear in the router eval fixture.
const INTENTS: [(&str, &str, &str); 12] = [
    ("spending_total", "total money spent, optionally in a period or account", "What did I spend overall in 2025-12?"),
    ("spending_by_category", "spending in one named category", "How much has gone to Pets since the start of the year?"),
    ("spending_comparison", "compare spending between two periods", "Am I burning through more cash than the month before?"),
    ("income_total", "total money earned", "How much did I earn in 2025-11?"),
    ("balance_check", "current balance of one account or all accounts", "What is the balance of my emergency fund?"),
    ("budget_status", "progress against budgets", "Which of my budgets are close to the limit?"),
    ("transaction_search", "list transactions by merchant, category, account, period or amount", "Show every charge from Netflix above $15."),
    ("top_merchants", "the merchants with the highest spending", "Which stores take most of my budget lately?"),
    ("top_expenses", "the largest single expenses", "What were the priciest things I bought last week?"),
    ("recurring_check", "recurring bills and subscriptions", "What subscriptions am I paying for?"),
    ("net_worth", "total net worth across accounts", "What am I worth overall?"),
    ("category_detail", "transactions and detail inside one named category", "Break down what I put under Pets."),
];

/// Worked examples for the router prompt (Locked decision D16): a question and the exact JSON.
/// The questions must not appear in, or share a distinctive phrase with, the router eval fixture.
const EXAMPLES: [(&str, &str); 13] = [
    ("How much did I spend on Education in 2025-09?", r#"{"intent": "spending_by_category", "category": "Education", "account": null, "merchant": null, "period": "2025-09", "amount": null, "operator": null}"#),
    ("What have I spent on Personal?", r#"{"intent": "spending_by_category", "category": "Personal", "account": null, "merchant": null, "period": null, "amount": null, "operator": null}"#),
    ("What is my total spending this month?", r#"{"intent": "spending_total", "category": null, "account": null, "merchant": null, "period": "this_month", "amount": null, "operator": null}"#),
    ("What is my total spending?", r#"{"intent": "spending_total", "category": null, "account": null, "merchant": null, "period": null, "amount": null, "operator": null}"#),
    ("How much have I earned?", r#"{"intent": "income_total", "category": null, "account": null, "merchant": null, "period": null, "amount": null, "operator": null}"#),
    ("What is the balance in High-Yield Savings?", r#"{"intent": "balance_check", "category": null, "account": "High-Yield Savings", "merchant": null, "period": null, "amount": null, "operator": null}"#),
    ("List purchases at Costco over $75", r#"{"intent": "transaction_search", "category": null, "account": null, "merchant": "Costco", "period": null, "amount": 75, "operator": "over"}"#),
    ("Any Netflix payments in the last 90 days?", r#"{"intent": "transaction_search", "category": null, "account": null, "merchant": "Netflix", "period": "last_90_days", "amount": null, "operator": null}"#),
    ("What is the single largest purchase I have made?", r#"{"intent": "top_expenses", "category": null, "account": null, "merchant": null, "period": null, "amount": null, "operator": null}"#),
    ("Which merchants cost me the most this quarter?", r#"{"intent": "top_merchants", "category": null, "account": null, "merchant": null, "period": "this_quarter", "amount": null, "operator": null}"#),
    ("Which of my budgets am I over?", r#"{"intent": "budget_status", "category": null, "account": null, "merchant": null, "period": null, "amount": null, "operator": null}"#),
    ("What recurring charges do I have?", r#"{"intent": "recurring_check", "category": null, "account": null, "merchant": null, "period": null, "amount": null, "operator": null}"#),
    ("Will it rain tomorrow?", r#"{"intent": "other", "category": null, "account": null, "merchant": null, "period": null, "amount": null, "operator": null}"#),
];

const PERIODS: [&str; 12] = [
    "today", "yesterday", "this_week", "last_week", "this_month", "last_month",
    "this_quarter", "this_year", "last_year", "last_7_days", "last_30_days", "last_90_days",
];

/// Escape `s` as a GBNF string literal that matches the JSON string literal of `s`.
/// JSON-escaping handles `"`, `\` and control characters; the GBNF layer then escapes
/// `"` and `\` again, and wraps the result in quotes.
fn gbnf_json_string(s: &str) -> String {
    let json = serde_json::to_string(s).unwrap_or_else(|_| "\"\"".to_string());
    gbnf_literal(&json)
}

/// A GBNF string literal that matches `s` exactly.
fn gbnf_literal(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Non-archived names from `table` (fixed internal literal), in a stable order.
fn active_names(conn: &Connection, table: &str) -> AppResult<Vec<String>> {
    let sql = format!("SELECT name FROM {table} WHERE is_archived = 0 ORDER BY name");
    let mut stmt = conn.prepare(&sql)?;
    let names = stmt.query_map([], |r| r.get::<_, String>(0))?.collect::<Result<Vec<_>, _>>()?;
    Ok(names)
}

/// `"null" | <name> | <name> ...` as a GBNF alternation.
fn nullable_names(names: &[String]) -> String {
    let mut alts = vec!["\"null\"".to_string()];
    alts.extend(names.iter().map(|n| gbnf_json_string(n)));
    alts.join(" | ")
}

/// Build the GBNF grammar for the router output (Locked decisions D3, D15).
/// Amount and operator are one rule (`amtop`): both null, or an integer with `over`/`under`.
pub fn build_router_grammar(conn: &Connection) -> AppResult<String> {
    let categories = active_names(conn, "categories")?;
    let accounts = active_names(conn, "accounts")?;

    let intent_alts = INTENTS
        .iter()
        .map(|(name, _, _)| *name)
        .chain(std::iter::once("other"))
        .map(|n| gbnf_json_string(n))
        .collect::<Vec<_>>()
        .join(" | ");
    let period_alts = PERIODS
        .iter()
        .map(|p| gbnf_json_string(p))
        .collect::<Vec<_>>()
        .join(" | ");

    Ok(format!(
        "root ::= \"{{\\\"intent\\\": \" intent \", \\\"category\\\": \" category \", \\\"account\\\": \" account \", \\\"merchant\\\": \" merchant \", \\\"period\\\": \" period \", \" amtop \"}}\"\n\
intent ::= {intent_alts}\n\
category ::= {category}\n\
account ::= {account}\n\
merchant ::= \"null\" | \"\\\"\" mchar{{1,40}} \"\\\"\"\n\
mchar ::= [^\"\\\\\\x00-\\x1f]\n\
period ::= \"null\" | {period_alts} | month\n\
month ::= \"\\\"\" [0-9] [0-9] [0-9] [0-9] \"-\" (\"0\" [1-9] | \"1\" [0-2]) \"\\\"\"\n\
amtop ::= \"\\\"amount\\\": null, \\\"operator\\\": null\" | \"\\\"amount\\\": \" amount \", \\\"operator\\\": \" opval\n\
amount ::= \"0\" | [1-9] [0-9]{{0,8}}\n\
opval ::= \"\\\"over\\\"\" | \"\\\"under\\\"\"\n",
        category = nullable_names(&categories),
        account = nullable_names(&accounts),
    ))
}

/// Build the router system prompt (Locked decision D5).
pub fn build_router_prompt(conn: &Connection, today: NaiveDate) -> AppResult<String> {
    let categories = active_names(conn, "categories")?;
    let accounts = active_names(conn, "accounts")?;

    let intents = INTENTS
        .iter()
        .map(|(name, desc, example)| format!("- {name}: {desc}. Example: \"{example}\""))
        .collect::<Vec<_>>()
        .join("\n");
    let periods = PERIODS.join(", ");
    let examples = EXAMPLES
        .iter()
        .map(|(q, json)| format!("Question: {q}\n{json}"))
        .collect::<Vec<_>>()
        .join("\n\n");

    Ok(format!(
        "You turn a question about personal finances into one JSON object.\n\
Today is {today} ({weekday}).\n\n\
Intents:\n{intents}\n- other: any question that none of the intents above can answer exactly\n\n\
Categories: {categories}\n\
Accounts: {accounts}\n\n\
Output keys, in this order: intent, category, account, merchant, period, amount, operator.\n\
- category: copy a name from the category list, or null.\n\
- account: set only when the question names an account; copy a name from the account list, or null.\n\
- merchant: set only when the question names a payee, written as in the question; otherwise null.\n\
- period: null unless the question names a time. Otherwise one of {periods}, or a month as YYYY-MM.\n\
- amount: null unless the question names a dollar figure. Otherwise a whole number of dollars.\n\
- operator: \"over\" or \"under\" when amount is set, otherwise null.\n\n\
Examples:\n\n{examples}\n\n\
Answer \"other\" when no intent can answer the question exactly. Output the JSON object and nothing else.",
        today = today.format("%Y-%m-%d"),
        weekday = today.format("%A"),
        categories = categories.join(", "),
        accounts = accounts.join(", "),
    ))
}

/// Map router output to an `IntentMatch` (Locked decision D4).
/// Returns `None` (decline) when the output cannot be answered exactly.
pub fn to_intent_match(out: &RouterOutput, conn: &Connection, today: NaiveDate) -> Option<IntentMatch> {
    let intent = parse_intent(&out.intent)?;

    // D15: amount and operator come together or not at all. The grammar enforces this;
    // output from any other source that breaks it is declined.
    match (out.amount, out.operator.as_deref()) {
        (None, None) | (Some(0..), Some("over" | "under")) => {}
        _ => return None,
    }

    let (category_id, category_name) = match out.category.as_deref() {
        Some(name) => {
            let (id, canonical) = lookup_named(conn, "categories", name)?;
            (Some(id), Some(canonical))
        }
        None => (None, None),
    };
    if matches!(intent, SearchIntent::SpendingByCategory | SearchIntent::CategoryDetail) && category_id.is_none() {
        return None;
    }

    let (account_id, account_name) = match out.account.as_deref() {
        Some(name) => {
            let (id, canonical) = lookup_named(conn, "accounts", name)?;
            (Some(id), Some(canonical))
        }
        None => (None, None),
    };

    let merchant = match out.merchant.as_deref() {
        Some(m) => {
            if !payee_matches(conn, m) {
                return None;
            }
            Some(m.to_string())
        }
        None => None,
    };

    let date_period = match out.period.as_deref() {
        Some(p) => Some(resolve_period(p, today)?),
        None => None,
    };

    Some(IntentMatch {
        intent,
        confidence: ROUTER_CONFIDENCE,
        entities: ExtractedEntities {
            category_id,
            category_name,
            account_id,
            account_name,
            date_period,
            comparison_period: None,
            amount_threshold: out.amount.map(|n| n.saturating_mul(100)),
            amount_operator: out.operator.clone(),
            merchant,
            wants_percentage: false,
        },
    })
}

/// Grammar and system prompt for one routing call. Built under the DB lock so the
/// caller can drop the lock before generating.
pub(crate) fn router_inputs(conn: &Connection, today: NaiveDate) -> AppResult<(String, String)> {
    Ok((build_router_grammar(conn)?, build_router_prompt(conn, today)?))
}

/// Parse the model's raw JSON and map it. `None` (decline) on invalid JSON,
/// `other`, or any output `to_intent_match` rejects.
pub(crate) fn route_from_json(raw: &str, conn: &Connection, today: NaiveDate) -> Option<IntentMatch> {
    let out: RouterOutput = serde_json::from_str(raw.trim()).ok()?;
    to_intent_match(&out, conn, today)
}

/// Run the router end to end: build grammar and prompt, generate, map.
/// Holds `conn` for the whole generation, so callers that share the connection
/// (`ai_search`) use `router_inputs` + `complete_json` + `route_from_json` instead.
#[allow(dead_code)] // used by the router eval (task 1.6); ai_search must not hold the DB lock while generating
pub(crate) fn route(engine: &Engine, conn: &Connection, query: &str, today: NaiveDate) -> AppResult<Option<IntentMatch>> {
    let (grammar, prompt) = router_inputs(conn, today)?;
    let raw = engine.complete_json(&prompt, query, &grammar)?;
    Ok(route_from_json(&raw, conn, today))
}

/// Card type for a routed intent; the same pairing the Tier-1 templates use.
pub(crate) fn card_type_for(intent: &SearchIntent) -> CardType {
    match intent {
        SearchIntent::SpendingTotal | SearchIntent::SpendingByCategory | SearchIntent::IncomeTotal => CardType::Total,
        SearchIntent::SpendingComparison => CardType::Comparison,
        SearchIntent::BudgetStatus => CardType::BudgetStatus,
        SearchIntent::TransactionSearch | SearchIntent::TopExpenses | SearchIntent::RecurringCheck => CardType::TransactionList,
        SearchIntent::TopMerchants | SearchIntent::CategoryDetail => CardType::Breakdown,
        SearchIntent::BalanceCheck | SearchIntent::NetWorth => CardType::AccountSummary,
    }
}

fn parse_intent(s: &str) -> Option<SearchIntent> {
    Some(match s {
        "spending_total" => SearchIntent::SpendingTotal,
        "spending_by_category" => SearchIntent::SpendingByCategory,
        "spending_comparison" => SearchIntent::SpendingComparison,
        "income_total" => SearchIntent::IncomeTotal,
        "balance_check" => SearchIntent::BalanceCheck,
        "budget_status" => SearchIntent::BudgetStatus,
        "transaction_search" => SearchIntent::TransactionSearch,
        "top_merchants" => SearchIntent::TopMerchants,
        "top_expenses" => SearchIntent::TopExpenses,
        "recurring_check" => SearchIntent::RecurringCheck,
        "net_worth" => SearchIntent::NetWorth,
        "category_detail" => SearchIntent::CategoryDetail,
        _ => return None, // includes "other"
    })
}

/// Look up a non-archived row by name. `table` is a fixed internal literal.
fn lookup_named(conn: &Connection, table: &str, name: &str) -> Option<(String, String)> {
    let sql = format!(
        "SELECT id, name FROM {table} WHERE is_archived = 0 AND name = ?1 COLLATE NOCASE LIMIT 1"
    );
    conn.query_row(&sql, [name], |r| Ok((r.get(0)?, r.get(1)?))).ok()
}

/// Same LIKE matching as the `TransactionSearch` builder uses on payee.
fn payee_matches(conn: &Connection, merchant: &str) -> bool {
    let lower = merchant.trim().to_lowercase();
    if lower.is_empty() {
        return false;
    }
    let stripped = strip_punctuation(&lower);
    conn.query_row(
        "SELECT 1 FROM transactions WHERE payee IS NOT NULL AND \
         (LOWER(payee) LIKE ?1 OR REPLACE(REPLACE(REPLACE(LOWER(payee), '''', ''), '.', ''), '-', '') LIKE ?2) \
         LIMIT 1",
        [format!("%{}%", lower), format!("%{}%", stripped)],
        |_| Ok(()),
    )
    .is_ok()
}

fn fmt(d: NaiveDate) -> String {
    d.format("%Y-%m-%d").to_string()
}

fn range(start: NaiveDate, end: NaiveDate, label: String) -> DatePeriod {
    DatePeriod { start_date: fmt(start), end_date: fmt(end), label }
}

/// Resolve a D2 period string with the same ranges and labels `parse_date_period` produces.
fn resolve_period(p: &str, today: NaiveDate) -> Option<DatePeriod> {
    let monday = today - Duration::days(today.weekday().num_days_from_monday() as i64);
    let rolling = |days: i64| range(today - Duration::days(days), today, format!("Last {} days", days));

    Some(match p {
        "today" => date_period_single_day(today, "Today"),
        "yesterday" => date_period_single_day(today - Duration::days(1), "Yesterday"),
        "this_week" => range(monday, today, "This week".into()),
        "last_week" => range(monday - Duration::days(7), monday - Duration::days(1), "Last week".into()),
        "this_month" => range(NaiveDate::from_ymd_opt(today.year(), today.month(), 1)?, today, "This month".into()),
        "last_month" => {
            let (y, m) = prev_month(today.year(), today.month());
            let this_first = NaiveDate::from_ymd_opt(today.year(), today.month(), 1)?;
            range(NaiveDate::from_ymd_opt(y, m, 1)?, this_first - Duration::days(1), "Last month".into())
        }
        "this_quarter" => {
            let q = (today.month() - 1) / 3;
            range(NaiveDate::from_ymd_opt(today.year(), q * 3 + 1, 1)?, today, format!("Q{} {}", q + 1, today.year()))
        }
        "this_year" => range(NaiveDate::from_ymd_opt(today.year(), 1, 1)?, today, format!("{}", today.year())),
        "last_year" => {
            let y = today.year() - 1;
            range(NaiveDate::from_ymd_opt(y, 1, 1)?, NaiveDate::from_ymd_opt(y, 12, 31)?, format!("{}", y))
        }
        "last_7_days" => rolling(7),
        "last_30_days" => rolling(30),
        "last_90_days" => rolling(90),
        _ => return resolve_month(p),
    })
}

/// `"YYYY-MM"` → that month's first to last day.
fn resolve_month(p: &str) -> Option<DatePeriod> {
    let (y, m) = p.split_once('-')?;
    if y.len() != 4 || m.len() != 2 || !y.bytes().all(|b| b.is_ascii_digit()) || !m.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let (y, m): (i32, u32) = (y.parse().ok()?, m.parse().ok()?);
    let start = NaiveDate::from_ymd_opt(y, m, 1)?;
    let end = end_of_month(y, m)?;
    Some(range(start, end, format!("{} {}", start.format("%B"), y)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seeded() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run_migrations(&conn).unwrap();
        crate::db::migrations::seed_test_data(&conn).unwrap();
        conn
    }

    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 3, 31).unwrap() // a Tuesday
    }

    fn out(intent: &str) -> RouterOutput {
        RouterOutput {
            intent: intent.into(),
            category: None,
            account: None,
            merchant: None,
            period: None,
            amount: None,
            operator: None,
        }
    }

    fn period_of(p: &str) -> Option<(String, String, String)> {
        let conn = seeded();
        let mut o = out("spending_total");
        o.period = Some(p.into());
        to_intent_match(&o, &conn, today())
            .map(|m| m.entities.date_period.unwrap())
            .map(|d| (d.start_date, d.end_date, d.label))
    }

    fn check(p: &str, start: &str, end: &str, label: &str) {
        assert_eq!(period_of(p), Some((start.into(), end.into(), label.into())), "period {p}");
    }

    #[test]
    fn every_period_value_resolves() {
        check("today", "2026-03-31", "2026-03-31", "Today");
        check("yesterday", "2026-03-30", "2026-03-30", "Yesterday");
        check("this_week", "2026-03-30", "2026-03-31", "This week");
        check("last_week", "2026-03-23", "2026-03-29", "Last week");
        check("this_month", "2026-03-01", "2026-03-31", "This month");
        check("last_month", "2026-02-01", "2026-02-28", "Last month");
        check("this_quarter", "2026-01-01", "2026-03-31", "Q1 2026");
        check("this_year", "2026-01-01", "2026-03-31", "2026");
        check("last_year", "2025-01-01", "2025-12-31", "2025");
        check("last_7_days", "2026-03-24", "2026-03-31", "Last 7 days");
        check("last_30_days", "2026-03-01", "2026-03-31", "Last 30 days");
        check("last_90_days", "2025-12-31", "2026-03-31", "Last 90 days");
        check("2026-02", "2026-02-01", "2026-02-28", "February 2026");
        check("2025-12", "2025-12-01", "2025-12-31", "December 2025");
        check("2024-02", "2024-02-01", "2024-02-29", "February 2024");
    }

    #[test]
    fn last_month_crosses_year_boundary() {
        let conn = seeded();
        let mut o = out("spending_total");
        o.period = Some("last_month".into());
        let jan = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let d = to_intent_match(&o, &conn, jan).unwrap().entities.date_period.unwrap();
        assert_eq!((d.start_date.as_str(), d.end_date.as_str()), ("2025-12-01", "2025-12-31"));
    }

    #[test]
    fn bad_period_declines() {
        assert!(period_of("next_week").is_none());
        assert!(period_of("2026-13").is_none());
        assert!(period_of("2026-3").is_none());
        assert!(period_of("").is_none());
    }

    #[test]
    fn null_period_is_none() {
        let conn = seeded();
        let m = to_intent_match(&out("spending_total"), &conn, today()).unwrap();
        assert!(m.entities.date_period.is_none());
        assert!(m.entities.comparison_period.is_none());
        assert_eq!(m.confidence, 0.8);
        assert_eq!(m.intent, SearchIntent::SpendingTotal);
    }

    #[test]
    fn other_declines() {
        let conn = seeded();
        assert!(to_intent_match(&out("other"), &conn, today()).is_none());
        assert!(to_intent_match(&out("nonsense"), &conn, today()).is_none());
    }

    #[test]
    fn all_intents_map() {
        let conn = seeded();
        let cases = [
            ("spending_total", SearchIntent::SpendingTotal),
            ("spending_comparison", SearchIntent::SpendingComparison),
            ("income_total", SearchIntent::IncomeTotal),
            ("balance_check", SearchIntent::BalanceCheck),
            ("budget_status", SearchIntent::BudgetStatus),
            ("transaction_search", SearchIntent::TransactionSearch),
            ("top_merchants", SearchIntent::TopMerchants),
            ("top_expenses", SearchIntent::TopExpenses),
            ("recurring_check", SearchIntent::RecurringCheck),
            ("net_worth", SearchIntent::NetWorth),
        ];
        for (name, intent) in cases {
            assert_eq!(to_intent_match(&out(name), &conn, today()).unwrap().intent, intent, "{name}");
        }
    }

    #[test]
    fn category_intents_need_a_category() {
        let conn = seeded();
        assert!(to_intent_match(&out("spending_by_category"), &conn, today()).is_none());
        assert!(to_intent_match(&out("category_detail"), &conn, today()).is_none());

        let mut o = out("spending_by_category");
        o.category = Some("Food".into());
        let m = to_intent_match(&o, &conn, today()).unwrap();
        assert_eq!(m.intent, SearchIntent::SpendingByCategory);
        assert_eq!(m.entities.category_name.as_deref(), Some("Food"));
        let food_id: String = conn
            .query_row("SELECT id FROM categories WHERE name = 'Food'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(m.entities.category_id.as_deref(), Some(food_id.as_str()));

        o.intent = "category_detail".into();
        assert!(to_intent_match(&o, &conn, today()).is_some());
    }

    #[test]
    fn unknown_category_or_account_declines() {
        let conn = seeded();
        let mut o = out("spending_total");
        o.category = Some("Nonexistent".into());
        assert!(to_intent_match(&o, &conn, today()).is_none());
        let mut o = out("balance_check");
        o.account = Some("Nonexistent".into());
        assert!(to_intent_match(&o, &conn, today()).is_none());
    }

    #[test]
    fn account_resolves_to_id() {
        let conn = seeded();
        let mut o = out("balance_check");
        o.account = Some("Primary Checking".into());
        let m = to_intent_match(&o, &conn, today()).unwrap();
        let id: String = conn
            .query_row("SELECT id FROM accounts WHERE name = 'Primary Checking'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(m.entities.account_id.as_deref(), Some(id.as_str()));
        assert_eq!(m.entities.account_name.as_deref(), Some("Primary Checking"));
    }

    #[test]
    fn unknown_merchant_declines() {
        let conn = seeded();
        let mut o = out("transaction_search");
        o.merchant = Some("Starbucks".into());
        assert!(to_intent_match(&o, &conn, today()).is_none());
        o.merchant = Some("".into());
        assert!(to_intent_match(&o, &conn, today()).is_none());
    }

    #[test]
    fn known_merchant_matches_like_builder() {
        let conn = seeded();
        let mut o = out("transaction_search");
        o.merchant = Some("whole foods".into());
        let m = to_intent_match(&o, &conn, today()).unwrap();
        assert_eq!(m.entities.merchant.as_deref(), Some("whole foods"));

        // punctuation-insensitive, like the builder
        o.merchant = Some("Trader Joes".into());
        assert!(to_intent_match(&o, &conn, today()).is_some());
        o.merchant = Some("Trader Joe's".into());
        assert!(to_intent_match(&o, &conn, today()).is_some());
    }

    #[test]
    fn amount_becomes_cents() {
        let conn = seeded();
        let mut o = out("transaction_search");
        o.amount = Some(20);
        o.operator = Some("over".into());
        let m = to_intent_match(&o, &conn, today()).unwrap();
        assert_eq!(m.entities.amount_threshold, Some(2000));
        assert_eq!(m.entities.amount_operator.as_deref(), Some("over"));

        let m = to_intent_match(&out("transaction_search"), &conn, today()).unwrap();
        assert_eq!(m.entities.amount_threshold, None);
        assert_eq!(m.entities.amount_operator, None);
    }

    #[test]
    fn amount_and_operator_must_be_paired() {
        let conn = seeded();
        let mut o = out("transaction_search");
        o.operator = Some("over".into());
        assert!(to_intent_match(&o, &conn, today()).is_none());
        o.operator = None;
        o.amount = Some(20);
        assert!(to_intent_match(&o, &conn, today()).is_none());
        o.operator = Some("about".into());
        assert!(to_intent_match(&o, &conn, today()).is_none());
        o.operator = Some("under".into());
        assert!(to_intent_match(&o, &conn, today()).is_some());
        o.amount = Some(0);
        assert!(to_intent_match(&o, &conn, today()).is_some());
    }

    #[test]
    fn route_from_json_declines_unpaired_amount_operator() {
        let conn = seeded();
        let raw = r#"{"intent": "transaction_search", "category": null, "account": null, "merchant": null, "period": null, "amount": null, "operator": "over"}"#;
        assert!(route_from_json(raw, &conn, today()).is_none());
        let raw = r#"{"intent": "transaction_search", "category": null, "account": null, "merchant": null, "period": null, "amount": 20, "operator": null}"#;
        assert!(route_from_json(raw, &conn, today()).is_none());
    }

    /// The body of grammar rule `name`, without the `name ::= ` prefix.
    fn rule<'a>(g: &'a str, name: &str) -> &'a str {
        let prefix = format!("{name} ::= ");
        g.lines()
            .find_map(|l| l.strip_prefix(prefix.as_str()))
            .unwrap_or_else(|| panic!("no rule {name}:\n{g}"))
    }

    #[test]
    fn grammar_has_no_operator_without_amount() {
        let conn = seeded();
        let g = build_router_grammar(&conn).unwrap();

        // Neither key is reachable from root; both live only in `amtop`.
        let root = rule(&g, "root");
        assert!(!root.contains("amount\\\"") && !root.contains("operator\\\""), "root: {root}");
        assert!(root.contains(" amtop "));
        let holders: Vec<&str> = g
            .lines()
            .filter(|l| l.contains(r#"\"operator\""#) && !l.starts_with("opval ::="))
            .collect();
        assert_eq!(holders.len(), 1, "{holders:?}");
        assert!(holders[0].starts_with("amtop ::= "));

        // Exactly two alternatives: both null, or amount value then an operator value.
        let alts: Vec<&str> = rule(&g, "amtop").split(" | ").collect();
        assert_eq!(alts.len(), 2, "{alts:?}");
        assert_eq!(alts[0], r#""\"amount\": null, \"operator\": null""#);
        assert_eq!(alts[1], r#""\"amount\": " amount ", \"operator\": " opval"#);

        // The operator value rule has no null; the amount rule has no null either.
        assert_eq!(rule(&g, "opval"), r#""\"over\"" | "\"under\"""#);
        assert!(!rule(&g, "opval").contains("null"));
        assert!(!rule(&g, "amount").contains("null"));
        // The old independent rules are gone.
        assert!(!g.lines().any(|l| l.starts_with("operator ::=")));
    }

    #[test]
    fn grammar_amount_rejects_leading_zeros() {
        let conn = seeded();
        let g = build_router_grammar(&conn).unwrap();
        assert_eq!(rule(&g, "amount"), r#""0" | [1-9] [0-9]{0,8}"#);

        // The rule's language, as a regex: `0` or a non-zero digit plus up to 8 digits.
        let re = regex::Regex::new(r"^(0|[1-9][0-9]{0,8})$").unwrap();
        for ok in ["0", "7", "20", "100", "999999999"] {
            assert!(re.is_match(ok), "{ok}");
        }
        for bad in ["00", "01", "007", "0123", "1000000000", "", "-5", "1.5"] {
            assert!(!re.is_match(bad), "{bad}");
        }
    }

    #[test]
    fn grammar_escapes_quotes_and_backslashes() {
        let conn = seeded();
        conn.execute(
            "INSERT INTO categories (id, name, type, created_at, updated_at) VALUES ('c-kid', ?1, 'expense', '2026-01-01', '2026-01-01')",
            [r#"Kid's "Fun" \ Money"#],
        )
        .unwrap();
        let g = build_router_grammar(&conn).unwrap();
        // JSON text is "Kid's \"Fun\" \\ Money"; GBNF escapes it once more.
        let expected = r#""\"Kid's \\\"Fun\\\" \\\\ Money\"""#;
        assert!(g.contains(expected), "grammar missing {expected}:\n{g}");
    }

    #[test]
    fn grammar_has_fixed_shape() {
        let conn = seeded();
        let g = build_router_grammar(&conn).unwrap();
        assert!(g.starts_with("root ::= "));
        let keys = ["intent", "category", "account", "merchant", "period", "amount", "operator"];
        let mut last = 0;
        for k in keys {
            let pos = g.find(&format!(r#"\"{k}\": "#)).unwrap_or_else(|| panic!("missing key {k}"));
            assert!(pos >= last, "key {k} out of order");
            last = pos;
        }
        assert!(g.contains(r#""\"Food\"""#));
        assert!(g.contains(r#""\"Primary Checking\"""#));
        for p in PERIODS {
            assert!(g.contains(&format!(r#""\"{p}\"""#)), "{p}");
        }
    }

    #[test]
    fn archived_names_are_excluded() {
        let conn = seeded();
        conn.execute("UPDATE categories SET is_archived = 1 WHERE name = 'Food'", []).unwrap();
        let g = build_router_grammar(&conn).unwrap();
        assert!(!g.contains(r#"\"Food\""#));
        let p = build_router_prompt(&conn, today()).unwrap();
        assert!(!p.contains("Food"));
    }

    #[test]
    fn every_intent_in_grammar_and_prompt() {
        let conn = seeded();
        let g = build_router_grammar(&conn).unwrap();
        let p = build_router_prompt(&conn, today()).unwrap();
        let names = [
            "spending_total", "spending_by_category", "spending_comparison", "income_total",
            "balance_check", "budget_status", "transaction_search", "top_merchants",
            "top_expenses", "recurring_check", "net_worth", "category_detail", "other",
        ];
        for n in names {
            assert!(g.contains(&format!(r#""\"{n}\"""#)), "grammar missing {n}");
            assert!(p.contains(&format!("- {n}:")), "prompt missing {n}");
        }
        // every intent the mapper accepts is listed
        for (n, _, _) in INTENTS {
            assert!(parse_intent(n).is_some(), "{n}");
        }
    }

    #[test]
    fn prompt_has_date_names_and_instruction() {
        let conn = seeded();
        let p = build_router_prompt(&conn, today()).unwrap();
        assert!(p.contains("2026-03-31"));
        assert!(p.contains("Food"));
        assert!(p.contains("Primary Checking"));
        assert!(p.contains(r#"Answer "other" when no intent can answer the question exactly"#));
    }

    /// Lowercased, trimmed fixture questions.
    fn fixture_questions() -> Vec<String> {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/ai-router-eval.json");
        let raw = std::fs::read_to_string(path).unwrap();
        let items: Vec<serde_json::Value> = serde_json::from_str(&raw).unwrap();
        items
            .iter()
            .map(|i| i["question"].as_str().unwrap().trim().to_lowercase())
            .collect()
    }

    #[test]
    fn router_prompt_examples_not_in_fixture() {
        let conn = seeded();
        let p = build_router_prompt(&conn, today()).unwrap();
        let fixture = fixture_questions();
        assert!(!fixture.is_empty());

        let questions = EXAMPLES.iter().map(|(q, _)| *q).chain(INTENTS.iter().map(|(_, _, q)| *q));
        for q in questions {
            assert!(p.contains(q), "prompt missing example {q}");
            assert!(!fixture.contains(&q.trim().to_lowercase()), "example is in the fixture: {q}");
        }
        let lower = p.to_lowercase();
        assert!(!lower.contains("stand right now"));
        assert!(!lower.contains("leave the most"));
    }

    #[test]
    fn router_prompt_examples_are_valid() {
        assert!((10..=14).contains(&EXAMPLES.len()));
        assert!(EXAMPLES.iter().filter(|(_, j)| j.contains(r#""period": null"#)).count() >= 4);
        for (q, json) in EXAMPLES {
            let o: RouterOutput = serde_json::from_str(json).unwrap_or_else(|e| panic!("{q}: {e}"));
            assert!(parse_intent(&o.intent).is_some() || o.intent == "other", "{q}");
            assert_eq!(o.amount.is_some(), o.operator.is_some(), "{q}");
        }
    }

    #[test]
    fn deserializes_d2_json() {
        let raw = r#"{"intent": "transaction_search", "category": null, "account": null, "merchant": "Whole Foods", "period": "2026-02", "amount": 20, "operator": "over"}"#;
        let o: RouterOutput = serde_json::from_str(raw).unwrap();
        assert_eq!(o.intent, "transaction_search");
        assert_eq!(o.merchant.as_deref(), Some("Whole Foods"));
        assert_eq!(o.amount, Some(20));
    }

    #[test]
    fn route_from_json_valid() {
        let conn = seeded();
        let raw = r#"{"intent": "transaction_search", "category": null, "account": null, "merchant": "Whole Foods", "period": "last_month", "amount": 20, "operator": "over"}"#;
        let m = route_from_json(raw, &conn, today()).unwrap();
        assert_eq!(m.intent, SearchIntent::TransactionSearch);
        assert_eq!(m.entities.merchant.as_deref(), Some("Whole Foods"));
        assert_eq!(m.entities.amount_threshold, Some(2000));
        assert_eq!(m.entities.amount_operator.as_deref(), Some("over"));
        assert_eq!(m.entities.date_period.unwrap().start_date, "2026-02-01");
        // surrounding whitespace is tolerated
        assert!(route_from_json(&format!("  {raw}\n"), &conn, today()).is_some());
    }

    #[test]
    fn route_from_json_invalid() {
        let conn = seeded();
        assert!(route_from_json("", &conn, today()).is_none());
        assert!(route_from_json("not json", &conn, today()).is_none());
        assert!(route_from_json(r#"{"intent": "spending_total""#, &conn, today()).is_none());
        assert!(route_from_json(r#"{"intent": 5, "category": null, "account": null, "merchant": null, "period": null, "amount": null, "operator": null}"#, &conn, today()).is_none());
    }

    #[test]
    fn route_from_json_other_declines() {
        let conn = seeded();
        let raw = r#"{"intent": "other", "category": null, "account": null, "merchant": null, "period": null, "amount": null, "operator": null}"#;
        assert!(route_from_json(raw, &conn, today()).is_none());
    }

    #[test]
    fn card_type_matches_template_pairing() {
        assert_eq!(card_type_for(&SearchIntent::IncomeTotal), CardType::Total);
        assert_eq!(card_type_for(&SearchIntent::TopMerchants), CardType::Breakdown);
        assert_eq!(card_type_for(&SearchIntent::NetWorth), CardType::AccountSummary);
        assert_eq!(card_type_for(&SearchIntent::RecurringCheck), CardType::TransactionList);
    }
}
