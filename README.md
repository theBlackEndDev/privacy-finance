# PrivacyFinance

A modern, privacy-first personal finance tracker for desktop. All data stays local — no accounts, no cloud, no subscriptions.

Built with Tauri 2.x, React 19, TypeScript, and SQLite.

---

## Privacy Statement

PrivacyFinance is built on three principles:

- **Privacy-first** — no telemetry, no analytics, no network calls. Your financial data never leaves your machine.
- **Offline-only** — works without an internet connection. No cloud sync, no accounts required.
- **No subscriptions** — free and open source. No premium tiers, no feature gates.

---

## Features

- **100% local** — your financial data never leaves your machine
- **Accounts** — checking, savings, credit card, cash, investment with multi-currency support
- **Transactions** — income, expenses, and transfers between accounts with split support
- **Categories** — color-coded with 15 defaults, fully customizable
- **Dashboard** — monthly summary cards, spending pie chart, trend chart, account balances
- **CSV Import** — 5-step wizard with column mapping, deduplication, and one-click rollback
- **Budgeting** — monthly envelope budgets per category with rollover support
- **Reports** — income/expense trends, category spending, cash flow, net worth, tag and payee analysis
- **Recurring Transactions** — scheduled entries with auto-posting and frequency control
- **Categorization Rules** — auto-categorize transactions by payee, description, or amount patterns
- **Tags** — flexible tagging system for cross-cutting organization
- **Split Transactions** — divide a single transaction across multiple categories
- **Reconciliation** — statement reconciliation workflow with balance verification
- **Database Encryption** — optional SQLCipher encryption at rest with password protection
- **Export & Backup** — CSV, JSON, and full data export; one-click backup and restore
- **Full-text Search** — FTS5-powered search across payees and descriptions
- **Keyboard shortcuts** — Ctrl+1-9 navigation, Ctrl+N new transaction, Ctrl+F search, Ctrl+? help
- **Accessibility** — ARIA landmarks, focus management, skip links, screen reader support
- **Dark/Light theme** — high-contrast themes with tabular number formatting
- **Import profiles** — save and reuse CSV column mappings per bank

---

## Tech Stack

| Layer | Technology |
|-------|------------|
| Desktop shell | Tauri 2.x |
| Frontend | React 19 + TypeScript + Vite 7 |
| Backend | Rust |
| Database | SQLite via rusqlite + SQLCipher (bundled, no install needed) |
| Charts | Recharts |
| CSV parsing | PapaParse (preview) + Rust csv crate (import) |
| Styling | CSS Modules + CSS custom properties |

---

## Installation

### Prerequisites by Platform

<details>
<summary><strong>macOS</strong></summary>

**1. Xcode Command Line Tools**
```bash
xcode-select --install
```

**2. Rust**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

**3. Node.js** (v18 or later)

