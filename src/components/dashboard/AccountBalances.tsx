import { AccountBalance } from '../../types';
import { formatCurrency } from '../../utils/formatCurrency';
import { ACCOUNT_TYPE_LABELS } from '../../utils/accountTypes';

interface Props {
  balances: AccountBalance[];
}

export function AccountBalances({ balances }: Props) {
  const total = balances.reduce((sum, b) => sum + b.balance, 0);

  return (
    <div>
      {balances.map(b => (
        <div
          key={b.accountId}
          style={{
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'center',
            padding: '10px 0',
            borderBottom: '1px solid var(--border-color)',
          }}
        >
          <div>
            <div style={{ fontSize: 14, color: 'var(--text-primary)', fontWeight: 500 }}>{b.accountName}</div>
            <div style={{ fontSize: 12, color: 'var(--text-muted)' }}>{ACCOUNT_TYPE_LABELS[b.accountType as keyof typeof ACCOUNT_TYPE_LABELS] ?? b.accountType}</div>
          </div>
          <div style={{
            fontSize: 14,
            fontFamily: 'var(--font-mono)',
            fontVariantNumeric: 'tabular-nums',
            color: b.balance < 0 ? 'var(--color-expense)' : 'var(--text-primary)',
          }}>
            {formatCurrency(b.balance)}
          </div>
        </div>
      ))}
      {balances.length > 1 && (
        <div style={{ display: 'flex', justifyContent: 'space-between', paddingTop: 10 }}>
          <div style={{ fontSize: 13, color: 'var(--text-secondary)', fontWeight: 600 }}>Net Worth</div>
          <div style={{
            fontSize: 14,
            fontFamily: 'var(--font-mono)',
            fontVariantNumeric: 'tabular-nums',
            fontWeight: 600,
            color: total < 0 ? 'var(--color-expense)' : 'var(--color-income)',
          }}>
            {formatCurrency(total)}
          </div>
        </div>
      )}
    </div>
  );
}
