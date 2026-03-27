use serde::Serialize;

/// A curated, pre-verified agent available for one-click install.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CuratedAgent {
    pub id: &'static str,
    pub name: &'static str,
    pub publisher: &'static str,
    pub description: &'static str,
    pub version: &'static str,
    pub verified: bool,
    pub requested_scopes: &'static [&'static str],
}

/// The curated agent directory — hardcoded for G4, will move to a
/// remote registry in future versions.
pub const CURATED_AGENTS: &[CuratedAgent] = &[
    CuratedAgent {
        id: "pf-netflix-analyzer",
        name: "Netflix Spend Analyzer",
        publisher: "PrivacyFinance Labs",
        description: "Analyzes your Netflix and streaming charges, suggests plan optimizations.",
        version: "1.0",
        verified: true,
        requested_scopes: &[
            "transactions:read:category:streaming",
            "transactions:read:date:last_90_days",
        ],
    },
    CuratedAgent {
        id: "pf-dining-tracker",
        name: "Dining Spend Tracker",
        publisher: "PrivacyFinance Labs",
        description: "Tracks your dining and restaurant spending over time with trend analysis.",
        version: "1.0",
        verified: true,
        requested_scopes: &[
            "transactions:read:category:dining",
        ],
    },
    CuratedAgent {
        id: "pf-budget-optimizer",
        name: "Budget Optimizer",
        publisher: "PrivacyFinance Labs",
        description: "Analyzes your spending vs budget allocations and suggests optimizations.",
        version: "1.0",
        verified: true,
        requested_scopes: &[
            "transactions:read",
            "budget:read",
        ],
    },
    CuratedAgent {
        id: "pf-income-insights",
        name: "Income Insights",
        publisher: "PrivacyFinance Labs",
        description: "Tracks your income patterns and provides monthly comparisons.",
        version: "1.0",
        verified: true,
        requested_scopes: &[
            "income:read",
        ],
    },
];

/// Get all curated agents.
pub fn get_curated_directory() -> Vec<CuratedAgent> {
    CURATED_AGENTS.to_vec()
}
