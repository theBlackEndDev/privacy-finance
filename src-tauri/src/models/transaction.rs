use serde::{Deserialize, Serialize};
use super::tag::TagBrief;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub id: String,
    pub account_id: String,
    pub date: String,           // YYYY-MM-DD
    pub amount: i64,            // cents, always positive
    pub kind: String,           // income | expense | transfer
    pub category_id: Option<String>,
    pub category_name: Option<String>,
    pub category_color: Option<String>,
    pub payee: String,
    pub description: String,
    pub is_cleared: bool,
    pub is_split: bool,
    pub transfer_group_id: Option<String>,
    pub import_batch_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub tags: Vec<TagBrief>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTransactionInput {
    pub account_id: String,
    pub date: String,
    pub amount: i64,
    pub kind: String,
    pub category_id: Option<String>,
    pub payee: Option<String>,
    pub description: Option<String>,
    pub is_cleared: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTransactionInput {
    pub date: Option<String>,
    pub amount: Option<i64>,
    pub kind: Option<String>,
    pub category_id: Option<String>,
    pub payee: Option<String>,
    pub description: Option<String>,
    pub is_cleared: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionFilters {
    pub account_id: Option<String>,
    pub kind: Option<String>,
    pub category_id: Option<String>,
    pub tag_id: Option<String>,
    pub search: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionPage {
    pub items: Vec<Transaction>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTransferInput {
    pub from_account_id: String,
    pub to_account_id: String,
    pub amount: i64,
    pub date: String,
    pub description: Option<String>,
}
