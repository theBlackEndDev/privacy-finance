# PrivacyFinance AI Search — Implementation Brief
**For: Claude Code**
**Purpose: Add natural language search to PrivacyFinance so users can query their own financial data conversationally**
**Version: 1.0**

---

## Context

PrivacyFinance is a local-first, offline personal finance app built with Tauri (Rust backend + React frontend) and SQLite. Users track accounts, transactions, budgets, and categories.

Read these files before planning:
- `STATE.md` (current app state)
- `POST-MVP-PLAN.md` (existing feature roadmap — Phase 12 covers FTS5 search)
- `src-tauri/src/` (Rust backend structure)
- `src/` (React frontend structure)

---

## What We're Building

A natural language search feature that lets users ask questions about their own financial data in plain English and get instant answers. No external services. No cloud. Everything runs on-device.

**Examples of what a user should be able to type:**

```
"How much did I spend on groceries last month?"
→ $847.23 across 23 transactions

"What's my biggest expense this year?"
→ Rent: $18,000 (12 transactions)

"Show me all transactions over $200 in March"
→ [filtered transaction list]

"Am I over budget on dining?"
→ Yes — you've spent $420 of your $300 dining budget (140%)

"How much did I spend at Amazon this quarter?"
→ $1,247.89 across 34 transactions

"What are my recurring subscriptions?"
→ Netflix $15.99/mo, Spotify $10.99/mo, iCloud $2.99/mo...

"Compare my spending this month vs last month"
→ This month: $3,200 | Last month: $2,890 | +$310 (+10.7%)

"How much money do I have across all accounts?"
→ Total: $12,450.23 (Checking: $8,200, Savings: $4,250.23)
```

---

## Design Principles

1. **It's a search bar, not a chatbot.** Users type a question, get an answer. No conversation history, no back-and-forth, no "how can I help you today?" vibes.

2. **Answers are data, not opinions.** The AI translates the question into a database query. It returns facts from the user's data — not financial advice, not predictions, not recommendations.

3. **100% local. No exceptions.** All processing happens on-device. No network requests. No telemetry. No cloud inference. The user's financial data never leaves their machine.

4. **Fast by default, smart when needed.** Most queries are handled by template matching (instant). Only ambiguous queries fall through to a local model (if installed). Users should never wait more than 1-2 seconds for an answer.

5. **Integrated into the existing search bar.** This isn't a separate feature or tab. It enhances the search that already exists (or will exist via Phase 12 FTS5). When a user types something that looks like a question, the AI search kicks in. When they type a keyword, the existing text search handles it.

---

## Two-Tier Inference Architecture

### Tier 1 — Template Matching (default, zero overhead, always available)

A pattern-matching engine that maps common financial questions to SQL queries. No ML model required. Runs on any device instantly.

**How it works:**

```
User input: "how much did I spend on groceries last month"
    ↓
Intent detection (regex/keyword): SPENDING_TOTAL
Entity extraction: category="groceries", period="last_month"
    ↓
SQL generation:
  SELECT SUM(ABS(amount)) as total, COUNT(*) as count
  FROM transactions
  WHERE category_id = (SELECT id FROM categories WHERE LOWER(name) = 'groceries')
  AND date >= '2026-03-01' AND date < '2026-04-01'
  AND kind = 'expense'
    ↓
Result formatting: "$847.23 across 23 transactions"
```

**Supported intent categories (Tier 1):**

| Intent | Example queries |
|--------|----------------|
| `SPENDING_TOTAL` | "how much did I spend on X", "total spending on X" |
| `SPENDING_BY_CATEGORY` | "breakdown by category", "where does my money go" |
| `SPENDING_COMPARISON` | "compare this month vs last month", "spending trend" |
| `INCOME_TOTAL` | "how much did I earn", "total income this month" |
| `BALANCE_CHECK` | "how much money do I have", "account balances" |
| `BUDGET_STATUS` | "am I over budget on X", "budget remaining for X" |
| `TRANSACTION_SEARCH` | "show me transactions over $X", "find payments to Y" |
| `TOP_MERCHANTS` | "where do I spend the most", "top merchants" |
| `TOP_EXPENSES` | "biggest expense", "largest transaction" |
| `RECURRING_CHECK` | "what are my subscriptions", "recurring charges" |
| `NET_WORTH` | "what's my net worth", "total across all accounts" |
| `CATEGORY_DETAIL` | "how much on dining this quarter", "entertainment spending" |

**Entity extraction (Tier 1):**

| Entity | Examples | How to extract |
|--------|----------|---------------|
| Category | "groceries", "dining", "rent" | Match against user's category names (fuzzy match) |
| Merchant/Payee | "Amazon", "Starbucks" | Match against transaction payees |
| Time period | "last month", "this year", "March", "Q1" | Keyword → date range mapping |
| Amount threshold | "over $200", "under $50", "more than $1000" | Regex for dollar amounts + comparators |
| Account | "checking", "savings", "Chase card" | Match against user's account names |

**Date period mapping:**

