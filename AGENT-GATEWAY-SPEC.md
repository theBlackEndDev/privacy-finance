# PrivacyFinance Agent Gateway — Product Spec
**Version 0.1 — Working Draft**
*Last updated: 2026-03-27*

---

## 1. Concept

PrivacyFinance is no longer just a personal finance tracker. It becomes **the privacy-first gateway that AI agents must pass through to access your financial data** — with your explicit consent, your rules, and your audit trail.

The core shift: instead of AI agents scraping your financial data through a Plaid link or API with no controls, PrivacyFinance becomes the intermediary layer. You approve which agents can see what. Data stays on-device. Agents get exactly what they need — nothing more.

---

## 2. The Problem

AI agents need financial context to be genuinely useful. But today:

| Today | Reality |
|-------|---------|
| Agent wants spending context | You hand over your Plaid credentials or export a CSV |
| Agent sees everything | There's no concept of "this agent can see my Netflix subscription but not my medical bills" |
| Data trains the model | Even when you use an AI tool, your financial data may be used for training |
| Access is binary | Either the agent sees everything or nothing |
| No audit trail | You have no log of what the agent looked at |

The agents are coming. They will need financial data. The question is whether users have a tool to control that access — or whether it becomes a free-for-all.

---

## 3. Vision Statement

> *PrivacyFinance is to AI agents what 1Password is to passwords — the trusted layer between you and every tool that wants access to your financial life.*

---

## 4. Core Features

### 4.1 Agent Directory & Registry

**What it is:** A list of AI agents that have requested access to your data.

**Agent metadata format (manifest):**
```json
{
  "agent_id": "privacy-finance-agent://netflix-spend-analyzer",
  "name": "Netflix Spend Analyzer",
  "publisher": "PrivacyFinance Labs",
  "version": "1.0",
  "description": "Analyzes your Netflix charges and suggests plan optimizations.",
  "requested_scopes": ["transactions:read:category:streaming", "transactions:read:date:last_30_days"],
  "privacy_policy_url": "https://example.com/privacy"
}
```

**Key rule:** Agents must declare all scopes upfront. They cannot request data outside their declared scopes.

---

### 4.2 Scoped API Keys

**What it is:** A key per agent, scoped to specific data access permissions.

**Key structure:**
```
pf_live_sk_[agent_id_hash]_[scope_hash]_[timestamp]
```

**Scope examples:**
| Scope | Meaning |
|-------|---------|
| `transactions:read` | Read all transactions |
| `transactions:read:category:X` | Transactions in category X only |
| `transactions:read:date:last_N_days` | Transactions in last N days |
| `transactions:read:amount:under_N` | Transactions under $N |
| `income:read` | Income transactions only |
| `accounts:read:balances` | Account balances only |
| `budget:read` | Budget allocations |
| `insights:generate` | Request AI-generated insights (runs locally) |

**What users control:**
- Per-agent on/off toggle
- Per-scope grants (逐项批准 or deny each scope)
- Expiry date (agent key auto-revokes after N days)
- Data ceiling (agent can only see up to $X in transactions)

---

### 4.3 Local Inference Engine

**What it is:** Two-tier on-device inference — template matching by default, optional small model for complex queries.

**Tier 1 — Template Matching (default, zero overhead):**

The 80% case: common financial queries are handled by direct pattern → SQL mapping with no ML model.

Example:
- Query: "what did I spend on restaurants last month"
- Template maps: `SELECT SUM(amount) FROM transactions WHERE category = 'Restaurants' AND date >= [first of last month] AND kind = 'expense'`
- Result: `$847.23`
- Zero model overhead — runs on any device instantly

**Tier 2 — Small Local Model (optional install):**

For queries that don't match a template, an optional local model handles translation.

- Recommended: **Phi-3.5-mini** (~2GB install)
- Runs via Ollama, entirely on-device
- Falls back to template suggestions if it can't resolve the query
- No GPU required, works on 4GB+ RAM

**Privacy guarantee:**
> No cloud. No network requests for inference. Raw transaction data never leaves the device under any circumstances.

---

### 4.4 Consent & Approval Flow

**What it is:** Every agent request requires explicit user approval (unless user sets "trust this agent permanently").

**Approval states:**
| State | Meaning |
|-------|---------|
| `pending` | Awaiting user approval |
| `approved_once` | Approved for this request only |
| `approved_permanent` | Approved until revoked manually |
| `denied` | Rejected |
| `expired` | Key expired (auto after expiry date) |

---

### 4.5 Audit Trail

**What it is:** A complete log of every access request, what was approved, what was returned.

**Logged for each request:**
- Timestamp
- Agent name + version
- Scope requested vs. scope granted
- Status (approved/denied)
- Result summary (e.g., "12 transactions, $847.23 total")
- Always `localhost` — data never left the device

