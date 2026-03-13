use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CategorySummary {
    pub category_id: Option<String>,
    pub category_name: String,
    pub category_color: String,
    pub total: i64,   // cents
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonthSummary {
    pub month: String,  // YYYY-MM
    pub income: i64,
    pub expense: i64,
    pub net: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountBalance {
    pub account_id: String,
    pub account_name: String,
    pub account_type: String,
    pub balance: i64,  // cents
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardData {
    pub income: i64,
    pub expense: i64,
    pub net: i64,
    pub by_category: Vec<CategorySummary>,
}
