# PrivacyFinance Agent Gateway — Implementation Brief
**For: Claude Code**
**Purpose: Generate a detailed implementation plan (UX/UI + technical)**
**Version: 1.0**

---

## Context

You are building the PrivacyFinance Agent Gateway — a feature that makes PrivacyFinance the trusted intermediary layer between AI agents and the user's financial data. This is the product's major differentiation move.

Read these files before planning:
- `AGENT-GATEWAY-SPEC.md` (concept, architecture, scope definitions)
- `STATE.md` (current app state)
- `src-tauri/src/` (Rust backend structure)
- `src/` (React frontend structure)

---

## The Core Product Idea

PrivacyFinance becomes the "1Password for AI agents accessing your financial data."

AI agents need financial context to be useful. But today they get binary access to everything or nothing — no consent layer, no audit trail, no granular control. PrivacyFinance fixes this.

**The value prop:**
- You control which agents can access what
- Agents get scoped API keys, not raw data
- All inference runs on-device (no cloud)
- You see every access in an audit log

---

## Design Principles — READ THIS FIRST

The most important challenge is **permission UX**. Permissions must be:

1. **Effortless to understand** — a non-technical user must immediately get it
2. **Quick to act on** — approve/deny in 2 taps
3. **Transparent** — always clear what's happening and why
4. **Gradual** — power users can go deeper, casual users never have to

### The mental model to design around

> *"It's like when an app asks to access your camera. You see the app name, what it wants, and you tap Allow or Don't Allow. That's it."*

NOT: a technical permission matrix with checkboxes and scope documentation.

### Permission complexity spectrum

```
Casual user ─────────────────────────────────────── Power user
     │                                                     │
     ▼                                                     ▼
"Allow Netflix Analyzer                         "Customize exactly which
 to see my streaming                             categories this agent
 spending?"                                      can access, set an
[Allow] [Don't Allow]                            expiry, add a data
                                                 ceiling, etc."
```

Both get the same outcome. Both feel natural. The power user path just goes deeper when they want it.

---

## Feature Breakdown

### 1. Agent Registration

**Flow:**
1. User pastes an agent manifest URL or scans an agent QR code
2. PrivacyFinance fetches and parses the manifest
3. Shows: agent name, publisher, icon, description, and what it wants to access (in plain English)
4. User approves or denies

**Manifest example:**
```json
{
  "agent_id": "privacy-finance-agent://netflix-spend-analyzer",
  "name": "Netflix Spend Analyzer",
  "publisher": "PrivacyFinance Labs",
  "version": "1.0",
  "description": "Analyzes your Netflix charges and suggests plan optimizations.",
  "requested_scopes": [
    "transactions:read:category:streaming",
    "transactions:read:date:last_90_days"
  ]
}
```

**Plain English translation the UI shows:**
> *"Netflix Spend Analyzer wants to see your streaming subscription transactions from the last 90 days."*

**NOT:**
> *"Requested scopes: transactions:read:category:streaming, transactions:read:date:last_90_days"*

### 2. Permission Presentation

When an agent requests access, show:

```
┌─────────────────────────────────────────────┐
│  🔵 Netflix Spend Analyzer                  │
│     by PrivacyFinance Labs                  │
│                                             │
│  📋 What it wants:                          │
│  • Your Netflix / streaming charges         │
│    (last 90 days)                           │
│                                             │
│  🔒 How it works:                          │
│  • Runs on your device                      │
│  • Your data never leaves your Mac          │
│                                             │
│  [Allow This Once]  [Allow & Trust]  [Deny]│
└─────────────────────────────────────────────┘
```

**"Allow This Once"** = approve for this session only, next time it asks it shows the same prompt again
**"Allow & Trust"** = permanent approval until revoked
**"Deny"** = reject and remove the agent

### 3. Granular Permission Customization (power user path)

Accessible via "Customize" button or Settings → Agents → [Agent Name] → Permissions

```
┌─────────────────────────────────────────────┐
│  Netflix Spend Analyzer — Permissions       │
│                                             │
│  TRANSACTIONS                               │
│  ┌─────────────────────────────────────┐   │
│  │ Streaming subscriptions     [Always] │   │
│  │ All other transactions      [Never]  │   │
│  │ Amounts over $200          [Never]  │   │
│  └─────────────────────────────────────┘   │
│                                             │
│  DATA ACCESS                                │
│  ┌─────────────────────────────────────┐   │
│  │ Time window              [90 days]  │   │
│  │ Data ceiling           [Unlimited]  │   │
│  └─────────────────────────────────────┘   │
│                                             │
│  ADVANCED                                   │
│  ┌─────────────────────────────────────┐   │
│  │ Key expires after         [Never ▾] │   │
│  │ Require re-approval after    [30 days]│   │
│  └─────────────────────────────────────┘   │
│                                             │
│              [Save Permissions]              │
└─────────────────────────────────────────────┘
```

