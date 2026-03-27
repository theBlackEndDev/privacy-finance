use crate::models::ai_search::{SearchIntent, CardType, ExtractedEntities, IntentMatch};

/// A keyword-scored intent template. Instead of requiring exact phrase matches,
/// each intent has weighted keywords. The intent with the highest score wins.
struct Template {
    intent: SearchIntent,
    card_type: CardType,
    /// (keyword, weight) pairs — if the keyword appears in the query, add weight to score
    keywords: &'static [(&'static str, u8)],
    /// Negative keywords — if present, reduce score
    anti_keywords: &'static [&'static str],
    requires_category: bool,
    requires_merchant: bool,
    /// Minimum score to consider this a match
    min_score: u8,
}

const TEMPLATES: &[Template] = &[
    // ── Spending by category ──────────────────────────────────────
    Template {
        intent: SearchIntent::SpendingByCategory,
        card_type: CardType::Total,
        keywords: &[
            ("spend", 5), ("spent", 5), ("how much", 6), ("total", 3),
            ("cost", 5), ("pay", 5), ("paid", 5), ("expense", 3),
            ("per year", 5), ("per month", 5), ("annually", 5), ("monthly", 4),
        ],
        anti_keywords: &["budget", "earn", "income", "balance", "worth", "subscribe", "recurring", "compare", "vs"],
        requires_category: true,
        requires_merchant: false,
        min_score: 5,
    },
    // ── Budget status ─────────────────────────────────────────────
    Template {
        intent: SearchIntent::BudgetStatus,
        card_type: CardType::BudgetStatus,
        keywords: &[
            ("budget", 8), ("over budget", 10), ("under budget", 10),
            ("on budget", 10), ("budget left", 9), ("budget remaining", 9),
            ("left in", 5), ("remaining", 5), ("allocated", 4),
            ("how much budget", 10), ("am i over", 7), ("am i under", 7),
        ],
        anti_keywords: &[],
        requires_category: false,
        requires_merchant: false,
        min_score: 5,
    },
    // ── Spending comparison ───────────────────────────────────────
    Template {
        intent: SearchIntent::SpendingComparison,
        card_type: CardType::Comparison,
        keywords: &[
            ("compare", 8), ("vs", 8), ("versus", 8), ("compared to", 8),
            ("difference", 5), ("trend", 5), ("more than last", 6),
            ("less than last", 6), ("this month vs", 10), ("month over month", 8),
        ],
        anti_keywords: &[],
        requires_category: false,
        requires_merchant: false,
        min_score: 5,
    },
    // ── Transaction search (merchant) ─────────────────────────────
    Template {
        intent: SearchIntent::TransactionSearch,
        card_type: CardType::TransactionList,
        keywords: &[
            ("at ", 3), ("transactions", 5), ("show me", 4), ("show", 3),
            ("find", 4), ("list", 4), ("purchases", 4), ("payments", 4),
            ("charges", 4), ("over $", 6), ("under $", 6), ("above $", 6),
            ("more than $", 6), ("less than $", 6),
        ],
        anti_keywords: &["budget", "balance", "worth", "subscribe", "recurring"],
        requires_category: false,
        requires_merchant: false,
        min_score: 5,
    },
    // ── General spending total ────────────────────────────────────
    Template {
        intent: SearchIntent::SpendingTotal,
        card_type: CardType::Total,
        keywords: &[
            ("spend", 5), ("spent", 5), ("how much", 4), ("total", 3),
            ("expenses", 4), ("cost", 3), ("per year", 4), ("per month", 4),
            ("annually", 4), ("monthly", 3),
        ],
        anti_keywords: &["earn", "income", "balance", "worth", "budget", "subscribe", "recurring"],
        requires_category: false,
        requires_merchant: false,
        min_score: 5,
    },
    // ── Income total ──────────────────────────────────────────────
    Template {
        intent: SearchIntent::IncomeTotal,
        card_type: CardType::Total,
        keywords: &[
            ("earn", 6), ("earned", 6), ("income", 7), ("salary", 6),
            ("made", 4), ("revenue", 5), ("pay", 3), ("paycheck", 6),
        ],
        anti_keywords: &["spend", "spent", "expense", "budget"],
        requires_category: false,
        requires_merchant: false,
        min_score: 5,
    },
    // ── Balance check ─────────────────────────────────────────────
    Template {
        intent: SearchIntent::BalanceCheck,
        card_type: CardType::AccountSummary,
        keywords: &[
            ("balance", 7), ("how much do i have", 9), ("how much money", 8),
            ("in my account", 6), ("available", 5), ("checking", 5),
            ("savings", 5), ("what's in my", 7),
        ],
        anti_keywords: &["spend", "budget", "earn"],
        requires_category: false,
        requires_merchant: false,
        min_score: 5,
    },
    // ── Net worth ─────────────────────────────────────────────────
    Template {
        intent: SearchIntent::NetWorth,
        card_type: CardType::AccountSummary,
        keywords: &[
            ("net worth", 10), ("total assets", 8), ("all accounts", 7),
            ("worth", 5), ("across all", 6), ("everything", 4),
        ],
        anti_keywords: &["spend", "budget"],
        requires_category: false,
        requires_merchant: false,
        min_score: 5,
    },
    // ── Top merchants / category breakdown ────────────────────────
    Template {
        intent: SearchIntent::TopMerchants,
        card_type: CardType::Breakdown,
        keywords: &[
            ("where does my money go", 10), ("top merchants", 9),
            ("biggest payees", 8), ("who do i pay", 7),
            ("where do i spend", 8), ("where do i shop", 7),
            ("breakdown", 6), ("by category", 6), ("spending breakdown", 8),
            ("category breakdown", 8),
        ],
        anti_keywords: &[],
        requires_category: false,
        requires_merchant: false,
        min_score: 5,
    },
    // ── Top expenses ──────────────────────────────────────────────
    Template {
        intent: SearchIntent::TopExpenses,
        card_type: CardType::TransactionList,
        keywords: &[
            ("biggest", 6), ("largest", 6), ("most expensive", 8),
            ("top spending", 7), ("biggest expense", 8), ("biggest purchase", 8),
            ("largest transaction", 8), ("highest", 5),
        ],
        anti_keywords: &["category", "merchant", "payee"],
        requires_category: false,
        requires_merchant: false,
        min_score: 5,
    },
    // ── Recurring / subscriptions ─────────────────────────────────
    Template {
        intent: SearchIntent::RecurringCheck,
        card_type: CardType::TransactionList,
        keywords: &[
            ("subscription", 8), ("subscriptions", 8), ("recurring", 8),
            ("monthly bills", 8), ("upcoming bills", 7), ("bills", 5),
            ("auto-pay", 7), ("autopay", 7),
        ],
        anti_keywords: &[],
        requires_category: false,
        requires_merchant: false,
        min_score: 5,
    },
    // ── Category detail ───────────────────────────────────────────
    Template {
        intent: SearchIntent::CategoryDetail,
        card_type: CardType::Breakdown,
        keywords: &[
            ("breakdown", 5), ("detail", 5), ("details", 5),
            ("drilling into", 5), ("drill into", 5),
        ],
        anti_keywords: &[],
        requires_category: true,
        requires_merchant: false,
        min_score: 5,
    },
];

