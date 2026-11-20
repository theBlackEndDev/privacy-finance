import { describe, it, expect } from 'vitest';
import { isQuestion } from '../isQuestion';

describe('isQuestion', () => {
  it('returns true for strings containing a question mark', () => {
    expect(isQuestion('how much did I spend?')).toBe(true);
  });

  it('returns true for question words at start', () => {
    expect(isQuestion('what is my balance')).toBe(true);
    expect(isQuestion('where did I spend the most')).toBe(true);
    expect(isQuestion('show my expenses')).toBe(true);
    expect(isQuestion('compare income vs expenses')).toBe(true);
    expect(isQuestion('find groceries')).toBe(true);
  });

  it('returns true for auxiliary verbs at start', () => {
    expect(isQuestion('do I have enough')).toBe(true);
    expect(isQuestion('can I afford this')).toBe(true);
    expect(isQuestion('is my balance positive')).toBe(true);
  });

  it('returns false for statements', () => {
    expect(isQuestion('groceries')).toBe(false);
    expect(isQuestion('rent payment')).toBe(false);
  });

  it('returns false for empty or whitespace-only input', () => {
    expect(isQuestion('')).toBe(false);
    expect(isQuestion('   ')).toBe(false);
  });

  it('is case-insensitive', () => {
    expect(isQuestion('How much')).toBe(true);
    expect(isQuestion('WHAT is this')).toBe(true);
  });
});
