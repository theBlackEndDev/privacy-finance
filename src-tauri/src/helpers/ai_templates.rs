use crate::models::ai_search::{SearchIntent, CardType, ExtractedEntities, IntentMatch};

struct Template {
    intent: SearchIntent,
    card_type: CardType,
    patterns: &'static [&'static str],
    requires_category: bool,
    requires_merchant: bool,
    priority: u8,
}

const TEMPLATES: &[Template] = &[
    // Category-specific spending (highest priority — must match before general spending)
    Template {
        intent: SearchIntent::SpendingByCategory,
        card_type: CardType::Total,
        patterns: &[
            "how much did i spend on",
            "how much on",
            "spending on",
            "total spending on",
            "what did i spend on",
            "how much have i spent on",
        ],
        requires_category: true,
        requires_merchant: false,
        priority: 10,
    },
    // Category detail / breakdown
    Template {
        intent: SearchIntent::CategoryDetail,
        card_type: CardType::Breakdown,
        patterns: &[
            "breakdown of",
            "details for",
            "spending details",
        ],
        requires_category: true,
        requires_merchant: false,
        priority: 9,
    },
    // Merchant-specific search
    Template {
        intent: SearchIntent::TransactionSearch,
        card_type: CardType::TransactionList,
        patterns: &[
            "how much at",
            "how much did i spend at",
            "spending at",
            "transactions at",
            "purchases at",
            "payments to",
        ],
        requires_category: false,
        requires_merchant: true,
        priority: 9,
    },
    // Budget status
    Template {
        intent: SearchIntent::BudgetStatus,
        card_type: CardType::BudgetStatus,
        patterns: &[
            "am i over budget",
            "am i on budget",
            "budget status",
            "budget remaining",
            "how is my budget",
            "how's my budget",
            "budget for",
            "over budget",
            "under budget",
        ],
        requires_category: false,
        requires_merchant: false,
        priority: 8,
    },
    // Comparison
    Template {
        intent: SearchIntent::SpendingComparison,
        card_type: CardType::Comparison,
        patterns: &[
            "compare",
            "vs",
            "versus",
            "compared to",
            "this month vs",
            "this month versus",
            "spending trend",
        ],
        requires_category: false,
        requires_merchant: false,
        priority: 8,
    },
    // Transaction search (amount-based)
    Template {
        intent: SearchIntent::TransactionSearch,
        card_type: CardType::TransactionList,
        patterns: &[
            "show me transactions",
            "show transactions",
            "find transactions",
            "list transactions",
            "transactions over",
            "transactions under",
            "purchases over",
            "charges over",
            "find payments",
            "show me all",
        ],
        requires_category: false,
        requires_merchant: false,
        priority: 7,
    },
    // General spending total
    Template {
        intent: SearchIntent::SpendingTotal,
        card_type: CardType::Total,
        patterns: &[
            "how much did i spend",
            "how much have i spent",
            "total spending",
            "total expenses",
            "what did i spend",
        ],
        requires_category: false,
        requires_merchant: false,
        priority: 6,
    },
    // Income
    Template {
        intent: SearchIntent::IncomeTotal,
        card_type: CardType::Total,
        patterns: &[
            "how much did i earn",
            "how much have i earned",
            "total income",
            "how much income",
            "what was my income",
            "what did i earn",
        ],
        requires_category: false,
        requires_merchant: false,
        priority: 7,
    },
    // Top merchants
    Template {
        intent: SearchIntent::TopMerchants,
        card_type: CardType::Breakdown,
        patterns: &[
            "top merchants",
            "where do i spend the most",
            "where do i shop",
            "biggest payees",
            "who do i pay the most",
            "where does my money go",
        ],
        requires_category: false,
        requires_merchant: false,
        priority: 7,
    },
    // Top expenses
    Template {
        intent: SearchIntent::TopExpenses,
        card_type: CardType::TransactionList,
        patterns: &[
            "biggest expense",
            "largest transaction",
            "most expensive",
            "top spending",
            "biggest purchase",
            "largest purchase",
        ],
        requires_category: false,
        requires_merchant: false,
        priority: 7,
    },
    // Recurring / subscriptions
    Template {
        intent: SearchIntent::RecurringCheck,
        card_type: CardType::TransactionList,
        patterns: &[
            "subscriptions",
            "recurring",
            "recurring charges",
            "recurring payments",
            "what are my subscriptions",
            "upcoming bills",
            "monthly bills",
        ],
        requires_category: false,
        requires_merchant: false,
        priority: 7,
    },
    // Balance check
    Template {
        intent: SearchIntent::BalanceCheck,
        card_type: CardType::AccountSummary,
        patterns: &[
            "what is my balance",
            "what's my balance",
            "account balance",
            "how much money do i have",
            "how much do i have",
            "what's in my",
            "check my balance",
        ],
        requires_category: false,
        requires_merchant: false,
        priority: 7,
    },
    // Net worth
    Template {
        intent: SearchIntent::NetWorth,
        card_type: CardType::AccountSummary,
        patterns: &[
            "net worth",
            "total assets",
            "total across all accounts",
            "how much money total",
            "what am i worth",
        ],
        requires_category: false,
        requires_merchant: false,
        priority: 7,
    },
    // Spending by category breakdown (general — no specific category)
    Template {
        intent: SearchIntent::TopMerchants,
        card_type: CardType::Breakdown,
        patterns: &[
            "spending by category",
            "breakdown by category",
            "category breakdown",
            "spending breakdown",
        ],
        requires_category: false,
        requires_merchant: false,
        priority: 6,
    },
];

