use std::fmt;
use crate::error::{AppError, AppResult};

/// Parsed representation of a scope string like "transactions:read:category:streaming"
#[derive(Debug, Clone, PartialEq)]
pub enum Scope {
    TransactionsRead {
        category: Option<String>,
        date_days: Option<u32>,
        amount_under: Option<i64>,
        amount_over: Option<i64>,
    },
    IncomeRead,
    AccountsReadBalances,
    BudgetRead {
        category: Option<String>,
    },
    InsightsGenerate,
}

impl Scope {
    /// Parse a colon-delimited scope string into a typed Scope.
    pub fn parse(s: &str) -> AppResult<Scope> {
        let parts: Vec<&str> = s.split(':').collect();
        match parts.as_slice() {
            ["transactions", "read"] => Ok(Scope::TransactionsRead {
                category: None,
                date_days: None,
                amount_under: None,
                amount_over: None,
            }),
            ["transactions", "read", "category", cat] => Ok(Scope::TransactionsRead {
                category: Some(cat.to_string()),
                date_days: None,
                amount_under: None,
                amount_over: None,
            }),
            ["transactions", "read", "date", period] => {
                let days = parse_date_period(period)?;
                Ok(Scope::TransactionsRead {
                    category: None,
                    date_days: Some(days),
                    amount_under: None,
                    amount_over: None,
                })
            }
            ["transactions", "read", "amount", constraint] => {
                if let Some(val) = constraint.strip_prefix("under_") {
                    let amount = val.parse::<i64>().map_err(|_| {
                        AppError::Validation(format!("invalid amount in scope: {s}"))
                    })?;
                    Ok(Scope::TransactionsRead {
                        category: None,
                        date_days: None,
                        amount_under: Some(amount),
                        amount_over: None,
                    })
                } else if let Some(val) = constraint.strip_prefix("over_") {
                    let amount = val.parse::<i64>().map_err(|_| {
                        AppError::Validation(format!("invalid amount in scope: {s}"))
                    })?;
                    Ok(Scope::TransactionsRead {
                        category: None,
                        date_days: None,
                        amount_under: None,
                        amount_over: Some(amount),
                    })
                } else {
                    Err(AppError::Validation(format!("invalid amount scope: {s}")))
                }
            }
            ["income", "read"] => Ok(Scope::IncomeRead),
            ["accounts", "read", "balances"] => Ok(Scope::AccountsReadBalances),
            ["budget", "read"] => Ok(Scope::BudgetRead { category: None }),
            ["budget", "read", "category", cat] => Ok(Scope::BudgetRead {
                category: Some(cat.to_string()),
            }),
            ["insights", "generate"] => Ok(Scope::InsightsGenerate),
            _ => Err(AppError::Validation(format!("unknown scope: {s}"))),
        }
    }

    /// Convert a scope back to its canonical string representation.
    pub fn to_scope_string(&self) -> String {
        match self {
            Scope::TransactionsRead { category, date_days, amount_under, amount_over } => {
                if let Some(cat) = category {
                    format!("transactions:read:category:{cat}")
                } else if let Some(days) = date_days {
                    format!("transactions:read:date:last_{days}_days")
                } else if let Some(amount) = amount_under {
                    format!("transactions:read:amount:under_{amount}")
                } else if let Some(amount) = amount_over {
                    format!("transactions:read:amount:over_{amount}")
                } else {
                    "transactions:read".to_string()
                }
            }
            Scope::IncomeRead => "income:read".to_string(),
            Scope::AccountsReadBalances => "accounts:read:balances".to_string(),
            Scope::BudgetRead { category } => {
                if let Some(cat) = category {
                    format!("budget:read:category:{cat}")
                } else {
                    "budget:read".to_string()
                }
            }
            Scope::InsightsGenerate => "insights:generate".to_string(),
        }
    }

