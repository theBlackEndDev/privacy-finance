use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportDateRange {
    pub start_date: String,
    pub end_date: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IncomeExpenseMonth {
    pub month: String,
    pub income: i64,
    pub expense: i64,
    pub net: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IncomeExpenseReport {
    pub months: Vec<IncomeExpenseMonth>,
    pub previous_income: i64,
    pub previous_expense: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CategorySpendingRow {
    pub category_id: Option<String>,
    pub category_name: String,
    pub category_color: String,
    pub total: i64,
    pub percentage: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CashFlowPoint {
    pub date: String,
    pub income: i64,
    pub expense: i64,
    pub net: i64,
    pub cumulative: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetWorthPoint {
    pub month: String,
    pub total: i64,
    pub accounts: Vec<NetWorthAccount>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetWorthAccount {
    pub account_id: String,
    pub account_name: String,
    pub balance: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TagSpendingRow {
    pub tag_id: String,
    pub tag_name: String,
    pub tag_color: Option<String>,
    pub total: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PayeeSpendingRow {
    pub payee: String,
    pub total: i64,
    pub count: i64,
}
