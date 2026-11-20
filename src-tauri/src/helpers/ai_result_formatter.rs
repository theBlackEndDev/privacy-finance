use rusqlite::Connection;
use crate::error::AppResult;
use crate::models::ai_search::*;
use crate::helpers::ai_sql_builder::BuiltQuery;

pub fn execute_and_format(
    queries: &[BuiltQuery],
    intent_match: &IntentMatch,
    conn: &Connection,
) -> AppResult<(CardData, String)> {
    match intent_match.intent {
        SearchIntent::SpendingTotal | SearchIntent::SpendingByCategory => {
            format_total_card(queries, intent_match, conn)
        }
        SearchIntent::CategoryDetail => {
            format_breakdown_card(queries, intent_match, conn)
        }
        SearchIntent::SpendingComparison => {
            format_comparison_card(queries, intent_match, conn)
        }
        SearchIntent::IncomeTotal => {
            format_income_card(queries, intent_match, conn)
        }
        SearchIntent::BalanceCheck | SearchIntent::NetWorth => {
            format_account_summary(queries, intent_match, conn)
        }
        SearchIntent::BudgetStatus => {
            format_budget_card(queries, intent_match, conn)
        }
        SearchIntent::TransactionSearch | SearchIntent::TopExpenses => {
            format_transaction_list(queries, intent_match, conn)
        }
        SearchIntent::TopMerchants => {
            format_breakdown_card(queries, intent_match, conn)
        }
        SearchIntent::RecurringCheck => {
            format_recurring_list(queries, intent_match, conn)
        }
    }
}

