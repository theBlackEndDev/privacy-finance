use crate::models::ai_search::{SearchIntent, IntentMatch};

pub struct BuiltQuery {
    pub sql: String,
    pub params: Vec<String>,
}

pub fn build_query(intent_match: &IntentMatch) -> Vec<BuiltQuery> {
    let e = &intent_match.entities;

    match intent_match.intent {
        SearchIntent::SpendingTotal | SearchIntent::SpendingByCategory | SearchIntent::CategoryDetail => {
            let mut where_clauses = vec!["t.kind = 'expense'".to_string()];
            let mut params = Vec::new();
            let mut idx = 1;

            if let Some(ref cat_id) = e.category_id {
                where_clauses.push(format!("t.category_id = ?{}", idx));
                params.push(cat_id.clone());
                idx += 1;
            }
            add_date_clauses(&mut where_clauses, &mut params, &mut idx, e);

            let where_str = where_clauses.join(" AND ");

            let mut queries = vec![];

            // Main aggregate
            queries.push(BuiltQuery {
                sql: format!(
                    "SELECT COALESCE(SUM(ABS(t.amount)), 0) as total, COUNT(*) as count
                     FROM transactions t WHERE {}", where_str
                ),
                params: params.clone(),
            });

            // Top payees breakdown
            queries.push(BuiltQuery {
                sql: format!(
                    "SELECT t.payee, SUM(ABS(t.amount)) as total, COUNT(*) as count
                     FROM transactions t WHERE {} AND t.payee != '' AND t.payee IS NOT NULL
                     GROUP BY t.payee ORDER BY total DESC LIMIT 5", where_str
                ),
                params: params.clone(),
            });

            // Category breakdown (for general spending or CategoryDetail)
            if e.category_id.is_none() {
                queries.push(BuiltQuery {
                    sql: format!(
                        "SELECT c.name, c.color, SUM(ABS(t.amount)) as total
                         FROM transactions t
                         LEFT JOIN categories c ON t.category_id = c.id
                         WHERE {} GROUP BY c.id ORDER BY total DESC LIMIT 10", where_str
                    ),
                    params: params.clone(),
                });
            }

            // Grand total for percentage calculation (query index 3 if breakdown present, 2 otherwise)
            if e.wants_percentage && e.category_id.is_some() {
                let mut total_clauses = vec!["t.kind = 'expense'".to_string()];
                let mut total_params = Vec::new();
                let mut total_idx = 1;
                add_date_clauses(&mut total_clauses, &mut total_params, &mut total_idx, e);
                queries.push(BuiltQuery {
                    sql: format!(
                        "SELECT COALESCE(SUM(ABS(t.amount)), 0) FROM transactions t WHERE {}",
                        total_clauses.join(" AND ")
                    ),
                    params: total_params,
                });
            }

            queries
        }

        SearchIntent::SpendingComparison => {
            let mut queries = vec![];
            // Current period
            if let Some(ref dp) = e.date_period {
                queries.push(BuiltQuery {
                    sql: "SELECT COALESCE(SUM(ABS(t.amount)), 0) FROM transactions t
                          WHERE t.kind = 'expense' AND t.date >= ?1 AND t.date <= ?2".to_string(),
                    params: vec![dp.start_date.clone(), dp.end_date.clone()],
                });
            } else {
                // Default: this month
                queries.push(BuiltQuery {
                    sql: "SELECT COALESCE(SUM(ABS(t.amount)), 0) FROM transactions t
                          WHERE t.kind = 'expense' AND strftime('%Y-%m', t.date) = strftime('%Y-%m', 'now')".to_string(),
                    params: vec![],
                });
            }
            // Comparison period
            if let Some(ref cp) = e.comparison_period {
                queries.push(BuiltQuery {
                    sql: "SELECT COALESCE(SUM(ABS(t.amount)), 0) FROM transactions t
                          WHERE t.kind = 'expense' AND t.date >= ?1 AND t.date <= ?2".to_string(),
                    params: vec![cp.start_date.clone(), cp.end_date.clone()],
                });
            } else {
                // Default: last month
                queries.push(BuiltQuery {
                    sql: "SELECT COALESCE(SUM(ABS(t.amount)), 0) FROM transactions t
                          WHERE t.kind = 'expense' AND strftime('%Y-%m', t.date) = strftime('%Y-%m', 'now', '-1 month')".to_string(),
                    params: vec![],
                });
            }
            queries
        }

        SearchIntent::IncomeTotal => {
            let mut where_clauses = vec!["t.kind = 'income'".to_string()];
            let mut params = Vec::new();
            let mut idx = 1;
            add_date_clauses(&mut where_clauses, &mut params, &mut idx, e);
            let where_str = where_clauses.join(" AND ");
            vec![BuiltQuery {
                sql: format!(
                    "SELECT COALESCE(SUM(ABS(t.amount)), 0) as total, COUNT(*) as count
                     FROM transactions t WHERE {}", where_str
                ),
                params,
            }]
        }

        SearchIntent::BalanceCheck | SearchIntent::NetWorth => {
            vec![BuiltQuery {
                sql: "SELECT a.id, a.name, a.type,
                      a.opening_balance + COALESCE(
                        (SELECT SUM(CASE WHEN t.kind = 'income' THEN ABS(t.amount)
                                         WHEN t.kind = 'expense' THEN -ABS(t.amount)
                                         ELSE 0 END)
                         FROM transactions t WHERE t.account_id = a.id), 0
                      ) as balance
                      FROM accounts a WHERE a.is_archived = 0 ORDER BY a.sort_order".to_string(),
                params: vec![],
            }]
        }

        SearchIntent::BudgetStatus => {
            let month = if let Some(ref dp) = e.date_period {
                dp.start_date[..7].to_string() // YYYY-MM
            } else {
                chrono::Local::now().format("%Y-%m").to_string()
            };
            vec![BuiltQuery {
                sql: "SELECT c.name, c.color,
                      bi.amount as budgeted,
                      COALESCE((SELECT SUM(ABS(t.amount)) FROM transactions t
                       WHERE t.category_id = bi.category_id
                       AND t.kind = 'expense'
                       AND strftime('%Y-%m', t.date) = ?1), 0) as spent
                      FROM budget_items bi
                      JOIN categories c ON bi.category_id = c.id
                      JOIN budgets b ON bi.budget_id = b.id
                      WHERE b.month = ?1
                      ORDER BY spent DESC".to_string(),
                params: vec![month],
            }]
        }

        SearchIntent::TransactionSearch => {
            let mut where_clauses = vec!["1=1".to_string()];
            let mut params = Vec::new();
            let mut idx = 1;

            if let Some(ref merchant) = e.merchant {
                let stripped = crate::helpers::ai_entities::strip_punctuation(&merchant.to_lowercase());
                let idx2 = idx + 1;
                where_clauses.push(format!(
                    "(LOWER(t.payee) LIKE ?{idx} OR LOWER(t.description) LIKE ?{idx} \
                     OR REPLACE(REPLACE(REPLACE(LOWER(t.payee), '''', ''), '.', ''), '-', '') LIKE ?{idx2})"
                ));
                params.push(format!("%{}%", merchant.to_lowercase()));
                params.push(format!("%{}%", stripped));
                idx += 2;
            }
            if let Some(ref cat_id) = e.category_id {
                where_clauses.push(format!("t.category_id = ?{}", idx));
                params.push(cat_id.clone());
                idx += 1;
            }
            if let Some(threshold) = e.amount_threshold {
                let op = e.amount_operator.as_deref().unwrap_or("over");
                match op {
                    "over" => where_clauses.push(format!("ABS(t.amount) > ?{}", idx)),
                    "under" => where_clauses.push(format!("ABS(t.amount) < ?{}", idx)),
                    _ => where_clauses.push(format!("ABS(t.amount) > ?{}", idx)),
                }
                params.push(threshold.to_string());
                idx += 1;
            }
            add_date_clauses(&mut where_clauses, &mut params, &mut idx, e);

            let where_str = where_clauses.join(" AND ");
            vec![BuiltQuery {
                sql: format!(
                    "SELECT t.id, t.date, t.payee, t.amount, t.kind, c.name as category_name
                     FROM transactions t
                     LEFT JOIN categories c ON t.category_id = c.id
                     WHERE {} ORDER BY t.date DESC LIMIT 20", where_str
                ),
                params,
            }]
        }

        SearchIntent::TopMerchants => {
            let mut where_clauses = vec!["t.kind = 'expense'".to_string(), "t.payee != ''".to_string(), "t.payee IS NOT NULL".to_string()];
            let mut params = Vec::new();
            let mut idx = 1;
            add_date_clauses(&mut where_clauses, &mut params, &mut idx, e);
            let where_str = where_clauses.join(" AND ");

            // Check if this is a "by category" breakdown
            let sql = if intent_match.entities.category_id.is_none() {
                format!(
                    "SELECT c.name, c.color, SUM(ABS(t.amount)) as total
                     FROM transactions t
                     LEFT JOIN categories c ON t.category_id = c.id
                     WHERE t.kind = 'expense' {} GROUP BY c.id ORDER BY total DESC LIMIT 10",
                    if where_clauses.len() > 3 {
                        format!("AND {}", where_clauses[3..].join(" AND "))
                    } else { String::new() }
                )
            } else {
                format!(
                    "SELECT t.payee, NULL, SUM(ABS(t.amount)) as total
                     FROM transactions t WHERE {} GROUP BY t.payee ORDER BY total DESC LIMIT 10", where_str
                )
            };
            vec![BuiltQuery { sql, params }]
        }

        SearchIntent::TopExpenses => {
            let mut where_clauses = vec!["t.kind = 'expense'".to_string()];
            let mut params = Vec::new();
            let mut idx = 1;
            add_date_clauses(&mut where_clauses, &mut params, &mut idx, e);
            let where_str = where_clauses.join(" AND ");
            vec![BuiltQuery {
                sql: format!(
                    "SELECT t.id, t.date, t.payee, t.amount, t.kind, c.name as category_name
                     FROM transactions t
                     LEFT JOIN categories c ON t.category_id = c.id
                     WHERE {} ORDER BY ABS(t.amount) DESC LIMIT 10", where_str
                ),
                params,
            }]
        }

        SearchIntent::RecurringCheck => {
            vec![BuiltQuery {
                sql: "SELECT id, payee, amount, frequency, next_date
                      FROM recurring_transactions WHERE is_active = 1
                      ORDER BY ABS(amount) DESC".to_string(),
                params: vec![],
            }]
        }
    }
}

fn add_date_clauses(
    where_clauses: &mut Vec<String>,
    params: &mut Vec<String>,
    idx: &mut usize,
    entities: &crate::models::ai_search::ExtractedEntities,
) {
    if let Some(ref dp) = entities.date_period {
        where_clauses.push(format!("t.date >= ?{}", idx));
        params.push(dp.start_date.clone());
        *idx += 1;
        where_clauses.push(format!("t.date <= ?{}", idx));
        params.push(dp.end_date.clone());
        *idx += 1;
    }
}
