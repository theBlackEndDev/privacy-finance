use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Budget {
    pub id: String,
    pub year: i32,
    pub month: i32,
    pub notes: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetCategoryRow {
    pub item_id: String,
    pub category_id: String,
    pub category_name: String,
    pub category_color: String,
    pub budgeted: i64,
    pub spent: i64,
    pub available: i64,
    pub rollover: bool,
    pub group_id: Option<String>,
    pub group_name: Option<String>,
    pub target_amount: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetWithItems {
    pub budget: Budget,
    pub items: Vec<BudgetCategoryRow>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetSummary {
    pub total_budgeted: i64,
    pub total_spent: i64,
    pub total_available: i64,
    pub total_income: i64,
    pub to_be_budgeted: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBudgetItemInput {
    pub budgeted: Option<i64>,
    pub rollover: Option<bool>,
    pub target_amount: Option<Option<i64>>,
}
