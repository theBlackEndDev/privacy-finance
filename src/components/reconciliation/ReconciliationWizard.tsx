import { useState } from 'react';
import { ReconciliationSummary } from '../../types';
import { reconciliationApi } from '../../api/reconciliation';
import { formatCurrency, parseCurrency } from '../../utils/formatCurrency';
import { todayISO } from '../../utils/formatDate';
import { ReconciliationRow } from './ReconciliationRow';
import { Button } from '../ui/Button';
import { cx } from '../../utils/classNames';
import styles from './ReconciliationWizard.module.css';

interface Props {
  accountId: string;
  accountName: string;
  onClose: () => void;
}

export function ReconciliationWizard({ accountId, accountName, onClose }: Props) {
  const [step, setStep] = useState<1 | 2 | 3>(1);
  const [summary, setSummary] = useState<ReconciliationSummary | null>(null);
  const [statementDate, setStatementDate] = useState(todayISO());
  const [statementBalance, setStatementBalance] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  async function handleStart() {
    const balanceCents = parseCurrency(statementBalance);
    if (!statementDate) {
      setError('Please enter a statement date.');
      return;
    }
    if (statementBalance.trim() === '') {
      setError('Please enter a statement balance.');
      return;
    }

    setError('');
    setLoading(true);
    try {
      const result = await reconciliationApi.start({
        accountId,
        statementDate,
        statementBalance: balanceCents,
      });
      setSummary(result);
      setStep(2);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }

  async function handleToggle(transactionId: string) {
    if (!summary) return;
    try {
      const result = await reconciliationApi.toggleItem(
        summary.reconciliation.id,
        transactionId,
      );
      setSummary(result);
    } catch (err) {
      setError(String(err));
    }
  }

  async function handleComplete() {
    if (!summary) return;
    setLoading(true);
    try {
      await reconciliationApi.complete(summary.reconciliation.id);
      setStep(3);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }

  async function handleCancel() {
    if (summary) {
      try {
        await reconciliationApi.cancel(summary.reconciliation.id);
      } catch {
        // Best effort cancel
      }
    }
    onClose();
  }

  const reconciledCount = summary
    ? summary.items.filter((i) => i.isReconciled).length
    : 0;

  return (
    <div className={styles.wizard}>
      {/* Step indicator */}
      <div className={styles.stepIndicator}>
        {[1, 2, 3].map((s) => (
          <div
            key={s}
            className={cx(
              styles.stepDot,
              s === step && styles.stepDotActive,
              s < step && styles.stepDotDone,
            )}
          />
        ))}
      </div>

      {error && <div className={styles.error}>{error}</div>}

      {/* Step 1: Setup */}
      {step === 1 && (
        <>
          <div className={styles.formRow}>
            <div className={styles.fieldGroup}>
              <div className={styles.label}>Statement Date</div>
              <input
                type="date"
                className={styles.input}
                value={statementDate}
                onChange={(e) => setStatementDate(e.target.value)}
                data-testid="recon-date-input"
              />
            </div>
            <div className={styles.fieldGroup}>
              <div className={styles.label}>Statement Balance ($)</div>
              <input
                type="text"
                className={styles.input}
                value={statementBalance}
                onChange={(e) => setStatementBalance(e.target.value)}
                placeholder="0.00"
                data-testid="recon-balance-input"
              />
            </div>
          </div>
          <div className={styles.actions}>
            <Button variant="ghost" onClick={onClose}>Cancel</Button>
            <Button variant="primary" onClick={handleStart} disabled={loading} data-testid="recon-start-btn">
              {loading ? 'Starting...' : 'Start Reconciliation'}
            </Button>
          </div>
        </>
      )}

      {/* Step 2: Reconcile */}
      {step === 2 && summary && (
        <>
          <div className={styles.diffBar}>
            <div>
              <div className={styles.diffLabel}>Difference</div>
              <div
                className={cx(
                  styles.diffValue,
                  summary.difference === 0 ? styles.diffZero : styles.diffNonZero,
                )}
              >
                {formatCurrency(summary.difference)}
              </div>
            </div>
            <div className={styles.balanceSummary}>
              <div className={styles.balanceItem}>
                <span className={styles.balanceLabel}>Cleared</span>
                <span className={styles.balanceValue}>
                  {formatCurrency(summary.clearedBalance)}
                </span>
              </div>
              <div className={styles.balanceItem}>
                <span className={styles.balanceLabel}>Statement</span>
                <span className={styles.balanceValue}>
                  {formatCurrency(summary.statementBalance)}
                </span>
              </div>
            </div>
          </div>

          <div className={styles.itemsList}>
            <div className={styles.listHeader}>
              <span className={styles.headerCheckbox} />
              <span className={styles.headerDate}>Date</span>
              <span className={styles.headerPayee}>Payee</span>
              <span className={styles.headerKind}>Type</span>
              <span className={styles.headerAmount}>Amount</span>
            </div>
            {summary.items.length === 0 ? (
              <div className={styles.emptyList}>
                No transactions found for this account.
              </div>
            ) : (
              summary.items.map((item) => (
                <ReconciliationRow
                  key={item.transactionId}
                  item={item}
                  onToggle={handleToggle}
                />
              ))
            )}
          </div>

          <div className={styles.actions}>
            <Button variant="ghost" onClick={handleCancel}>
              Cancel
            </Button>
            <Button
              variant="primary"
              onClick={handleComplete}
              disabled={summary.difference !== 0 || loading}
            >
              {loading ? 'Completing...' : 'Complete Reconciliation'}
            </Button>
          </div>
        </>
      )}

      {/* Step 3: Complete */}
      {step === 3 && (
        <>
          <div className={styles.successMessage}>
            <div className={styles.successTitle}>Reconciliation Complete</div>
            <div className={styles.successDetail}>
              {reconciledCount} transaction{reconciledCount !== 1 ? 's' : ''} cleared
              for {accountName}.
            </div>
          </div>
          <div className={styles.actions}>
            <Button variant="primary" onClick={onClose}>Done</Button>
          </div>
        </>
      )}
    </div>
  );
}