| Input | Resolves to |
|-------|-------------|
| "today" | Current date |
| "this week" | Monday → Sunday of current week |
| "last week" | Previous Monday → Sunday |
| "this month" | 1st → last day of current month |
| "last month" | 1st → last day of previous month |
| "this quarter" / "Q1" / "Q2" etc. | Quarter start → end |
| "this year" | Jan 1 → Dec 31 of current year |
| "last year" | Previous year Jan 1 → Dec 31 |
| "March" / "January" etc. | 1st → last day of that month (current year, or last year if month is in the future) |
| "last 30 days" / "past 90 days" | Rolling window from today |

---

### Tier 2 — Small Local Model (optional install, for complex queries)

When Tier 1 can't match the query (no template hits), the user can optionally install a small local model that handles more complex or ambiguous questions.

**Model:** Phi-3.5-mini (3.8B params, ~2GB download)
**Runtime:** Ollama (user installs separately) or bundled llama.cpp
**Hardware:** Runs on CPU, 4GB+ RAM, no GPU required
**Response time:** 2-5 seconds on modest hardware

**What Tier 2 handles that Tier 1 can't:**
- Ambiguous queries: "am I spending too much?" → needs to infer "compared to what?"
- Multi-step queries: "what percentage of my income goes to housing?" → needs income total + housing total + division
- Relative queries: "did I spend more on food or entertainment?" → needs both totals + comparison
- Unusual phrasing: "where's all my money going?" → map to category breakdown

**Tier 2 pipeline:**
```
User input: "what percentage of my income goes to housing?"
    ↓
Tier 1 template matching: NO MATCH
    ↓
Tier 2 local model:
  System prompt: "You are a SQL query generator for a personal finance SQLite database.
  Given a natural language question about the user's finances, generate one or more
  SQL queries that answer the question. Return ONLY valid SQL and a brief result
  format description. Available tables: transactions, accounts, categories, budgets..."
    ↓
Model output:
  Query 1: SELECT SUM(ABS(amount)) FROM transactions WHERE kind = 'income' AND date >= [period]
  Query 2: SELECT SUM(ABS(amount)) FROM transactions WHERE category_id IN (SELECT id FROM categories WHERE LOWER(name) LIKE '%housing%' OR LOWER(name) LIKE '%rent%' OR LOWER(name) LIKE '%mortgage%') AND date >= [period]
  Format: "percentage = query2 / query1 * 100"
    ↓
Execute queries against SQLite
    ↓
Format result: "32% of your income goes to housing ($1,600 of $5,000)"
```

**When Tier 2 is not installed:**
If the user types something Tier 1 can't match, show helpful suggestions:
```
"I'm not sure what you're asking. Try one of these:"
• "How much did I spend on [category]?"
• "Show me transactions over $[amount]"
• "Am I over budget on [category]?"
• "Compare this month vs last month"

Want smarter search? Install the AI Search add-on in Settings.
```

---

## UI Integration

### Where it lives: the existing search bar

The search bar in the app header handles both keyword search (existing FTS5 from Phase 12) and natural language queries (this feature).

**How to distinguish:**
- Starts with a question word (who, what, where, when, how, why, did, am, is, are, do, show, compare, find) → AI search
- Contains a question mark → AI search
- Everything else → keyword/FTS5 search

### Search bar behavior

```
┌─────────────────────────────────────────────────┐
│ 🔍  How much did I spend on groceries last month│
└─────────────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────────────┐
│                                                 │
│  🍎 Groceries — Last Month                     │
│                                                 │
│  Total spent:     $847.23                       │
│  Transactions:    23                            │
│  Daily average:   $28.24                        │
│  vs. previous month: +$52.10 (+6.6%)           │
│                                                 │
│  Top merchants:                                 │
│  • Kroger           $312.45 (15 transactions)   │
│  • Costco           $287.30 (3 transactions)    │
│  • Whole Foods      $247.48 (5 transactions)    │
│                                                 │
│  [View All Transactions →]                      │
│                                                 │
└─────────────────────────────────────────────────┘
```

### Answer card types

Different query types produce different result cards:

**1. Total card** (spending total, income total, balance)
- Big number headline
- Transaction count
- Comparison to previous period (if available)
- Top merchants/payees
- Link to view matching transactions

**2. Comparison card** (this month vs last, category vs category)
- Side-by-side numbers
- Delta (absolute + percentage)
- Mini bar chart or sparkline

**3. Budget card** (budget status queries)
- Category name + budget amount
- Spent so far
- Remaining / over amount
- Progress bar (green/yellow/red)

**4. Transaction list card** (find/show queries)
- Filtered transaction list inline
- Link to open in full transactions view with filter applied

**5. Breakdown card** (category breakdown, top expenses)
- Ranked list of categories with amounts
- Percentage of total
- Mini horizontal bars

**6. Account summary card** (balance check, net worth)
- Per-account balances
- Total across all accounts

### Loading state