**Key UX decisions:**
- Defaults are pre-filled from the agent's manifest (what it asked for)
- User only sees what's relevant to this agent — don't show every possible scope, only the ones the agent uses
- Simple toggle for each: `[Always]` / `[Ask Each Time]` / `[Never]`
- Time window: slider or preset buttons ("Last 30 days / 90 days / 1 year / All time")
- Data ceiling: preset amounts or "Unlimited"

### 4. Agent Directory (Settings → Agents)

Shows all registered agents in a clean list:

```
┌─────────────────────────────────────────────┐
│  🤖 Agents                            [+ Add]│
│                                             │
│  ● Netflix Spend Analyzer    Active        │
│    PrivacyFinance Labs                      │
│    Last active: 2 hours ago               │
│    Access: Streaming, last 90 days         │
│    [Manage]  [Revoke]                      │
│                                             │
│  ○ BudgetPro Optimizer      Pending        │
│    BudgetPro Inc.                         │
│    Waiting for your approval               │
│    [Review Request]                        │
│                                             │
│  🔴 Old Agent                    Revoked  │
│    Removed — key invalidated               │
│    [Re-authorize]                          │
└─────────────────────────────────────────────┘
```

**Agent status badges:**
- **Active** (blue dot) — approved, key active
- **Pending** (yellow dot) — waiting for first approval
- **Revoked** (red dot) — user denied or revoked
- **Expired** (gray dot) — key expired naturally

### 5. Approval Notification (Shield Icon in Header)

A shield icon in the app header acts as the ambient agent status indicator:

```
[💰 Accounts] [📊 Budgets] [⚙️ Settings] [🔵🔒]
                                                   ── shield icon
```

- **Blue shield, no badge** — agents running normally, no pending items
- **Blue shield, yellow badge (1)** — one or more pending approvals
- **Red shield** — a trusted agent requested a scope outside its normal pattern

Click shield → dropdown:
```
┌──────────────────────────┐
│ Agent Activity           │
│                          │
│ 🔵 Netflix Analyzer      │
│ Queried streaming (90d)  │
│ Just now                  │
│                          │
│ 🟡 BudgetPro             │
│ Requesting new scope:    │
│ Income data               │
│ [Review →]               │
└──────────────────────────┘
```

### 6. Audit Log

Full history of every agent access, filterable:

```
┌─────────────────────────────────────────────────┐
│ Agent Activity Log                              │
│ [Filter: All Agents ▾]  [Last 30 days ▾]      │
│                                                 │
│ Mar 27 10:14  Netflix Analyzer  ✓ Streaming    │
│ Mar 27 09:02  Netflix Analyzer  ✓ Streaming    │
│ Mar 26 14:33  BudgetPro          ✓ Spending    │
│ Mar 26 11:07  BudgetPro          ✗ Income data │
│ Mar 25 16:45  Old Agent          ✓ Revoked    │
│                                                 │
│ [Export CSV]  [Export JSON]                     │
└─────────────────────────────────────────────────┘
```

Each entry expandable for detail:
```
Mar 27 10:14  Netflix Analyzer
Scope approved:  transactions:read:category:streaming:last_90_days
Status:         ✓ Approved (permanent)
Result:         12 transactions, total $164.87
Location:       This device only
```

### 7. First-run Agent Discovery

When the user first opens the Agents section with no agents registered:

```
┌─────────────────────────────────────────────────┐
│                                                 │
│   🤖  Your Agent Gateway                       │
│                                                 │
│   PrivacyFinance protects your data             │
│   from AI agents — on your device.            │
│                                                 │
│   No agents yet.                               │
│                                                 │
│   Want to try one?                             │
│                                                 │
│   [Install Netflix Analyzer]  ← one-click      │
│   [Browse Agent Directory]     ← curated list   │
│   [Add Agent Manually]         ← paste URL     │
│                                                 │
└─────────────────────────────────────────────────┘
```

Curated agent directory: pre-approved agents with verified manifests. One-click install. This makes the first experience effortless.

### 8. Scope Language — Plain English Only

Every scope is presented in user-understandable language. Internal scope names never shown to non-technical users.

| Internal Scope | What User Sees |
|---------------|----------------|
| `transactions:read:category:X` | Your [category name] transactions |
| `transactions:read:date:last_30_days` | Transactions from the last 30 days |
| `transactions:read:amount:under_500` | Transactions under $500 |
| `income:read` | Your income transactions |
| `accounts:read:balances` | Your account balances |
| `budget:read` | Your budget allocations |

