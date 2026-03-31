use serde::Serialize;

// ── Intents ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SearchIntent {
    SpendingTotal,
    SpendingByCategory,
    SpendingComparison,
    IncomeTotal,
    BalanceCheck,
    BudgetStatus,
    TransactionSearch,
    TopMerchants,
    TopExpenses,
    RecurringCheck,
    NetWorth,
    CategoryDetail,
}

// ── Card types ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum CardType {
    Total,
    Comparison,
    BudgetStatus,
    TransactionList,
    Breakdown,
    AccountSummary,
}

// ── Entities ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DatePeriod {
    pub start_date: String,
    pub end_date: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ExtractedEntities {
    pub category_id: Option<String>,
    pub category_name: Option<String>,
    pub account_id: Option<String>,
    pub account_name: Option<String>,
    pub date_period: Option<DatePeriod>,
    pub comparison_period: Option<DatePeriod>,
    pub amount_threshold: Option<i64>,
    pub amount_operator: Option<String>,
    pub merchant: Option<String>,
    pub wants_percentage: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IntentMatch {
    pub intent: SearchIntent,
    pub confidence: f64,
    pub entities: ExtractedEntities,
}

// ── Card data variants ────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum CardData {
    #[serde(rename_all = "camelCase")]
    Total {
        label: String,
        amount: i64,
        count: i64,
        period_label: String,
        comparison_amount: Option<i64>,
        comparison_count: Option<i64>,
        change_pct: Option<f64>,
        top_items: Vec<BreakdownItem>,
    },
    #[serde(rename_all = "camelCase")]
    Comparison {
        label: String,
        current_label: String,
        current_amount: i64,
        previous_label: String,
        previous_amount: i64,
        change_amount: i64,
        change_pct: f64,
    },
    #[serde(rename_all = "camelCase")]
    BudgetStatus {
        items: Vec<BudgetStatusItem>,
        period_label: String,
    },
    #[serde(rename_all = "camelCase")]
    TransactionList {
        transactions: Vec<TransactionItem>,
        total: i64,
        count: i64,
        label: String,
    },
    #[serde(rename_all = "camelCase")]
    Breakdown {
        label: String,
        items: Vec<BreakdownItem>,
        total: i64,
    },
    #[serde(rename_all = "camelCase")]
    AccountSummary {
        accounts: Vec<AccountItem>,
        net_worth: i64,
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BreakdownItem {
    pub name: String,
    pub color: Option<String>,
    pub amount: i64,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetStatusItem {
    pub category_name: String,
    pub color: String,
    pub budgeted: i64,
    pub spent: i64,
    pub available: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionItem {
    pub id: String,
    pub date: String,
    pub payee: String,
    pub amount: i64,
    pub kind: String,
    pub category_name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountItem {
    pub name: String,
    pub account_type: String,
    pub balance: i64,
}

// ── Top-level result ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AISearchResult {
    pub matched: bool,
    pub intent: Option<SearchIntent>,
    pub card_type: Option<CardType>,
    pub card_data: Option<CardData>,
    pub text_answer: Option<String>,
    pub suggestions: Vec<String>,
    pub query_time_ms: u64,
}

impl AISearchResult {
    pub fn no_match() -> Self {
        AISearchResult {
            matched: false,
            intent: None,
            card_type: None,
            card_data: None,
            text_answer: None,
            suggestions: vec![
                "How much did I spend on [category]?".to_string(),
                "Show me transactions over $[amount]".to_string(),
                "Am I over budget on [category]?".to_string(),
                "Compare this month vs last month".to_string(),
                "What are my subscriptions?".to_string(),
                "How much money do I have?".to_string(),
            ],
            query_time_ms: 0,
        }
    }
}
