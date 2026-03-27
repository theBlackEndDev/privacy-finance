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

    let text = format!(
        "You spent {} on {}{} ({} transactions)",
        format_dollars(total),
        label.to_lowercase(),
        if period_label != "All time" { format!(" {}", period_label.to_lowercase()) } else { String::new() },
        count
    );

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
