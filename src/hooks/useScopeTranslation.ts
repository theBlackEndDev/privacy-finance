const SCOPE_TRANSLATIONS: Record<string, string> = {
  'transactions:read': 'All your transactions',
  'income:read': 'Your income transactions',
  'accounts:read:balances': 'Your account balances',
  'budget:read': 'Your budget allocations',
  'insights:generate': 'Generate insights from your data',
};

export function translateScope(scope: string): string {
  // Check exact match first
  if (SCOPE_TRANSLATIONS[scope]) return SCOPE_TRANSLATIONS[scope];

  // Parse parameterized scopes
  const parts = scope.split(':');

  if (parts[0] === 'transactions' && parts[1] === 'read') {
    if (parts[2] === 'category' && parts[3]) {
      return `Your ${parts[3]} transactions`;
    }
    if (parts[2] === 'date' && parts[3]) {
      const match = parts[3].match(/last_(\d+)_days/);
      if (match) return `Transactions from the last ${match[1]} days`;
    }
    if (parts[2] === 'amount') {
      if (parts[3]?.startsWith('under_')) return `Transactions under $${parts[3].replace('under_', '')}`;
      if (parts[3]?.startsWith('over_')) return `Transactions over $${parts[3].replace('over_', '')}`;
    }
  }

  if (parts[0] === 'budget' && parts[1] === 'read' && parts[2] === 'category' && parts[3]) {
    return `Your ${parts[3]} budget`;
  }

  return scope;
}

export function translateScopes(scopes: string[]): string[] {
  return scopes.map(translateScope);
}
