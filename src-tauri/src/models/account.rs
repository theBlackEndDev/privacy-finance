use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub account_type: String,
    pub currency_code: String,
    pub opening_balance: i64,
    pub sort_order: i64,
    pub is_archived: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAccountInput {
    pub name: String,
    #[serde(rename = "type")]
    pub account_type: String,
    pub currency_code: Option<String>,
    pub opening_balance: Option<i64>,
    pub sort_order: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAccountInput {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub account_type: Option<String>,
    pub currency_code: Option<String>,
    pub opening_balance: Option<i64>,
    pub sort_order: Option<i64>,
    pub is_archived: Option<bool>,
}