fn format_total_card(queries: &[BuiltQuery], im: &IntentMatch, conn: &Connection) -> AppResult<(CardData, String)> {
    let q = &queries[0];
    let (total, count): (i64, i64) = execute_row2(conn, &q.sql, &q.params)?;

    let period_label = im.entities.date_period.as_ref()
        .map(|dp| dp.label.clone())
        .unwrap_or_else(|| "All time".to_string());

    let label = if let Some(ref cat) = im.entities.category_name {
        cat.clone()
    } else {
        "Total Spending".to_string()
    };

    let mut top_items = vec![];
    if queries.len() > 1 {
        let q2 = &queries[1];
        top_items = execute_breakdown(conn, &q2.sql, &q2.params)?;
    }

    // Check for grand-total query (appended when wants_percentage + category)
    let pct_text = if im.entities.wants_percentage {
        // Grand total query is the last one (index 2 or 3 depending on breakdown)
        let last = &queries[queries.len() - 1];
        if let Ok(grand_total) = execute_single_i64(conn, &last.sql, &last.params) {
            if grand_total > 0 {
                let pct = (total as f64 / grand_total as f64) * 100.0;
                Some((pct, grand_total))
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let text = if let Some((pct, grand_total)) = pct_text {
        format!(
            "{} is {:.1}% of your total spending ({}) {} ({} transactions)",
            label,
            pct,
            format_dollars(grand_total),
            if period_label != "All time" { period_label.to_lowercase() } else { String::new() },
            count
        )
    } else {
        format!(
            "You spent {} on {}{} ({} transactions)",
            format_dollars(total),
            label.to_lowercase(),
            if period_label != "All time" { format!(" {}", period_label.to_lowercase()) } else { String::new() },
            count
        )
    };

    Ok((CardData::Total {
        label,
        amount: total,
        count,
        period_label,
        comparison_amount: None,
        comparison_count: None,
        change_pct: None,
        top_items,
    }, text))
}

fn format_income_card(queries: &[BuiltQuery], im: &IntentMatch, conn: &Connection) -> AppResult<(CardData, String)> {
    let q = &queries[0];
    let (total, count): (i64, i64) = execute_row2(conn, &q.sql, &q.params)?;

    let period_label = im.entities.date_period.as_ref()
        .map(|dp| dp.label.clone())
        .unwrap_or_else(|| "All time".to_string());

    let text = format!("You earned {} {} ({} transactions)", format_dollars(total), period_label.to_lowercase(), count);

    Ok((CardData::Total {
        label: "Total Income".to_string(),
        amount: total,
        count,
        period_label,
        comparison_amount: None,
        comparison_count: None,
        change_pct: None,
        top_items: vec![],
    }, text))
}

fn format_comparison_card(queries: &[BuiltQuery], im: &IntentMatch, conn: &Connection) -> AppResult<(CardData, String)> {
    let current: i64 = if !queries.is_empty() {
        execute_single_i64(conn, &queries[0].sql, &queries[0].params)?
    } else { 0 };

    let previous: i64 = if queries.len() > 1 {
        execute_single_i64(conn, &queries[1].sql, &queries[1].params)?
    } else { 0 };

    let change = current - previous;
    let change_pct = if previous > 0 { (change as f64 / previous as f64) * 100.0 } else { 0.0 };

    let current_label = im.entities.date_period.as_ref()
        .map(|dp| dp.label.clone())
        .unwrap_or_else(|| "This month".to_string());

    let previous_label = im.entities.comparison_period.as_ref()
        .map(|dp| dp.label.clone())
        .unwrap_or_else(|| "Last month".to_string());

    let direction = if change > 0 { "+" } else { "" };
    let text = format!(
        "{}: {} | {}: {} | {}{} ({}{:.1}%)",
        current_label, format_dollars(current),
        previous_label, format_dollars(previous),
        direction, format_dollars(change.abs()),
        direction, change_pct
    );

    Ok((CardData::Comparison {
        label: "Spending Comparison".to_string(),
        current_label,
        current_amount: current,
        previous_label,
        previous_amount: previous,
        change_amount: change,
        change_pct,
    }, text))
}

fn format_account_summary(queries: &[BuiltQuery], _im: &IntentMatch, conn: &Connection) -> AppResult<(CardData, String)> {
    let q = &queries[0];
    let params_ref: Vec<&dyn rusqlite::types::ToSql> = q.params.iter().map(|p| p as &dyn rusqlite::types::ToSql).collect();
    let mut stmt = conn.prepare(&q.sql)?;
    let accounts: Vec<AccountItem> = stmt.query_map(params_ref.as_slice(), |row| {
        Ok(AccountItem {
            name: row.get(1)?,
            account_type: row.get(2)?,
            balance: row.get(3)?,
        })
    })?.filter_map(|r| r.ok()).collect();

    let net_worth: i64 = accounts.iter().map(|a| a.balance).sum();
    let text = format!("Total: {} across {} accounts", format_dollars(net_worth), accounts.len());

    Ok((CardData::AccountSummary { accounts, net_worth }, text))
}

fn format_budget_card(queries: &[BuiltQuery], im: &IntentMatch, conn: &Connection) -> AppResult<(CardData, String)> {
    let q = &queries[0];
    let params_ref: Vec<&dyn rusqlite::types::ToSql> = q.params.iter().map(|p| p as &dyn rusqlite::types::ToSql).collect();
    let mut stmt = conn.prepare(&q.sql)?;
    let items: Vec<BudgetStatusItem> = stmt.query_map(params_ref.as_slice(), |row| {
        let budgeted: i64 = row.get(2)?;
        let spent: i64 = row.get(3)?;
        Ok(BudgetStatusItem {
            category_name: row.get(0)?,
            color: row.get::<_, Option<String>>(1)?.unwrap_or_else(|| "#888".to_string()),
            budgeted,
            spent,
            available: budgeted - spent,
        })
    })?.filter_map(|r| r.ok()).collect();

    let period_label = im.entities.date_period.as_ref()
        .map(|dp| dp.label.clone())
        .unwrap_or_else(|| "This month".to_string());

    let over_budget: Vec<&BudgetStatusItem> = items.iter().filter(|i| i.spent > i.budgeted).collect();
    let text = if over_budget.is_empty() {
        format!("All {} categories are within budget for {}", items.len(), period_label.to_lowercase())
    } else {
        let names: Vec<String> = over_budget.iter().map(|i| i.category_name.clone()).collect();
        format!("Over budget on: {} ({})", names.join(", "), period_label.to_lowercase())
    };

    Ok((CardData::BudgetStatus { items, period_label }, text))
}

fn format_transaction_list(queries: &[BuiltQuery], im: &IntentMatch, conn: &Connection) -> AppResult<(CardData, String)> {
    let q = &queries[0];
    let params_ref: Vec<&dyn rusqlite::types::ToSql> = q.params.iter().map(|p| p as &dyn rusqlite::types::ToSql).collect();
    let mut stmt = conn.prepare(&q.sql)?;
    let transactions: Vec<TransactionItem> = stmt.query_map(params_ref.as_slice(), |row| {
        Ok(TransactionItem {
            id: row.get(0)?,
            date: row.get(1)?,
            payee: row.get(2)?,
            amount: row.get(3)?,
            kind: row.get(4)?,
            category_name: row.get(5)?,
        })
    })?.filter_map(|r| r.ok()).collect();

    let total: i64 = transactions.iter().map(|t| t.amount.abs()).sum();
    let count = transactions.len() as i64;

    let label = if let Some(ref m) = im.entities.merchant {
        format!("Transactions at {}", m)
    } else if let Some(ref cat) = im.entities.category_name {
        format!("{} transactions", cat)
    } else if im.intent == SearchIntent::TopExpenses {
        "Largest transactions".to_string()
    } else {
        "Matching transactions".to_string()
    };

    let text = format!("{} transactions totaling {}", count, format_dollars(total));

    Ok((CardData::TransactionList { transactions, total, count, label }, text))
}

fn format_recurring_list(queries: &[BuiltQuery], _im: &IntentMatch, conn: &Connection) -> AppResult<(CardData, String)> {
    let q = &queries[0];
    let mut stmt = conn.prepare(&q.sql)?;
    let transactions: Vec<TransactionItem> = stmt.query_map([], |row| {
        Ok(TransactionItem {
            id: row.get(0)?,
            date: row.get::<_, Option<String>>(4)?.unwrap_or_default(),
            payee: row.get(1)?,
            amount: row.get(2)?,
            kind: "recurring".to_string(),
            category_name: row.get::<_, Option<String>>(3)?,
        })
    })?.filter_map(|r| r.ok()).collect();

    let monthly_total: i64 = transactions.iter().map(|t| t.amount.abs()).sum();
    let count = transactions.len() as i64;
    let text = format!("{} active subscriptions totaling {}/month", count, format_dollars(monthly_total));

    Ok((CardData::TransactionList {
        transactions,
        total: monthly_total,
        count,
        label: "Active Subscriptions".to_string(),
    }, text))
}

fn format_breakdown_card(queries: &[BuiltQuery], im: &IntentMatch, conn: &Connection) -> AppResult<(CardData, String)> {
    let q = if queries.len() > 2 { &queries[2] } else { &queries[0] };
    let items = execute_breakdown(conn, &q.sql, &q.params)?;
    let total: i64 = items.iter().map(|i| i.amount).sum();

    let label = if let Some(ref cat) = im.entities.category_name {
        format!("{} — By merchant", cat)
    } else {
        "Spending by category".to_string()
    };

    let text = if let Some(first) = items.first() {
        format!("Top: {} ({})", first.name, format_dollars(first.amount))
    } else {
        "No spending data found".to_string()
    };

    Ok((CardData::Breakdown { label, items, total }, text))
}

// ── Helpers ───────────────────────────────────────────────────────

fn execute_row2(conn: &Connection, sql: &str, params: &[String]) -> AppResult<(i64, i64)> {
    let params_ref: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p as &dyn rusqlite::types::ToSql).collect();
    Ok(conn.query_row(sql, params_ref.as_slice(), |row| {
        Ok((row.get(0)?, row.get(1)?))
    }).unwrap_or((0, 0)))
}

fn execute_single_i64(conn: &Connection, sql: &str, params: &[String]) -> AppResult<i64> {
    let params_ref: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p as &dyn rusqlite::types::ToSql).collect();
    Ok(conn.query_row(sql, params_ref.as_slice(), |row| row.get(0)).unwrap_or(0))
}

fn execute_breakdown(conn: &Connection, sql: &str, params: &[String]) -> AppResult<Vec<BreakdownItem>> {
    let params_ref: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p as &dyn rusqlite::types::ToSql).collect();
    let mut stmt = conn.prepare(sql)?;
    let rows: Vec<(String, Option<String>, i64)> = stmt.query_map(params_ref.as_slice(), |row| {
        Ok((
            row.get::<_, Option<String>>(0)?.unwrap_or_else(|| "Uncategorized".to_string()),
            row.get(1)?,
            row.get(2)?,
        ))
    })?.filter_map(|r| r.ok()).collect();

    let total: i64 = rows.iter().map(|(_, _, a)| *a).sum();
    let items: Vec<BreakdownItem> = rows.into_iter().map(|(name, color, amount)| {
        BreakdownItem {
            name,
            color,
            amount,
            percentage: if total > 0 { (amount as f64 / total as f64) * 100.0 } else { 0.0 },
        }
    }).collect();

    Ok(items)
}

