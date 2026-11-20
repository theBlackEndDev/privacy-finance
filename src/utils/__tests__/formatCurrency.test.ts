import { describe, it, expect } from 'vitest';
import { formatCurrency, setCurrencySymbol, getCurrencySymbol } from '../formatCurrency';

describe('formatCurrency', () => {
  it('formats positive cents', () => {
    expect(formatCurrency(4299)).toBe('$42.99');
  });

  it('formats negative cents', () => {
    expect(formatCurrency(-4299)).toBe('-$42.99');
  });

  it('formats zero', () => {
    expect(formatCurrency(0)).toBe('$0.00');
  });

  it('formats large amounts with comma separators', () => {
    expect(formatCurrency(1234567)).toBe('$12,345.67');
  });

  it('formats small amounts under a dollar', () => {
    expect(formatCurrency(5)).toBe('$0.05');
  });

  it('configured symbol', () => {
    const original = getCurrencySymbol();
    try {
      setCurrencySymbol('€');
      expect(getCurrencySymbol()).toBe('€');
      expect(formatCurrency(4299)).toBe('€42.99');
      expect(formatCurrency(-4299)).toBe('-€42.99');

      setCurrencySymbol('R$');
      expect(formatCurrency(4299)).toBe('R$42.99');
    } finally {
      setCurrencySymbol(original);
    }
  });
});
