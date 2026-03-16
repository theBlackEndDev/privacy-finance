export const TICK_STYLE = { fill: 'var(--text-muted)', fontSize: 11 } as const;

export const TOOLTIP_STYLE = {
  contentStyle: { background: 'var(--bg-card)', border: '1px solid var(--border-color)', borderRadius: 6 },
  labelStyle: { color: 'var(--text-primary)' },
  itemStyle: { color: 'var(--text-secondary)' },
} as const;

export function dollarTickFormatter(v: number): string {
  return `$${(v / 100).toFixed(0)}`;
}
