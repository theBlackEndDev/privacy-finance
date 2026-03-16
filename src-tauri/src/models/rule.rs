use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategorizationRule {
    pub id: String,
    pub name: String,
    pub match_field: String,
    pub match_type: String,
    pub pattern: String,
    pub category_id: String,
    pub priority: i64,
    pub is_active: bool,
    pub apply_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRuleInput {
    pub name: String,
    pub match_field: String,
    pub match_type: String,
    pub pattern: String,
    pub category_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRuleInput {
    pub name: Option<String>,
    pub match_field: Option<String>,
    pub match_type: Option<String>,
    pub pattern: Option<String>,
    pub category_id: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleTestResult {
    pub transaction_id: String,
    pub date: String,
    pub payee: String,
    pub description: String,
    pub amount: i64,
    pub matched_field_value: String,
}
