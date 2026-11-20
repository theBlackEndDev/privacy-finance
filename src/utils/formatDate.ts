/**
 * Date display formats supported by `formatDate` (Locked decision D13).
 */
export type DateFormat = 'MMM D' | 'MMM D, YYYY' | 'MM/DD/YYYY' | 'DD/MM/YYYY' | 'YYYY-MM-DD';

const MONTH_NAMES = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];

let dateFormat: DateFormat = 'MMM D, YYYY';

/**
 * Set the module-level format used by `formatDate`.
 */
export function setDateFormat(format: DateFormat): void {
  dateFormat = format;
}

/**
 * Format an ISO date string (YYYY-MM-DD) using the configured format.
 * Default: "2026-03-13" → "Mar 13, 2026"
 */
export function formatDate(isoDate: string): string {
  const [year, month, day] = isoDate.split('-').map(Number);
  const mm = String(month).padStart(2, '0');
  const dd = String(day).padStart(2, '0');
  switch (dateFormat) {
    case 'MMM D':
      return `${MONTH_NAMES[month - 1]} ${day}`;
    case 'MMM D, YYYY':
      return `${MONTH_NAMES[month - 1]} ${day}, ${year}`;
    case 'MM/DD/YYYY':
      return `${mm}/${dd}/${year}`;
    case 'DD/MM/YYYY':
      return `${dd}/${mm}/${year}`;
    case 'YYYY-MM-DD':
      return `${year}-${mm}-${dd}`;
  }
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
