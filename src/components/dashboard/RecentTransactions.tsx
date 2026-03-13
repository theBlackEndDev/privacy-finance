import { Transaction } from '../../types';
import { formatCurrency } from '../../utils/formatCurrency';
import { formatDate } from '../../utils/formatDate';

interface Props {
  transactions: Transaction[];
}

export function RecentTransactions({ transactions }: Props) {
  if (transactions.length === 0) {
    return (
      <div style={{ color: 'var(--text-muted)', fontSize: 14, padding: '16px 0', textAlign: 'center' }}>
        No transactions this month
      </div>
    );
  }

  return (
    <div>
      {transactions.map(t => (
        <div
          key={t.id}
          style={{
            display: 'grid',
            gridTemplateColumns: '80px 1fr auto',
            gap: 12,
            alignItems: 'center',
            padding: '9px 0',
            borderBottom: '1px solid var(--border-color)',
          }}
        >
          <div style={{ fontSize: 12, color: 'var(--text-secondary)' }}>{formatDate(t.date)}</div>
          <div>
            <div style={{ fontSize: 13, color: 'var(--text-primary)', fontWeight: 500, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
              {t.payee || '—'}
            </div>
            {t.categoryName && (
              <div style={{ fontSize: 11, color: 'var(--text-muted)' }}>{t.categoryName}</div>
            )}
          </div>
          <div style={{
            fontSize: 13,
            fontFamily: 'var(--font-mono)',
            fontVariantNumeric: 'tabular-nums',
            textAlign: 'right',
            color: t.kind === 'income' ? 'var(--color-income)' : t.kind === 'transfer' ? 'var(--color-transfer)' : 'var(--color-expense)',
          }}>
            {t.kind === 'income' ? '+' : t.kind === 'transfer' ? '' : '-'}{formatCurrency(Math.abs(t.amount))}
          </div>
        </div>
      ))}
    </div>
  );
}