**User-visible:** "Agent Activity" section in Settings, per-agent activity log, exportable to CSV/JSON.

---

### 4.6 Agent SDK / Integration

```typescript
import { PrivacyFinanceAgent } from '@privacyfinance/agent-sdk';

const pf = new PrivacyFinanceAgent({
  gateway: 'http://localhost:3847',
  agentId: 'my-agent-id'
});

const status = await pf.checkAccess(['transactions:read:category:streaming']);
if (status.granted) {
  const spending = await pf.query({
    scope: 'transactions:read:category:streaming',
    period: 'last_30_days'
  });
  // { total: 54.99, count: 1 }
}
```

---

## 5. User Stories

### Story 1: First-time agent access
> Jason wants to use a "Netflix Spend Analyzer" agent.
>
> 1. Jason pastes the agent's manifest URL into PrivacyFinance
> 2. PrivacyFinance shows: agent name, publisher, what it wants to access
> 3. Jason reviews: "Streaming transactions, last 90 days" — ✅ looks fine
> 4. Jason clicks "Approve"
> 5. PrivacyFinance generates a scoped API key
> 6. The agent uses the key, queries streaming spending — Jason sees it in his audit log
> 7. Done. Jason can revoke at any time.

### Story 2: Unexpected scope request
> A familiar agent suddenly requests access to income data — something it never needed before.
>
> 1. PrivacyFinance shows a macOS notification: "BudgetPro Agent requesting new scope: income:read"
> 2. Jason opens PrivacyFinance, sees the request
> 3. Jason clicks "Deny" and "Revoke Agent"
> 4. BudgetPro's key is invalidated immediately

### Story 3: Local insight without raw data
> Jason asks his "Financial Advisor" agent: "Am I spending more on dining out than I should?"
>
> 1. Agent calls `pf://infer`
> 2. PrivacyFinance local inference engine computes dining spending vs. budget
> 3. Returns: "You're spending $420/mo on dining, which is 140% of your $300 budget."
> 4. Raw transactions never left PrivacyFinance

---

## 6. Technical Architecture

### 6.1 Stack
- **Platform:** Tauri (existing PrivacyFinance app)
- **Database:** SQLite (existing, local)
- **Agent gateway:** Local HTTP server on `localhost` port 3847
- **Local inference:** Tier 1 template matching (default) + optional Tier 2 Phi-3.5-mini via Ollama
- **Agent protocol:** REST over localhost — simple, debuggable
- **Agent discovery:** Manifest URL scheme (`privacy-finance-agent://`)

### 6.2 Database Schema (new tables)

```sql
-- Agent registry
CREATE TABLE agents (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  publisher TEXT NOT NULL,
  version TEXT NOT NULL,
  description TEXT,
  manifest_url TEXT,
  status TEXT NOT NULL DEFAULT 'pending'
    CHECK(status IN ('pending','approved','denied','revoked')),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

-- Scoped API keys
CREATE TABLE agent_keys (
  id TEXT PRIMARY KEY,
  agent_id TEXT NOT NULL,
  key_hash TEXT NOT NULL UNIQUE,
  scopes TEXT NOT NULL,
  expires_at TEXT,
  last_used_at TEXT,
  created_at TEXT NOT NULL,
  FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE
);

-- Per-scope grants
CREATE TABLE agent_scope_grants (
  id TEXT PRIMARY KEY,
  agent_id TEXT NOT NULL,
  scope TEXT NOT NULL,
  granted INTEGER NOT NULL DEFAULT 0,
  granted_at TEXT,
  granted_by TEXT,
  FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE,
  UNIQUE(agent_id, scope)
);

-- Audit log
CREATE TABLE agent_audit_log (
  id TEXT PRIMARY KEY,
  agent_id TEXT NOT NULL,
  scope_requested TEXT NOT NULL,
  scope_granted TEXT NOT NULL,
  status TEXT NOT NULL,
  result_summary TEXT,
  ip_address TEXT,
  created_at TEXT NOT NULL,
  FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE SET NULL
);

CREATE INDEX idx_audit_agent ON agent_audit_log(agent_id);
CREATE INDEX idx_audit_created ON agent_audit_log(created_at);
```

### 6.3 Local Gateway Endpoints

```
GET  /v1/agents              — list registered agents
POST /v1/agents/register     — register new agent (requires user approval)
GET  /v1/agents/:id/status  — check agent status + granted scopes
POST /v1/agents/:id/request — request access to specific scopes
GET  /v1/query              — query data (requires granted scope)
POST /v1/infer               — local inference (natural language → result)
GET  /v1/audit               — read audit log
```

Auth: `Authorization: Bearer <key>` header. CORS: localhost only.

### 6.4 Scope Definitions

