use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub id: String,
    pub account_id: String,
    pub date: String,
    pub amount: i64,
    pub kind: String,
    pub category_id: Option<String>,
    pub payee: String,
    pub description: String,
    pub is_cleared: bool,
    pub transfer_group_id: Option<String>,
    pub import_batch_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub category_name: Option<String>,
    pub category_color: Option<String>,
    pub snippet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchSuggestion {
    pub text: String,
    pub field: String,
}
