import { describe, it, expect, afterEach } from 'vitest';
import { formatDate, setDateFormat } from '../formatDate';

describe('formatDate', () => {
  afterEach(() => setDateFormat('MMM D, YYYY'));

  it('defaults to MMM D, YYYY', () => {
    expect(formatDate('2024-01-05')).toBe('Jan 5, 2024');
  });

  it('configured format', () => {
    const cases: Array<[Parameters<typeof setDateFormat>[0], string]> = [
      ['MMM D', 'Jan 5'],
      ['MMM D, YYYY', 'Jan 5, 2024'],
      ['MM/DD/YYYY', '01/05/2024'],
      ['DD/MM/YYYY', '05/01/2024'],
      ['YYYY-MM-DD', '2024-01-05'],
    ];
    for (const [format, expected] of cases) {
      setDateFormat(format);
      expect(formatDate('2024-01-05')).toBe(expected);
    }
  });
});
