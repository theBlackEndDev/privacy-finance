use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reconciliation {
    pub id: String,
    pub account_id: String,
    pub statement_date: String,
    pub statement_balance: i64,
    pub status: String, // pending | completed | cancelled
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReconciliationItemDetail {
    pub transaction_id: String,
    pub date: String,
    pub payee: String,
    pub amount: i64,
    pub kind: String,
    pub is_reconciled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReconciliationSummary {
    pub reconciliation: Reconciliation,
    pub items: Vec<ReconciliationItemDetail>,
    pub cleared_balance: i64,
    pub statement_balance: i64,
    pub difference: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartReconciliationInput {
    pub account_id: String,
    pub statement_date: String,
    pub statement_balance: i64,
}