| Scope Pattern | Matches |
|--------------|---------|
| `transactions:read` | All transactions |
| `transactions:read:category:X` | Transactions in category X |
| `transactions:read:date:last_N_days` | Transactions in last N days |
| `transactions:read:amount:under_N` | Transactions under $N |
| `transactions:read:amount:over_N` | Transactions over $N |
| `income:read` | Income transactions only |
| `accounts:read:balances` | Account balances only |
| `budget:read` | Budget allocations |
| `budget:read:category:X` | Budget for specific category |
| `insights:generate` | Request local inference |

### 6.5 Privacy Model

**No cloud. No exceptions.**

All inference runs 100% on-device. There is no cloud fallback, no query routing to external servers, and no mode where raw financial data leaves the user's machine.

| Data | Who sees it |
|------|-------------|
| Raw transactions | Only PrivacyFinance local engine |
| Aggregated results | Agent (e.g., "you spent $412 on dining") |
| Account balances | Only if `accounts:read:balances` scope granted |
| Audit log | User only |
| Nothing | Ever, outside the device |

### 6.6 Inference Tiers

**Tier 1 — Template Matching (default, always on)**

The 80% case: common financial queries are handled by pattern → SQL mapping with zero ML overhead.

Examples:
```
"what did I spend on dining"     → SELECT SUM(amount) WHERE category = 'Dining'
"how much did I earn this month" → SELECT SUM(amount) WHERE kind = 'income'
"show transactions over $500"    → SELECT * WHERE amount > 50000
```

Covers: spending totals, income totals, category breakdowns, date range queries, amount thresholds, merchant lookups, budget comparisons.

**Tier 2 — Small Local Model (optional install)**

For queries that don't match a template, an optional small quantized model can be installed.

Recommended model: **Phi-3.5-mini** (3.8B params, ~2GB install)
- Runs entirely on-device
- Translates natural language → SQL
- Falls back to template suggestions when it can't resolve the query
- User opts in via Settings → "Install Local Inference Model"

No GPU required. Works on 4GB+ RAM devices.

**Privacy guarantee:** Inference is fully local. No network requests for inference. No data leaves the device under any circumstances.

---

## 7. Competitive Moat

| Factor | Why competitors can't easily copy it |
|--------|-------------------------------------|
| Local-first | Plaid, MX, Copilot all route data through their servers. PrivacyFinance keeps data on-device. |
| Scoped keys | No existing personal finance tool has an agent permission system. |
| User-controlled audit | Existing tools log access server-side (for their benefit, not yours). |
| Local inference | Technical foundation already being built in the Omi Bridge work. |
| Trust moat | Once users adopt PrivacyFinance as their agent gateway, switching costs are high. |

---

## 8. Phases

### Phase G1: Foundation (Gateway Core)
- Local HTTP server on localhost with agent key auth
- Agent registry (CRUD)
- Scoped key generation + validation
- Basic scope enforcement
- Agent directory UI in Settings
- Basic audit log

### Phase G2: Approval Flow
- Agent manifest parsing
- Approval/revocation UI
- macOS notifications for new agent requests
- Per-scope grant/deny UI

### Phase G3: Local Inference
- Template matching engine (Tier 1) — covers 80% of queries, no model required
- Phi-3.5-mini integration as optional Tier 2 install
- Fallback: when template doesn't match and no model installed, return structured suggestions ("Did you mean: [query options]?")
- No cloud component. No exceptions.

### Phase G4: Agent Ecosystem
- Agent manifest registry
- "Verified by PrivacyFinance" badge
- Publisher documentation
- Developer SDK docs

---

## 9. Open Questions

1. ~~Bundled model or user-provided?~~ — **Decided: user-provided optional install. Tier 1 template matching is default (zero overhead). Tier 2 Phi-3.5-mini is an opt-in install.**
2. **Scope granularity** — is `transactions:read:category:streaming` fine-grained enough, or do we need field-level (e.g., no payee names)?
3. **Agent key rotation** — should keys auto-rotate? How does that interact with permanently approved agents?
4. **Cross-device** — if a user has PrivacyFinance on multiple devices, do agents need a key per device or can they sync?
5. ~~**Agent manifest hosting**~~ — ~~who hosts the manifest? What prevents manifest spoofing?~~
6. **Monetization** — free gateway for personal use, paid for agent publishers?
7. **Tier 2 model distribution** — direct download, or integrate with a model registry like HuggingFace?

---

## 10. Immediate Next Steps

1. **This week:** Sketch the Phase G1 data model — add the new tables to the existing PrivacyFinance schema
2. **Next week:** Build the local gateway HTTP server as a Rust crate in the Tauri app
3. **Week after:** Wire up the agent directory UI and key generation
4. **Before shipping:** Design the user explanation copy — "why does this agent want this?" needs to be crystal clear to non-technical users
