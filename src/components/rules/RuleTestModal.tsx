import { useEffect, useState } from 'react';
import { CategorizationRule, RuleTestResult } from '../../types';
import { rulesApi } from '../../api/rules';
import { formatCurrency } from '../../utils/formatCurrency';
import { Modal } from '../ui/Modal';
import { Spinner } from '../ui/Spinner';
import styles from './RuleTestModal.module.css';

interface RuleTestModalProps {
  rule: CategorizationRule;
  onClose: () => void;
}

export function RuleTestModal({ rule, onClose }: RuleTestModalProps) {
  const [results, setResults] = useState<RuleTestResult[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');

  useEffect(() => {
    async function runTest() {
      setLoading(true);
      setError('');
      try {
        const data = await rulesApi.testRule(rule.matchField, rule.matchType, rule.pattern);
        setResults(data);
      } catch (err) {
        setError(`Test failed: ${err}`);
      } finally {
        setLoading(false);
      }
    }
    runTest();
  }, [rule.matchField, rule.matchType, rule.pattern]);

  return (
    <Modal title={`Test Rule: ${rule.name}`} onClose={onClose}>
      <div className={styles.container}>
        {loading ? (
          <Spinner />
        ) : error ? (
          <div className={styles.empty}>{error}</div>
        ) : results.length === 0 ? (
          <div className={styles.empty}>No matching transactions found</div>
        ) : (
          <>
            <div className={styles.summary}>
              Found <span className={styles.count}>{results.length}</span> matching transaction{results.length !== 1 ? 's' : ''} (showing up to 20)
            </div>
            <table className={styles.table}>
              <thead>
                <tr>
                  <th>Date</th>
                  <th>Payee</th>
                  <th>Description</th>
                  <th>Amount</th>
                  <th>Matched Value</th>
                </tr>
              </thead>
              <tbody>
                {results.map((r) => (
                  <tr key={r.transactionId}>
                    <td>{r.date}</td>
                    <td>{r.payee}</td>
                    <td>{r.description}</td>
                    <td className={styles.amount}>{formatCurrency(r.amount)}</td>
                    <td><span className={styles.matchHighlight}>{r.matchedFieldValue}</span></td>
                  </tr>
                ))}
              </tbody>
            </table>
          </>
        )}
      </div>
    </Modal>
  );
}