pub fn detect_intent(query: &str, entities: &ExtractedEntities) -> Option<(IntentMatch, CardType)> {
    let normalized = query.trim().to_lowercase().replace('?', "");

    let mut best: Option<(&Template, usize)> = None; // (template, pattern_position)

    for template in TEMPLATES {
        // Check entity requirements
        if template.requires_category && entities.category_name.is_none() {
            continue;
        }
        if template.requires_merchant && entities.merchant.is_none() {
            continue;
        }

        for pattern in template.patterns {
            if let Some(pos) = normalized.find(pattern) {
                match &best {
                    None => best = Some((template, pos)),
                    Some((prev, _)) => {
                        if template.priority > prev.priority {
                            best = Some((template, pos));
                        }
                    }
                }
                break; // First matching pattern for this template is enough
            }
        }
    }

    let (template, _) = best?;

    Some((
        IntentMatch {
            intent: template.intent.clone(),
            confidence: 1.0,
            entities: entities.clone(),
        },
        template.card_type.clone(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_entities() -> ExtractedEntities {
        ExtractedEntities::default()
    }

    fn entities_with_category() -> ExtractedEntities {
        ExtractedEntities {
            category_name: Some("Dining".to_string()),
            category_id: Some("c1".to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn test_spending_total() {
        let (m, ct) = detect_intent("how much did i spend", &empty_entities()).unwrap();
        assert_eq!(m.intent, SearchIntent::SpendingTotal);
        assert_eq!(ct, CardType::Total);
    }

    #[test]
    fn test_spending_by_category() {
        let (m, ct) = detect_intent("how much did i spend on dining", &entities_with_category()).unwrap();
        assert_eq!(m.intent, SearchIntent::SpendingByCategory);
        assert_eq!(ct, CardType::Total);
    }

    #[test]
    fn test_category_required_skips_without() {
        // "how much did i spend on X" without category entity should fall through to SpendingTotal
        let (m, _) = detect_intent("how much did i spend on something", &empty_entities()).unwrap();
        assert_eq!(m.intent, SearchIntent::SpendingTotal);
    }

    #[test]
    fn test_income() {
        let (m, _) = detect_intent("how much did i earn this month", &empty_entities()).unwrap();
        assert_eq!(m.intent, SearchIntent::IncomeTotal);
    }

    #[test]
    fn test_balance() {
        let (m, ct) = detect_intent("how much money do i have", &empty_entities()).unwrap();
        assert_eq!(m.intent, SearchIntent::BalanceCheck);
        assert_eq!(ct, CardType::AccountSummary);
    }

    #[test]
    fn test_budget() {
        let (m, ct) = detect_intent("am i over budget on dining", &entities_with_category()).unwrap();
        assert_eq!(m.intent, SearchIntent::BudgetStatus);
        assert_eq!(ct, CardType::BudgetStatus);
    }

    #[test]
    fn test_comparison() {
        let (m, ct) = detect_intent("compare this month vs last month", &empty_entities()).unwrap();
        assert_eq!(m.intent, SearchIntent::SpendingComparison);
        assert_eq!(ct, CardType::Comparison);
    }

    #[test]
    fn test_subscriptions() {
        let (m, _) = detect_intent("what are my subscriptions", &empty_entities()).unwrap();
        assert_eq!(m.intent, SearchIntent::RecurringCheck);
    }

    #[test]
    fn test_top_merchants() {
        let (m, ct) = detect_intent("where does my money go", &empty_entities()).unwrap();
        assert_eq!(m.intent, SearchIntent::TopMerchants);
        assert_eq!(ct, CardType::Breakdown);
    }

    #[test]
    fn test_no_match() {
        let result = detect_intent("what color is the sky", &empty_entities());
        assert!(result.is_none());
    }

    #[test]
    fn test_net_worth() {
        let (m, ct) = detect_intent("what is my net worth", &empty_entities()).unwrap();
        assert_eq!(m.intent, SearchIntent::NetWorth);
        assert_eq!(ct, CardType::AccountSummary);
    }

    #[test]
    fn test_transactions_over() {
        let (m, ct) = detect_intent("show me transactions over $200", &empty_entities()).unwrap();
        assert_eq!(m.intent, SearchIntent::TransactionSearch);
        assert_eq!(ct, CardType::TransactionList);
    }
}