fn format_dollars(cents: i64) -> String {
    let dollars = cents.abs() as f64 / 100.0;
    if cents < 0 {
        format!("-${:.2}", dollars)
    } else {
        format!("${:.2}", dollars)
    }
}

/// Plain-English line saying what the answer's SQL actually computes, built from the
/// `IntentMatch`. Mirrors `ai_sql_builder::build_query`, including which slots it ignores.
pub fn describe_intent(im: &IntentMatch) -> String {
    let e = &im.entities;
    let period = e.date_period.as_ref().map(|dp| period_phrase(&dp.label));
    let with_period = |base: String| match &period {
        Some(p) => format!("{base}, {p}"),
        None => base,
    };

    match im.intent {
        SearchIntent::SpendingTotal | SearchIntent::SpendingByCategory | SearchIntent::CategoryDetail => {
            match &e.category_name {
                Some(cat) => with_period(format!("Spending in {cat}")),
                None => with_period("Total spending".to_string()),
            }
        }
        SearchIntent::SpendingComparison => {
            let current = period.clone().unwrap_or_else(|| "this month".to_string());
            let previous = e.comparison_period.as_ref()
                .map(|cp| period_phrase(&cp.label))
                .unwrap_or_else(|| "last month".to_string());
            format!("Spending, {current} vs {previous}")
        }
        SearchIntent::IncomeTotal => with_period("Income".to_string()),
        SearchIntent::BalanceCheck => "Current balance of every account".to_string(),
        SearchIntent::NetWorth => "Net worth across all accounts".to_string(),
        SearchIntent::BudgetStatus => {
            let month = e.date_period.as_ref()
                .and_then(|dp| dp.start_date.get(..7))
                .map(|m| m.to_string());
            match month {
                Some(m) => format!("Budget status for {m}"),
                None => "Budget status for this month".to_string(),
            }
        }
        SearchIntent::TransactionSearch => {
            let mut s = String::from("Transactions");
            if let Some(m) = &e.merchant {
                s.push_str(&format!(" at {m}"));
            }
            if let Some(cat) = &e.category_name {
                s.push_str(&format!(" in {cat}"));
            }
            if let Some(t) = e.amount_threshold {
                let op = if e.amount_operator.as_deref() == Some("under") { "under" } else { "over" };
                s.push_str(&format!(" {op} {}", whole_dollars(t)));
            }
            with_period(s)
        }
        // Without a category the builder groups spending by category; with one it lists payees.
        SearchIntent::TopMerchants => match &e.category_name {
            None => with_period("Top spending categories".to_string()),
            Some(_) => with_period("Top payees by spending".to_string()),
        },
        SearchIntent::TopExpenses => with_period("Largest expenses".to_string()),
        SearchIntent::RecurringCheck => "Active recurring transactions".to_string(),
    }
}

