use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecurringTransaction {
    pub id: String,
    pub account_id: String,
    pub payee: String,
    pub description: String,
    pub amount: i64,
    pub kind: String,
    pub category_id: Option<String>,
    pub transfer_to_account_id: Option<String>,
    pub frequency: String,
    pub start_date: String,
    pub end_date: Option<String>,
    pub next_date: String,
    pub is_active: bool,
    pub auto_post: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRecurringInput {
    pub account_id: String,
    pub payee: Option<String>,
    pub description: Option<String>,
    pub amount: i64,
    pub kind: String,
    pub category_id: Option<String>,
    pub transfer_to_account_id: Option<String>,
    pub frequency: String,
    pub start_date: String,
    pub end_date: Option<String>,
    pub auto_post: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRecurringInput {
    pub account_id: Option<String>,
    pub payee: Option<String>,
    pub description: Option<String>,
    pub amount: Option<i64>,
    pub kind: Option<String>,
    pub category_id: Option<String>,
    pub transfer_to_account_id: Option<String>,
    pub frequency: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub next_date: Option<String>,
    pub is_active: Option<bool>,
    pub auto_post: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DueRecurringItem {
    pub recurring: RecurringTransaction,
    pub missed_count: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateResult {
    pub generated_count: i64,
    pub items_processed: i64,
}
