/**
 * Format an ISO date string (YYYY-MM-DD) to display format.
 * e.g. "2026-03-13" → "Mar 13, 2026"
 */
export function formatDate(isoDate: string): string {
  const [year, month, day] = isoDate.split('-').map(Number);
  const date = new Date(year, month - 1, day);
  return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
}

/**
 * Format an ISO date string to short month/day display.
 * e.g. "2026-03-13" → "Mar 13"
 */
export function formatShortDate(isoDate: string): string {
  const [year, month, day] = isoDate.split('-').map(Number);
  const date = new Date(year, month - 1, day);
  return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
}

/**
 * Format a Date object as YYYY-MM-DD.
 */
export function formatDateISO(d: Date): string {
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, '0');
  const day = String(d.getDate()).padStart(2, '0');
  return `${y}-${m}-${day}`;
}

/**
 * Returns today's date as YYYY-MM-DD.
 */
export function todayISO(): string {
  return formatDateISO(new Date());
}