For Tier 1 (instant): no loading indicator needed.
For Tier 2 (2-5 seconds): show a subtle "Thinking..." indicator below the search bar.

### Error/no-result state

If neither tier can answer:
```
┌─────────────────────────────────────────────────┐
│  Couldn't find an answer for that.              │
│                                                 │
│  Try asking:                                    │
│  • "How much did I spend on [category]?"        │
│  • "Show me transactions over $[amount]"        │
│  • "Am I over budget on [category]?"            │
│                                                 │
└─────────────────────────────────────────────────┘
```

---

## Technical Implementation

### New Rust modules

```
src-tauri/src/
  models/
    ai_search.rs         — SearchQuery, SearchResult, SearchIntent, Entity structs
  commands/
    ai_search.rs         — main entry point: parse_and_execute_query()
    ai_templates.rs      — Tier 1 template matching engine
    ai_entities.rs       — entity extraction (categories, dates, amounts, merchants)
    ai_sql_builder.rs    — builds parameterized SQL from parsed intent + entities
    ai_local_model.rs    — Tier 2 Ollama/llama.cpp integration (optional)
    ai_result_formatter.rs — formats raw SQL results into structured answer cards
```

### New React components

```
src/
  components/
    search/
      AISearchResults.tsx      — container for answer cards
      TotalCard.tsx            — spending/income/balance results
      ComparisonCard.tsx       — side-by-side comparisons
      BudgetStatusCard.tsx     — budget check results
      TransactionListCard.tsx  — filtered transaction results
      BreakdownCard.tsx        — category/merchant breakdowns
      AccountSummaryCard.tsx   — account balance overview
      SearchSuggestions.tsx     — fallback suggestions when no match
      SearchLoading.tsx        — Tier 2 loading indicator
  hooks/
    useAISearch.ts             — manages search state, dispatches to backend
  api/
    ai_search.ts               — Tauri invoke wrappers
```

### Tauri commands

```rust
// Main search entry point
#[tauri::command]
fn ai_search(query: String) -> Result<SearchResult, String>

// Check if local model is available
#[tauri::command]
fn ai_model_status() -> Result<ModelStatus, String>

// Install/configure local model (Settings UI)
#[tauri::command]
fn ai_model_configure(model_path: String) -> Result<(), String>
```

### SearchResult struct

```rust
pub struct SearchResult {
    pub card_type: CardType,        // total, comparison, budget, transactions, breakdown, summary
    pub title: String,              // "Groceries — Last Month"
    pub primary_value: String,      // "$847.23"
    pub secondary_values: Vec<KeyValue>,  // transaction count, daily average, etc.
    pub comparison: Option<Comparison>,   // vs previous period
    pub items: Vec<ResultItem>,     // top merchants, category breakdown, transactions
    pub filter_link: Option<TransactionFilter>,  // "View All Transactions" link
    pub inference_tier: u8,         // 1 or 2 — which tier answered
    pub confidence: f32,            // 0.0-1.0 — how confident the match was
}
```

---

## Settings UI

### Settings → Search

```
┌─────────────────────────────────────────────────┐
│  Search Settings                                │
│                                                 │
│  Natural Language Search           [Enabled ✓]  │
│  Ask questions about your data in plain English │
│                                                 │
│  ─────────────────────────────────────────────  │
│                                                 │
│  AI Search Add-on                               │
│  For complex questions that basic search can't  │
│  handle. Runs entirely on your device.          │
│                                                 │
│  Status: Not installed                          │
│  Size: ~2GB download                            │
│  Requires: 4GB+ RAM                             │
│                                                 │
│  [Install AI Search Add-on]                     │
│                                                 │
│  ─────────────────────────────────────────────  │
│                                                 │
│  Privacy: All search processing runs locally.   │
│  Your data never leaves your device.            │
│                                                 │
└─────────────────────────────────────────────────┘
```

---

## What to Produce

Given this brief, produce a **detailed implementation plan** with:

1. **Phased rollout** — what's shippable first, what builds on it
2. **Task breakdown per phase** — concrete, small, completable tasks (10-30 min each)
3. **Tier 1 template engine spec** — full list of templates, regex patterns, SQL generators
4. **Entity extraction spec** — how categories, dates, amounts, and merchants are parsed
5. **Answer card component specs** — props, states, layouts for each card type
6. **Tier 2 integration plan** — how Ollama/llama.cpp is called, prompt engineering, SQL validation
7. **Testing strategy** — example queries and expected results at each phase
8. **Integration with existing Phase 12 (FTS5)** — how AI search and keyword search coexist

Focus on Tier 1 first. Tier 2 is a follow-up phase.

---

## Constraints

- Must work offline — no network requests, ever
- Must not break existing app functionality
- Search results must return in <500ms for Tier 1, <5s for Tier 2
- Must work on macOS (primary beta target)
- Template matching must be extensible — easy to add new patterns without rewriting the engine
- SQL queries must be parameterized — no injection risk from user input
- Answer cards must match the existing PrivacyFinance design language (clean, minimal, dark/light mode)
