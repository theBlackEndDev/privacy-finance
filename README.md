# PrivacyFinance

A modern, privacy-first personal finance tracker for desktop. All data stays local — no accounts, no cloud, no subscriptions.

Built with Tauri 2.x, React 19, TypeScript, and SQLite.

---

## Features

- **100% local** — your financial data never leaves your machine
- **Accounts** — checking, savings, credit card, cash, investment
- **Transactions** — income, expenses, and transfers between accounts
- **Categories** — color-coded with 15 defaults, fully customizable
- **Dashboard** — monthly summary cards, spending pie chart, trend chart, account balances
- **CSV Import** — 5-step wizard with column mapping, deduplication, and one-click rollback
- **Keyboard shortcuts** — Ctrl+1-5 navigation, Ctrl+N new transaction, Ctrl+F search
- **Dark theme** — high-contrast dark UI with tabular number formatting
- **Import profiles** — save and reuse CSV column mappings per bank

---

## Tech Stack

| Layer | Technology |
|-------|------------|
| Desktop shell | Tauri 2.x |
| Frontend | React 19 + TypeScript + Vite 7 |
| Backend | Rust |
| Database | SQLite via rusqlite (bundled, no install needed) |
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

```bash
# Clone the repo
git clone <repo-url>
cd privacy-finance

# Install frontend dependencies
npm install

# Start the app (compiles Rust + launches desktop window)
npm run tauri dev
```

First run takes 2–5 minutes to compile the Rust backend. Subsequent runs are faster.

> **Note:** This opens a native desktop window — it is not a web app. Accessing the Vite dev server URL directly in a browser will not work (Tauri `invoke()` calls require the desktop runtime).

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

6 tables: `accounts`, `categories`, `transactions`, `import_batches`, `import_profiles`, `settings`.

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

To reset to seed data, close the app and delete the database file, then reopen.

---

## Running Tests

```bash
cd src-tauri
cargo test
```

42 tests covering migrations, account/category/transaction CRUD, transfers, dashboard aggregation, CSV import, and edge cases.

---

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+1` | Dashboard |
| `Ctrl+2` | Accounts |
| `Ctrl+3` | Transactions |
| `Ctrl+4` | Categories |
| `Ctrl+5` | Import |
| `Ctrl+N` | Focus transaction entry |
| `Ctrl+F` | Focus search |
| `Escape` | Close modal / clear search |

---

## Post-MVP Roadmap

**Dashboard**
- Spending pie chart slices are clickable — navigates to Transactions filtered by that category and the selected month

**Import**
- Fuzzy duplicate detection — flag rows where amount + payee match an existing transaction within ±3 days and let the user decide before committing
- FTS5 full-text search on payee/description (current LIKE query doesn't scale past ~50k rows)

**Transactions**
- Recurring transactions (scheduled repeating entries)
- Full undo/redo across sessions (currently only import batch rollback is supported)
- Split transactions (one transaction split across multiple categories)

**Budgets**
- Monthly budget / envelope system per category
- Budget vs. actual variance on the dashboard

**Data**
- Data export — CSV, JSON, PDF statement
- Backup/restore UI (manual DB file copy with guided dialog)
- Multi-currency support with exchange rates

**Other**
- Complex ARIA combobox dropdowns (currently using native `<select>`)
- Mobile / responsive layout