    /// Translate a scope to plain English for the UI.
    pub fn to_human_readable(&self) -> String {
        match self {
            Scope::TransactionsRead { category, date_days, amount_under, amount_over } => {
                let mut parts = Vec::new();
                if let Some(cat) = category {
                    parts.push(format!("Your {cat} transactions"));
                } else {
                    parts.push("Your transactions".to_string());
                }
                if let Some(days) = date_days {
                    parts.push(format!("from the last {days} days"));
                }
                if let Some(amount) = amount_under {
                    parts.push(format!("under ${amount}"));
                }
                if let Some(amount) = amount_over {
                    parts.push(format!("over ${amount}"));
                }
                parts.join(" ")
            }
            Scope::IncomeRead => "Your income transactions".to_string(),
            Scope::AccountsReadBalances => "Your account balances".to_string(),
            Scope::BudgetRead { category } => {
                if let Some(cat) = category {
                    format!("Your {cat} budget allocations")
                } else {
                    "Your budget allocations".to_string()
                }
            }
            Scope::InsightsGenerate => "Generate insights from your data".to_string(),
        }
    }
}

impl fmt::Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_scope_string())
    }
}

fn parse_date_period(period: &str) -> AppResult<u32> {
    // Handle "last_N_days" format
    let s = period.strip_prefix("last_").unwrap_or(period);
    let s = s.strip_suffix("_days").unwrap_or(s);
    s.parse::<u32>().map_err(|_| {
        AppError::Validation(format!("invalid date period: {period}"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_transactions_read_all() {
        let scope = Scope::parse("transactions:read").unwrap();
        assert_eq!(scope, Scope::TransactionsRead {
            category: None, date_days: None, amount_under: None, amount_over: None,
        });
        assert_eq!(scope.to_scope_string(), "transactions:read");
    }

    #[test]
    fn parse_transactions_read_category() {
        let scope = Scope::parse("transactions:read:category:streaming").unwrap();
        assert_eq!(scope, Scope::TransactionsRead {
            category: Some("streaming".to_string()), date_days: None, amount_under: None, amount_over: None,
        });
        assert_eq!(scope.to_scope_string(), "transactions:read:category:streaming");
        assert_eq!(scope.to_human_readable(), "Your streaming transactions");
    }

    #[test]
    fn parse_transactions_read_date() {
        let scope = Scope::parse("transactions:read:date:last_90_days").unwrap();
        assert_eq!(scope, Scope::TransactionsRead {
            category: None, date_days: Some(90), amount_under: None, amount_over: None,
        });
    }

    #[test]
    fn parse_transactions_read_amount() {
        let scope = Scope::parse("transactions:read:amount:under_500").unwrap();
        assert_eq!(scope, Scope::TransactionsRead {
            category: None, date_days: None, amount_under: Some(500), amount_over: None,
        });
    }

    #[test]
    fn parse_income_read() {
        let scope = Scope::parse("income:read").unwrap();
        assert_eq!(scope, Scope::IncomeRead);
    }

    #[test]
    fn parse_budget_read_category() {
        let scope = Scope::parse("budget:read:category:dining").unwrap();
        assert_eq!(scope, Scope::BudgetRead { category: Some("dining".to_string()) });
    }

    #[test]
    fn parse_invalid_scope() {
        assert!(Scope::parse("invalid:scope:value").is_err());
    }

    #[test]
    fn round_trip_all_scopes() {
        let scopes = vec![
            "transactions:read",
            "transactions:read:category:streaming",
            "transactions:read:date:last_30_days",
            "transactions:read:amount:under_500",
            "transactions:read:amount:over_200",
            "income:read",
            "accounts:read:balances",
            "budget:read",
            "budget:read:category:dining",
            "insights:generate",
        ];
        for s in scopes {
            let parsed = Scope::parse(s).unwrap();
            assert_eq!(parsed.to_scope_string(), s, "round-trip failed for: {s}");
        }
    }
}
