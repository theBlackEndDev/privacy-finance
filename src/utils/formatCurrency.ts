/**
 * Format integer cents as a currency string.
 * e.g. 4299 → "$42.99", -4299 → "-$42.99"
 */
export function formatCurrency(cents: number, _currencyCode = 'USD'): string {
  const negative = cents < 0;
  const abs = Math.abs(cents);
  const dollars = Math.floor(abs / 100);
  const centsRemainder = abs % 100;
  const formatted = `$${dollars.toLocaleString()}.${String(centsRemainder).padStart(2, '0')}`;
  return negative ? `-${formatted}` : formatted;
}

/**
 * Parse a user-entered dollar string to cents.
 * e.g. "42.99" → 4299, "1,234.56" → 123456
 */
export function parseCurrency(value: string): number {
  const cleaned = value.replace(/[^0-9.-]/g, '');
  const parsed = parseFloat(cleaned);
  if (isNaN(parsed)) return 0;
  return Math.round(parsed * 100);
}
