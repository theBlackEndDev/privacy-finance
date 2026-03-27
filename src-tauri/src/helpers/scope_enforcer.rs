use rusqlite::Connection;
use crate::error::{AppError, AppResult};
use crate::models::agent::Agent;
use crate::models::scope::Scope;
use crate::commands::agent_keys::hash_key;

/// Result of validating a bearer key against the database.
pub struct ValidatedAgent {
    pub agent: Agent,
    pub granted_scopes: Vec<String>,
}

/// Validate a raw API key: hash it, look up in agent_keys, check expiry,
/// return the associated agent and its granted scopes.
pub fn validate_key(raw_key: &str, conn: &Connection) -> AppResult<ValidatedAgent> {
    let key_hash = hash_key(raw_key);

    let (key_id, agent_id, scopes_json, expires_at): (String, String, String, Option<String>) = conn
        .query_row(
            "SELECT ak.id, ak.agent_id, ak.scopes, ak.expires_at
             FROM agent_keys ak WHERE ak.key_hash = ?1",
            rusqlite::params![key_hash],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .map_err(|_| AppError::AgentAuth("invalid API key".to_string()))?;

    // Check expiry
    if let Some(ref exp) = expires_at {
        if let Ok(expiry) = chrono::DateTime::parse_from_rfc3339(exp) {
            if expiry < chrono::Utc::now() {
                return Err(AppError::AgentAuth("API key has expired".to_string()));
            }
        }
    }

    // Update last_used_at
    let now = crate::helpers::timestamps::now_utc();
    let _ = conn.execute(
        "UPDATE agent_keys SET last_used_at = ?1 WHERE id = ?2",
        rusqlite::params![now, key_id],
    );

    // Get the agent
    let agent = conn
        .query_row(
            "SELECT id, name, publisher, version, description, manifest_url, icon_url, status, created_at, updated_at
             FROM agents WHERE id = ?1",
            rusqlite::params![agent_id],
            |row| {
                Ok(Agent {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    publisher: row.get(2)?,
                    version: row.get(3)?,
                    description: row.get(4)?,
                    manifest_url: row.get(5)?,
                    icon_url: row.get(6)?,
                    status: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                })
            },
        )
        .map_err(|_| AppError::AgentAuth("agent not found for key".to_string()))?;

    // Verify agent is still approved
    if agent.status != "approved" {
        return Err(AppError::AgentAuth(format!(
            "agent '{}' is no longer approved (status: {})",
            agent.name, agent.status
        )));
    }

    let granted_scopes: Vec<String> = serde_json::from_str(&scopes_json).unwrap_or_default();

    Ok(ValidatedAgent {
        agent,
        granted_scopes,
    })
}

/// Check if a requested scope is covered by any of the granted scopes.
/// A broader grant covers a narrower request (e.g., "transactions:read" covers "transactions:read:category:streaming").
pub fn enforce_scope(requested: &Scope, granted: &[Scope]) -> bool {
    for grant in granted {
        if scope_covers(grant, requested) {
            return true;
        }
    }
    false
}

/// Check if `grant` covers `request`.
/// A grant covers a request if it is the same scope or a broader version.
fn scope_covers(grant: &Scope, request: &Scope) -> bool {
    match (grant, request) {
        // transactions:read (broad) covers any transactions:read:* (narrow)
        (
            Scope::TransactionsRead { category: None, date_days: None, amount_under: None, amount_over: None },
            Scope::TransactionsRead { .. },
        ) => true,

        // Exact category match
        (
            Scope::TransactionsRead { category: Some(gc), .. },
            Scope::TransactionsRead { category: Some(rc), .. },
        ) if gc == rc => true,

        // Broader date grant covers narrower request (grant 90 days covers request 30 days)
        (
            Scope::TransactionsRead { date_days: Some(gd), .. },
            Scope::TransactionsRead { date_days: Some(rd), .. },
        ) if gd >= rd => true,

        // Broader amount_under grant covers narrower request
        (
            Scope::TransactionsRead { amount_under: Some(ga), .. },
            Scope::TransactionsRead { amount_under: Some(ra), .. },
        ) if ga >= ra => true,

        // Broader amount_over grant covers narrower request (lower threshold is broader)
        (
            Scope::TransactionsRead { amount_over: Some(ga), .. },
            Scope::TransactionsRead { amount_over: Some(ra), .. },
        ) if ga <= ra => true,

        // Simple scope exact matches
        (Scope::IncomeRead, Scope::IncomeRead) => true,
        (Scope::AccountsReadBalances, Scope::AccountsReadBalances) => true,

        // budget:read (broad) covers budget:read:category:X (narrow)
        (Scope::BudgetRead { category: None }, Scope::BudgetRead { .. }) => true,
        (Scope::BudgetRead { category: Some(gc) }, Scope::BudgetRead { category: Some(rc) })
            if gc == rc => true,

        (Scope::InsightsGenerate, Scope::InsightsGenerate) => true,

        _ => false,
    }
}

/// Build a SQL WHERE clause fragment for the transactions table based on a scope.
/// Returns (where_clause, params) for use in a query.
pub fn build_where_clause(scope: &Scope) -> AppResult<(String, Vec<String>)> {
    match scope {
        Scope::TransactionsRead { category, date_days, amount_under, amount_over } => {
            let mut conditions = Vec::new();
            let mut params = Vec::new();

            if let Some(cat) = category {
                conditions.push("c.name = ?".to_string());
                params.push(cat.clone());
            }
            if let Some(days) = date_days {
                conditions.push(format!(
                    "t.date >= date('now', '-{days} days')"
                ));
            }
            if let Some(amount) = amount_under {
                // Amounts stored in cents
                conditions.push("ABS(t.amount) < ?".to_string());
                params.push((amount * 100).to_string());
            }
            if let Some(amount) = amount_over {
                conditions.push("ABS(t.amount) > ?".to_string());
                params.push((amount * 100).to_string());
            }

            let where_clause = if conditions.is_empty() {
                "1=1".to_string()
            } else {
                conditions.join(" AND ")
            };

            Ok((where_clause, params))
        }
        Scope::IncomeRead => {
            Ok(("t.kind = 'income'".to_string(), vec![]))
        }
        _ => Err(AppError::AgentScope(format!(
            "scope {} cannot be used for transaction queries",
            scope
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn broad_transactions_read_covers_all_narrow() {
        let broad = Scope::TransactionsRead {
            category: None, date_days: None, amount_under: None, amount_over: None,
        };
        let narrow = Scope::TransactionsRead {
            category: Some("streaming".to_string()), date_days: None,
            amount_under: None, amount_over: None,
        };
        assert!(enforce_scope(&narrow, &[broad]));
    }

    #[test]
    fn category_match() {
        let grant = Scope::TransactionsRead {
            category: Some("streaming".to_string()), date_days: None,
            amount_under: None, amount_over: None,
        };
        let request_same = Scope::TransactionsRead {
            category: Some("streaming".to_string()), date_days: None,
            amount_under: None, amount_over: None,
        };
        let request_diff = Scope::TransactionsRead {
            category: Some("dining".to_string()), date_days: None,
            amount_under: None, amount_over: None,
        };
        assert!(enforce_scope(&request_same, &[grant.clone()]));
        assert!(!enforce_scope(&request_diff, &[grant]));
    }

    #[test]
    fn date_range_broader_covers_narrower() {
        let grant_90 = Scope::TransactionsRead {
            category: None, date_days: Some(90), amount_under: None, amount_over: None,
        };
        let request_30 = Scope::TransactionsRead {
            category: None, date_days: Some(30), amount_under: None, amount_over: None,
        };
        let request_120 = Scope::TransactionsRead {
            category: None, date_days: Some(120), amount_under: None, amount_over: None,
        };
        assert!(enforce_scope(&request_30, &[grant_90.clone()]));
        assert!(!enforce_scope(&request_120, &[grant_90]));
    }

    #[test]
    fn cross_scope_denied() {
        let grant = Scope::TransactionsRead {
            category: Some("streaming".to_string()), date_days: None,
            amount_under: None, amount_over: None,
        };
        assert!(!enforce_scope(&Scope::IncomeRead, &[grant.clone()]));
        assert!(!enforce_scope(&Scope::AccountsReadBalances, &[grant]));
    }

    #[test]
    fn budget_broad_covers_narrow() {
        let grant = Scope::BudgetRead { category: None };
        let request = Scope::BudgetRead { category: Some("dining".to_string()) };
        assert!(enforce_scope(&request, &[grant]));
    }

    #[test]
    fn build_where_category() {
        let scope = Scope::TransactionsRead {
            category: Some("streaming".to_string()),
            date_days: None, amount_under: None, amount_over: None,
        };
        let (clause, params) = build_where_clause(&scope).unwrap();
        assert!(clause.contains("c.name = ?"));
        assert_eq!(params, vec!["streaming"]);
    }

    #[test]
    fn build_where_income() {
        let (clause, params) = build_where_clause(&Scope::IncomeRead).unwrap();
        assert_eq!(clause, "t.kind = 'income'");
        assert!(params.is_empty());
    }

    #[test]
    fn key_hash_deterministic() {
        let key = "pf_live_sk_abc123";
        let h1 = super::hash_key(key);
        let h2 = super::hash_key(key);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64); // SHA-256 hex = 64 chars
    }
}
