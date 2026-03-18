import { MouseEvent } from 'react';
import { Account } from '../../types';
import { Button } from '../ui/Button';
import { EmptyState } from '../ui/EmptyState';
import { formatCurrency } from '../../utils/formatCurrency';
import { cx } from '../../utils/classNames';
import { ACCOUNT_TYPE_LABELS } from '../../utils/accountTypes';
import styles from './AccountList.module.css';

interface AccountListProps {
  accounts: Account[];
  onAdd: () => void;
  onEdit: (account: Account) => void;
  onDelete: (account: Account) => void;
  onClickAccount?: (account: Account) => void;
}

export function AccountList({ accounts, onAdd, onEdit, onDelete, onClickAccount }: AccountListProps) {
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
              <AccountRow
                key={account.id}
                account={account}
                onEdit={onEdit}
                onDelete={onDelete}
                onClick={onClickAccount}
              />
            ))}
          </div>
          {archived.length > 0 && (
            <>
              <p style={{ margin: '16px 0 8px', fontSize: 12, color: 'var(--text-muted)', textTransform: 'uppercase', letterSpacing: '0.05em' }}>Archived</p>
              <div className={styles.list}>
                {archived.map((account) => (
                  <AccountRow
                    key={account.id}
                    account={account}
                    onEdit={onEdit}
                    onDelete={onDelete}
                    onClick={onClickAccount}
                  />
                ))}
              </div>
            </>
          )}
        </>
      )}
    </div>
  );
}

interface AccountRowProps {
  account: Account;
  onEdit: (a: Account) => void;
  onDelete: (a: Account) => void;
  onClick?: (a: Account) => void;
}

function AccountRow({ account, onEdit, onDelete, onClick }: AccountRowProps) {
  function handleActionClick(e: MouseEvent, action: () => void) {
    e.stopPropagation();
    action();
  }

  return (
    <div
      className={cx(styles.row, account.isArchived && styles.archived, onClick && styles.clickable)}
      onClick={onClick ? () => onClick(account) : undefined}
      role={onClick ? 'button' : undefined}
      tabIndex={onClick ? 0 : undefined}
      onKeyDown={onClick ? (e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); onClick(account); } } : undefined}
    >
      <div>
        <div className={styles.name}>{account.name}</div>
        <div className={styles.type}>{ACCOUNT_TYPE_LABELS[account.type] ?? account.type}</div>
      </div>
      <div className={styles.balance}>{formatCurrency(account.openingBalance)}</div>
      <div className={styles.actions}>
        <Button variant="ghost" size="sm" onClick={(e) => handleActionClick(e, () => onEdit(account))}>Edit</Button>
        <Button variant="danger" size="sm" onClick={(e) => handleActionClick(e, () => onDelete(account))}>Delete</Button>
      </div>
    </div>
  );
}