Download from [nodejs.org](https://nodejs.org) or via Homebrew:
```bash
brew install node
```

**4. Verify**
```bash
rustc --version   # rustc 1.7x.x
node --version    # v18+
npm --version
```

No additional system libraries required on macOS.

</details>

<details>
<summary><strong>Linux (Ubuntu / Debian)</strong></summary>

**1. System dependencies**
```bash
sudo apt update
sudo apt install -y \
  build-essential \
  curl \
  wget \
  libssl-dev \
  libgtk-3-dev \
  libwebkit2gtk-4.1-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  patchelf
```

**2. Rust**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

**3. Node.js** (v18 or later)
```bash
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt install -y nodejs
```

**4. Verify**
```bash
rustc --version
node --version
npm --version
```

</details>

<details>
<summary><strong>Windows</strong></summary>

**1. WebView2 Runtime**

Already installed on Windows 11. For Windows 10, download from [Microsoft](https://developer.microsoft.com/en-us/microsoft-edge/webview2/).

**2. Visual Studio Build Tools**

Download [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) and install the **"Desktop development with C++"** workload.

**3. Rust**

Download and run [rustup-init.exe](https://rustup.rs). Accept the defaults.

**4. Node.js** (v18 or later)

Download from [nodejs.org](https://nodejs.org) and run the installer.

**5. Verify** (in a new terminal after installation)
```powershell
rustc --version
node --version
npm --version
```

</details>

---

## Running in Development

### macOS / Linux

```bash
# Clone the repo
git clone <repo-url>
cd privacy-finance

# Install frontend dependencies
npm install

# Start the app (compiles Rust + launches desktop window)
npm run tauri dev
```

### Pulling a feature branch locally

```bash
git fetch origin
git checkout feat/branding-demo-screenshots
git pull
```

### Clean rebuild (macOS / Linux)

If the app UI is not reflecting recent changes, do a clean rebuild:

```bash
rm -rf dist
npm install
npm run tauri dev
```

### Browser preview only (macOS / Linux)

Useful for quick frontend review. This does **not** replace the native Tauri app:

```bash
npm run build
npm run preview
```

First run takes 2–5 minutes to compile the Rust backend. Subsequent runs are faster.

> **Note:** `npm run tauri dev` opens the real native desktop app. `npm run preview` is only for browser/frontend review.

---

## Building for Production

```bash
npm run tauri build
```

Produces a platform-native installer in `src-tauri/target/release/bundle/`:

| Platform | Output |
|----------|--------|
| macOS | `.dmg` + `.app` |
| Linux | `.deb` + `.AppImage` |
| Windows | `.msi` + `.exe` |

---

## Project Structure

```
src/                        # React + TypeScript frontend
├── api/                    # Tauri invoke() wrappers
├── components/             # UI components by feature
│   ├── ui/                 # Shared primitives (Button, Modal, Toast, etc.)
│   ├── accounts/
│   ├── categories/
│   ├── dashboard/
│   ├── import/
│   └── transactions/
├── context/                # React Context + useReducer state
├── hooks/                  # Custom hooks
├── styles/                 # CSS custom properties + global styles
├── types/                  # TypeScript interfaces
├── utils/                  # formatCurrency, formatDate, classNames
└── views/                  # Page-level components

src-tauri/src/              # Rust backend
├── commands/               # Tauri command handlers
├── db/                     # SQLite connection, migrations, seed data
├── models/                 # Rust structs (serde Serialize/Deserialize)
├── helpers/                # Timestamp utilities
├── error.rs                # AppError enum
└── lib.rs                  # App setup + command registration
```

---

## Database

SQLite database stored at:

| Platform | Path |
|----------|------|
| macOS | `~/Library/Application Support/com.privacyfinance.app/privacyfinance.db` |
| Linux | `~/.local/share/com.privacyfinance.app/privacyfinance.db` |
| Windows | `%APPDATA%\com.privacyfinance.app\privacyfinance.db` |

### Schema

Tables: `accounts`, `categories`, `transactions`, `import_batches`, `import_profiles`, `settings`, `recurring_transactions`, `budgets`, `budget_items`, `categorization_rules`, `tags`, `transaction_tags`, `transaction_splits`, `reconciliations`, `reconciliation_items`, `exchange_rates`, `transactions_fts`.

Key design decisions:
- Amounts stored as **integer cents** (no floating point)
- All amounts stored as **positive values** — the `kind` field (`income`/`expense`/`transfer`) determines sign
- Timestamps as UTC RFC3339 strings
- WAL mode enabled for concurrent read/write
- Foreign keys enforced

### Resetting / Seed Data

On first launch, the app seeds:
- 3 accounts (Primary Checking, High-Yield Savings, Visa Rewards Card)
- 15 categories (4 income, 11 expense)
- 3 months of realistic sample transactions

To reset back to seed data:
1. fully close the app
2. delete the SQLite database file for your platform
3. reopen the app

Platform-specific commands:

#### macOS
```bash
rm -f ~/Library/Application\ Support/com.privacyfinance.app/privacyfinance.db
```

#### Linux
```bash
rm -f ~/.local/share/com.privacyfinance.app/privacyfinance.db
```

#### Windows (PowerShell)
```powershell
Remove-Item "$env:APPDATA\com.privacyfinance.app\privacyfinance.db" -Force
```

#### Windows (Command Prompt)
```cmd
del /f "%APPDATA%\com.privacyfinance.app\privacyfinance.db"
```

If the file does not exist yet, the command will simply do nothing. On next launch, the app will recreate the database and re-seed the default sample data.

### Reset app data directory

If you want to wipe both the database **and** local settings/state:

#### macOS
```bash
rm -rf ~/Library/Application\ Support/com.privacyfinance.app
```

#### Linux
```bash
rm -rf ~/.local/share/com.privacyfinance.app
```

#### Windows (PowerShell)
```powershell
Remove-Item "$env:APPDATA\com.privacyfinance.app" -Recurse -Force
```

This is more aggressive than deleting just `privacyfinance.db`.

---

## Running Tests

```bash
cd src-tauri
cargo test
```

113 tests covering migrations, account/category/transaction CRUD, transfers, dashboard aggregation, CSV import, encryption, and edge cases.

---

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+1` | Dashboard |
| `Ctrl+2` | Accounts |
| `Ctrl+3` | Transactions |
| `Ctrl+4` | Categories |
| `Ctrl+5` | Import |
| `Ctrl+6` | Recurring |
| `Ctrl+7` | Budgets |
| `Ctrl+8` | Rules |
| `Ctrl+9` | Reports |
| `Ctrl+B` | Budgets (alt) |
| `Ctrl+N` | Focus transaction entry |
| `Ctrl+F` | Focus search |
| `Ctrl+?` | Keyboard shortcut help |
| `Escape` | Close modal / clear search |

---

## License

MIT
