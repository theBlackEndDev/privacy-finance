import { describe, it, expect } from 'vitest';
import { parseCurrency } from '../formatCurrency';

describe('parseCurrency', () => {
  it('parses simple dollar string', () => {
    expect(parseCurrency('42.99')).toBe(4299);
  });

  it('strips commas', () => {
    expect(parseCurrency('1,234.56')).toBe(123456);
  });

  it('handles negative values', () => {
    expect(parseCurrency('-10.50')).toBe(-1050);
  });

  it('returns 0 for non-numeric input', () => {
    expect(parseCurrency('abc')).toBe(0);
  });

  it('returns 0 for empty string', () => {
    expect(parseCurrency('')).toBe(0);
  });

  it('strips dollar sign', () => {
    expect(parseCurrency('$99.99')).toBe(9999);
  });
});
