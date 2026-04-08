import { describe, it, expect } from 'vitest';
import { formatCurrency } from '../formatCurrency';

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
});
