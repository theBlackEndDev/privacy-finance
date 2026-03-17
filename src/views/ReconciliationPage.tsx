import { useState, useEffect } from 'react';
import { Account } from '../types';
import { invoke } from '../api/invoke';
import { ReconciliationWizard } from '../components/reconciliation/ReconciliationWizard';
import styles from './ReconciliationPage.module.css';

export function ReconciliationPage() {
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [selectedAccount, setSelectedAccount] = useState<Account | null>(null);

  useEffect(() => {
    invoke<Account[]>('get_accounts', {}, []).then(setAccounts);
  }, []);

  if (selectedAccount) {
    return (
      <div className={styles.page}>
        <h2 className={styles.title}>Reconcile: {selectedAccount.name}</h2>
        <ReconciliationWizard
          accountId={selectedAccount.id}
          accountName={selectedAccount.name}
          onClose={() => setSelectedAccount(null)}
        />
      </div>
    );
  }

  return (
    <div className={styles.page}>
      <h2 className={styles.title}>Account Reconciliation</h2>
      <p className={styles.subtitle}>
        Select an account to reconcile against your bank statement.
      </p>
      <div className={styles.accountList}>
        {accounts
          .filter((a) => !a.isArchived)
          .map((account) => (
            <button
              key={account.id}
              className={styles.accountCard}
              onClick={() => setSelectedAccount(account)}
            >
              <span className={styles.accountName}>{account.name}</span>
              <span className={styles.accountType}>{account.type}</span>
            </button>
          ))}
      </div>
    </div>
  );
}