Scope explanations should answer: *"What would this agent actually learn about me?"*

---

## Technical Implementation Notes

### Stack
- **Rust/Tauri backend** — new commands for agent registry, key generation, scope validation, audit logging
- **React frontend** — new AgentSettings view, ShieldIcon component, ApprovalModal, AuditLog view
- **SQLite** — new tables: `agents`, `agent_keys`, `agent_scope_grants`, `agent_audit_log`
- **Local HTTP server** — runs on localhost port 3847, agents call this to query data
- **Inference** — Tier 1 template matching (default), Tier 2 Phi-3.5-mini via Ollama (optional install)

### Key Files to Create/Modify

```
src-tauri/src/
  models/
    agent.rs          — Agent, AgentKey, ScopeGrant, AuditEntry structs
    scope.rs          — Scope enum + parsing + human-readable translation
  commands/
    agents.rs          — CRUD for agent registry
    agent_keys.rs      — key generation, validation, revocation
    agent_scopes.rs    — scope grant management
    agent_audit.rs     — audit log write + read
    agent_gateway.rs   — local HTTP server lifecycle
  db/
    migrations.rs     — add agents, agent_keys, agent_scope_grants, agent_audit_log tables

src/
  components/
    agents/
      AgentDirectory.tsx
      AgentCard.tsx
      AgentApprovalModal.tsx
      PermissionCustomizer.tsx
      AuditLog.tsx
      AuditEntryRow.tsx
    layout/
      ShieldIcon.tsx        — header icon with badge
      AgentActivityDropdown.tsx
  views/
    AgentSettings.tsx        — Settings → Agents
  hooks/
    useAgentGateway.ts      — frontend state management for agents
    useScopeTranslation.ts   — internal scope → plain English
  api/
    agents.ts               — Tauri invoke wrappers

scripts/
  start-agent-gateway.ts    — starts local HTTP server on port 3847
```

### Database Schema (reference)

```sql
CREATE TABLE agents (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  publisher TEXT NOT NULL,
  version TEXT NOT NULL,
  description TEXT,
  manifest_url TEXT,
  icon_url TEXT,
  status TEXT NOT NULL DEFAULT 'pending'
    CHECK(status IN ('pending','approved','denied','revoked')),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE agent_keys (
  id TEXT PRIMARY KEY,
  agent_id TEXT NOT NULL,
  key_hash TEXT NOT NULL UNIQUE,
  scopes TEXT NOT NULL,           -- JSON array of granted scopes
  expires_at TEXT,
  last_used_at TEXT,
  created_at TEXT NOT NULL,
  FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE
);

CREATE TABLE agent_scope_grants (
  id TEXT PRIMARY KEY,
  agent_id TEXT NOT NULL,
  scope TEXT NOT NULL,
  access_mode TEXT NOT NULL DEFAULT 'always'
    CHECK(access_mode IN ('always','ask_each_time','never')),
  FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE,
  UNIQUE(agent_id, scope)
);

CREATE TABLE agent_audit_log (
  id TEXT PRIMARY KEY,
  agent_id TEXT NOT NULL,
  scope_requested TEXT NOT NULL,
  scope_granted TEXT NOT NULL,
  status TEXT NOT NULL,
  result_summary TEXT,
  created_at TEXT NOT NULL,
  FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE SET NULL
);
```

---

## What to Produce

Given this brief, produce a **detailed implementation plan** with:

1. **Phased rollout plan** — what's shippable first, what builds on it
2. **Task breakdown per phase** — concrete, small, completable tasks
3. **UI wireframes or screen specs** — describe each screen in enough detail to build
4. **Component inventory** — all new components, their states, and props
5. **API design** — Tauri commands, their inputs/outputs
6. **Data model decisions** — anything not specified above
7. **Testing strategy** — what to test at each phase
8. **Open questions flagged** — anything ambiguous in this brief that needs a decision before building

Keep it focused on Phase G1 (the gateway core) first. Don't spec out G2/G3/G4 in detail — just outline them.

---

## Constraints

- Must work on macOS (primary target for beta)
- Must not break existing PrivacyFinance functionality
- All inference is local-only — no cloud component, no exceptions
- The app must feel fast — agent queries should return in <1 second for template matches
- Permission UX must be testable with non-technical users before shipping

---

## What "intuitive" means here

- A user who's never thought about AI agent permissions should be able to register an agent and approve its first request in under 60 seconds
- The permission system should feel like a natural extension of PrivacyFinance's existing clean UX — not a security product bolted on
- Every screen should answer the user's question before they ask it: *"What is this?", "What does it want?", "Is it safe?", "How do I stop it?"*
