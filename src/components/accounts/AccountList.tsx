import { Account } from '../../types';
import { Button } from '../ui/Button';
import { EmptyState } from '../ui/EmptyState';
import { formatCurrency } from '../../utils/formatCurrency';
import { cx } from '../../utils/classNames';
import styles from './AccountList.module.css';

interface AccountListProps {
  accounts: Account[];
  onAdd: () => void;
  onEdit: (account: Account) => void;
  onDelete: (account: Account) => void;
}

const TYPE_LABELS: Record<string, string> = {
  checking: 'Checking',
  savings: 'Savings',
  credit_card: 'Credit Card',
  cash: 'Cash',
  investment: 'Investment',
  other: 'Other',
};

export function AccountList({ accounts, onAdd, onEdit, onDelete }: AccountListProps) {
  const active = accounts.filter((a) => !a.isArchived);
  const archived = accounts.filter((a) => a.isArchived);

  return (
    <div>
      <div className={styles.header}>
        <span className={styles.title}>Accounts</span>
        <Button onClick={onAdd} size="sm">+ Add Account</Button>
      </div>

      {accounts.length === 0 ? (
        <EmptyState
          title="No accounts yet"
          description="Add your first account to start tracking finances."
          action={<Button onClick={onAdd}>Add Account</Button>}
        />
      ) : (
        <>
          <div className={styles.list}>
            {active.map((account) => (
              <AccountRow key={account.id} account={account} onEdit={onEdit} onDelete={onDelete} />
            ))}
          </div>
          {archived.length > 0 && (
            <>
              <p style={{ margin: '16px 0 8px', fontSize: 12, color: 'var(--text-muted)', textTransform: 'uppercase', letterSpacing: '0.05em' }}>Archived</p>
              <div className={styles.list}>
                {archived.map((account) => (
                  <AccountRow key={account.id} account={account} onEdit={onEdit} onDelete={onDelete} />
                ))}
              </div>
            </>
          )}
        </>
      )}
    </div>
  );
}

function AccountRow({ account, onEdit, onDelete }: { account: Account; onEdit: (a: Account) => void; onDelete: (a: Account) => void }) {
  return (
    <div className={cx(styles.row, account.isArchived && styles.archived)}>
      <div>
        <div className={styles.name}>{account.name}</div>
        <div className={styles.type}>{TYPE_LABELS[account.type] ?? account.type}</div>
      </div>
      <div className={styles.balance}>{formatCurrency(account.openingBalance)}</div>
      <div className={styles.actions}>
        <Button variant="ghost" size="sm" onClick={() => onEdit(account)}>Edit</Button>
        <Button variant="danger" size="sm" onClick={() => onDelete(account)}>Delete</Button>
      </div>
    </div>
  );
}
