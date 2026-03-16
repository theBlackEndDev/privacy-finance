use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionSplit {
    pub id: String,
    pub transaction_id: String,
    pub category_id: String,
    pub category_name: Option<String>,
    pub category_color: Option<String>,
    pub amount: i64,
    pub description: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSplitInput {
    pub category_id: String,
    pub amount: i64,
    pub description: String,
}
