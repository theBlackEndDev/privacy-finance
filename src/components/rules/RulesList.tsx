import { CategorizationRule } from '../../types';
import { cx } from '../../utils/classNames';
import styles from './RulesList.module.css';

interface RulesListProps {
  rules: CategorizationRule[];
  onEdit: (rule: CategorizationRule) => void;
  onDelete: (id: string) => void;
  onToggleActive: (rule: CategorizationRule) => void;
  onMove: (id: string, direction: 'up' | 'down') => void;
  onTest: (rule: CategorizationRule) => void;
}

const MATCH_TYPE_LABELS: Record<string, string> = {
  contains: 'Contains',
  starts_with: 'Starts with',
  ends_with: 'Ends with',
  exact: 'Exact',
  regex: 'Regex',
};

const FIELD_LABELS: Record<string, string> = {
  payee: 'Payee',
  description: 'Description',
  amount: 'Amount',
};

export function RulesList({ rules, onEdit, onDelete, onToggleActive, onMove, onTest }: RulesListProps) {
  return (
    <div className={styles.container}>
      <table className={styles.table}>
        <thead>
          <tr>
            <th style={{ width: '3rem' }}>#</th>
            <th>Name</th>
            <th>Match</th>
            <th>Pattern</th>
            <th>Applied</th>
            <th>Status</th>
            <th>Actions</th>
          </tr>
        </thead>
        <tbody>
          {rules.map((rule, idx) => (
            <tr key={rule.id}>
              <td>
                <div className={styles.orderBtns}>
                  <button
                    className={styles.orderBtn}
                    disabled={idx === 0}
                    onClick={() => onMove(rule.id, 'up')}
                    title="Move up (higher priority)"
                  >
                    ▲
                  </button>
                  <button
                    className={styles.orderBtn}
                    disabled={idx === rules.length - 1}
                    onClick={() => onMove(rule.id, 'down')}
                    title="Move down (lower priority)"
                  >
                    ▼
                  </button>
                </div>
              </td>
              <td>{rule.name}</td>
              <td>
                <div className={styles.matchInfo}>
                  <span>{FIELD_LABELS[rule.matchField] ?? rule.matchField}</span>
                  <span className={styles.matchType}>
                    {MATCH_TYPE_LABELS[rule.matchType] ?? rule.matchType}
                  </span>
                </div>
              </td>
              <td>
                <span className={styles.pattern}>{rule.pattern}</span>
              </td>
              <td>
                <span className={styles.applyCount}>{rule.applyCount}</span>
              </td>
              <td>
                <span
                  className={cx(styles.badge, rule.isActive ? styles.badgeActive : styles.badgeInactive)}
                  style={{ cursor: 'pointer' }}
                  onClick={() => onToggleActive(rule)}
                  title={rule.isActive ? 'Click to deactivate' : 'Click to activate'}
                >
                  {rule.isActive ? 'Active' : 'Inactive'}
                </span>
              </td>
              <td>
                <div className={styles.actions}>
                  <button className={styles.actionBtn} onClick={() => onTest(rule)} title="Test rule">
                    🔍
                  </button>
                  <button className={styles.actionBtn} onClick={() => onEdit(rule)} title="Edit">
                    ✎
                  </button>
                  <button className={styles.actionBtn} onClick={() => onDelete(rule.id)} title="Delete">
                    ✕
                  </button>
                </div>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
