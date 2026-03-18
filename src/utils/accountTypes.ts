import type { Account } from '../types';

export const ACCOUNT_TYPE_LABELS: Record<Account['type'], string> = {
  checking: 'Checking',
  savings: 'Savings',
  credit_card: 'Credit Card',
  cash: 'Cash',
  investment: 'Investment',
  other: 'Other',
};

export const ACCOUNT_TYPE_OPTIONS = (
  Object.entries(ACCOUNT_TYPE_LABELS) as [Account['type'], string][]
).map(([value, label]) => ({ value, label }));
