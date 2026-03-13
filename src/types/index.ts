// ── Accounts ──────────────────────────────────────────────────────

export interface Account {
  id: string;
  name: string;
  type: 'checking' | 'savings' | 'credit_card' | 'cash' | 'investment' | 'other';
  currencyCode: string;
  openingBalance: number; // cents
  sortOrder: number;
  isArchived: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface CreateAccountInput {
  name: string;
  type: Account['type'];
  currencyCode?: string;
  openingBalance?: number;
  sortOrder?: number;
}

export interface UpdateAccountInput {
  name?: string;
  type?: Account['type'];
  currencyCode?: string;
  openingBalance?: number;
  sortOrder?: number;
  isArchived?: boolean;
}

// ── Categories ────────────────────────────────────────────────────

export interface Category {
  id: string;
  name: string;
  color: string;
  icon: string;
  type: 'income' | 'expense';
  sortOrder: number;
  isArchived: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface CreateCategoryInput {
  name: string;
  type: Category['type'];
  color?: string;
  icon?: string;
  sortOrder?: number;
}

export interface UpdateCategoryInput {
  name?: string;
  type?: Category['type'];
  color?: string;
  icon?: string;
  sortOrder?: number;
  isArchived?: boolean;
}

// ── Views ─────────────────────────────────────────────────────────

export type View = 'dashboard' | 'accounts' | 'transactions' | 'categories' | 'import';