/// "Last month" reads as "last month" mid-sentence; "Q1 2026" and "March 2026" stay as they are.
fn period_phrase(label: &str) -> String {
    let lower_first = ["This ", "Last ", "Today", "Yesterday"]
        .iter()
        .any(|p| label.starts_with(p));
    if lower_first {
        let mut chars = label.chars();
        match chars.next() {
            Some(c) => c.to_lowercase().collect::<String>() + chars.as_str(),
            None => String::new(),
        }
    } else {
        label.to_string()
    }
}

fn whole_dollars(cents: i64) -> String {
    if cents % 100 == 0 {
        format!("${}", cents / 100)
    } else {
        format!("${:.2}", cents as f64 / 100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn im(intent: SearchIntent, f: impl FnOnce(&mut ExtractedEntities)) -> IntentMatch {
        let mut entities = ExtractedEntities::default();
        f(&mut entities);
        IntentMatch { intent, confidence: 0.8, entities }
    }

    fn period(label: &str, start: &str) -> Option<DatePeriod> {
        Some(DatePeriod { start_date: start.to_string(), end_date: start.to_string(), label: label.to_string() })
    }

    #[test]
    fn describe_spending_total() {
        assert_eq!(describe_intent(&im(SearchIntent::SpendingTotal, |_| {})), "Total spending");
        let m = im(SearchIntent::SpendingTotal, |e| e.date_period = period("This year", "2026-01-01"));
        assert_eq!(describe_intent(&m), "Total spending, this year");
        let m = im(SearchIntent::SpendingTotal, |e| e.category_name = Some("Food".into()));
        assert_eq!(describe_intent(&m), "Spending in Food");
    }

    #[test]
    fn describe_spending_by_category() {
        let m = im(SearchIntent::SpendingByCategory, |e| {
            e.category_name = Some("Food".into());
            e.date_period = period("Last month", "2026-02-01");
        });
        assert_eq!(describe_intent(&m), "Spending in Food, last month");
    }

    #[test]
    fn describe_category_detail() {
        let m = im(SearchIntent::CategoryDetail, |e| {
            e.category_name = Some("Dining".into());
            e.date_period = period("Q1 2026", "2026-01-01");
        });
        assert_eq!(describe_intent(&m), "Spending in Dining, Q1 2026");
        assert_eq!(describe_intent(&im(SearchIntent::CategoryDetail, |_| {})), "Total spending");
    }

    #[test]
    fn describe_spending_comparison() {
        assert_eq!(describe_intent(&im(SearchIntent::SpendingComparison, |_| {})), "Spending, this month vs last month");
        let m = im(SearchIntent::SpendingComparison, |e| {
            e.date_period = period("March 2026", "2026-03-01");
            e.comparison_period = period("February 2026", "2026-02-01");
        });
        assert_eq!(describe_intent(&m), "Spending, March 2026 vs February 2026");
    }

    #[test]
    fn describe_income_total() {
        assert_eq!(describe_intent(&im(SearchIntent::IncomeTotal, |_| {})), "Income");
        let m = im(SearchIntent::IncomeTotal, |e| e.date_period = period("Last 30 days", "2026-03-01"));
        assert_eq!(describe_intent(&m), "Income, last 30 days");
    }

    #[test]
    fn describe_balance_check() {
        // The builder ignores any account filter, so the text must not mention one.
        let m = im(SearchIntent::BalanceCheck, |e| e.account_name = Some("Checking".into()));
        assert_eq!(describe_intent(&m), "Current balance of every account");
    }

    #[test]
    fn describe_budget_status() {
        assert_eq!(describe_intent(&im(SearchIntent::BudgetStatus, |_| {})), "Budget status for this month");
        let m = im(SearchIntent::BudgetStatus, |e| e.date_period = period("Last month", "2026-02-01"));
        assert_eq!(describe_intent(&m), "Budget status for 2026-02");
    }

    #[test]
    fn describe_transaction_search() {
        let m = im(SearchIntent::TransactionSearch, |e| {
            e.merchant = Some("Starbucks".into());
            e.amount_threshold = Some(2000);
            e.amount_operator = Some("over".into());
        });
        assert_eq!(describe_intent(&m), "Transactions at Starbucks over $20");

        let m = im(SearchIntent::TransactionSearch, |e| {
            e.category_name = Some("Food".into());
            e.amount_threshold = Some(1550);
            e.amount_operator = Some("under".into());
            e.date_period = period("Yesterday", "2026-03-30");
        });
        assert_eq!(describe_intent(&m), "Transactions in Food under $15.50, yesterday");
        assert_eq!(describe_intent(&im(SearchIntent::TransactionSearch, |_| {})), "Transactions");
    }

    #[test]
    fn describe_top_merchants() {
        let m = im(SearchIntent::TopMerchants, |e| e.date_period = period("This month", "2026-03-01"));
        assert_eq!(describe_intent(&m), "Top spending categories, this month");
        let m = im(SearchIntent::TopMerchants, |e| e.category_name = Some("Food".into()));
        assert_eq!(describe_intent(&m), "Top payees by spending");
    }

    #[test]
    fn describe_top_expenses() {
        let m = im(SearchIntent::TopExpenses, |e| e.date_period = period("This week", "2026-03-23"));
        assert_eq!(describe_intent(&m), "Largest expenses, this week");
    }

    #[test]
    fn describe_recurring_check() {
        assert_eq!(describe_intent(&im(SearchIntent::RecurringCheck, |_| {})), "Active recurring transactions");
    }

    #[test]
    fn describe_net_worth() {
        assert_eq!(describe_intent(&im(SearchIntent::NetWorth, |_| {})), "Net worth across all accounts");
    }
}