pub fn detect_intent(query: &str, entities: &ExtractedEntities) -> Option<(IntentMatch, CardType)> {
    let normalized = query.trim().to_lowercase().replace('?', "");

    let mut best: Option<(&Template, u16)> = None;

    for template in TEMPLATES {
        // Check entity requirements
        if template.requires_category && entities.category_name.is_none() {
            continue;
        }
        if template.requires_merchant && entities.merchant.is_none() {
            continue;
        }

        // Score this template
        let mut score: u16 = 0;
        let mut anti_hit = false;

        for (keyword, weight) in template.keywords {
            if normalized.contains(keyword) {
                score += *weight as u16;
            }
        }

        for anti in template.anti_keywords {
            if normalized.contains(anti) {
                anti_hit = true;
                break;
            }
        }

        // Anti-keywords reduce but don't eliminate (unless score is marginal)
        if anti_hit && score < 8 {
            continue;
        }

        if score < template.min_score as u16 {
            continue;
        }

        match &best {
            None => best = Some((template, score)),
            Some((_, prev_score)) => {
                if score > *prev_score {
                    best = Some((template, score));
                }
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

    fn entities_with_housing() -> ExtractedEntities {
        ExtractedEntities {
            category_name: Some("Housing".to_string()),
            category_id: Some("c2".to_string()),
            ..Default::default()
        }
    }

    fn entities_with_healthcare() -> ExtractedEntities {
        ExtractedEntities {
            category_name: Some("Healthcare".to_string()),
            category_id: Some("c3".to_string()),
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
    fn test_how_much_per_year_apartment() {
        // "How much per year is my apartment" — should match spending
        let entities = ExtractedEntities {
            category_name: Some("Housing".to_string()),
            category_id: Some("c2".to_string()),
            ..Default::default()
        };
        let (m, _) = detect_intent("how much per year is my apartment", &entities).unwrap();
        assert_eq!(m.intent, SearchIntent::SpendingByCategory);
    }

    #[test]
    fn test_budget_left_in_healthcare() {
        let (m, ct) = detect_intent("how much budget do i have left in healthcare", &entities_with_healthcare()).unwrap();
        assert_eq!(m.intent, SearchIntent::BudgetStatus);
        assert_eq!(ct, CardType::BudgetStatus);
    }

    #[test]
    fn test_over_budget_in_housing() {
        let (m, ct) = detect_intent("am i over budget in housing", &entities_with_housing()).unwrap();
        assert_eq!(m.intent, SearchIntent::BudgetStatus);
        assert_eq!(ct, CardType::BudgetStatus);
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

    #[test]
    fn test_informal_spending_question() {
        // "what did i pay for food" — informal phrasing
        let entities = ExtractedEntities {
            category_name: Some("Food".to_string()),
            category_id: Some("c1".to_string()),
            ..Default::default()
        };
        let (m, _) = detect_intent("what did i pay for food", &entities).unwrap();
        assert_eq!(m.intent, SearchIntent::SpendingByCategory);
    }

    #[test]
    fn test_budget_remaining() {
        let (m, ct) = detect_intent("how much budget remaining for dining", &entities_with_category()).unwrap();
        assert_eq!(m.intent, SearchIntent::BudgetStatus);
        assert_eq!(ct, CardType::BudgetStatus);
    }

    #[test]
    fn test_monthly_bills() {
        let (m, _) = detect_intent("show me my monthly bills", &empty_entities()).unwrap();
        assert_eq!(m.intent, SearchIntent::RecurringCheck);
    }
}
