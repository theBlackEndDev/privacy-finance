import { useState, useEffect } from 'react';
import { AuditEntry } from '../../types';
import { agentsApi } from '../../api/agents';
import { translateScope } from '../../hooks/useScopeTranslation';
import styles from './AuditLog.module.css';

interface AuditLogProps {
  agentId?: string;
}

export function AuditLog({ agentId }: AuditLogProps) {
  const [entries, setEntries] = useState<AuditEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [expanded, setExpanded] = useState<string | null>(null);

  useEffect(() => {
    setLoading(true);
    agentsApi.getAuditLog(agentId, 100).then((data) => {
      setEntries(data);
      setLoading(false);
    });
  }, [agentId]);

  if (loading) return <div className={styles.loading}>Loading audit log...</div>;

  if (entries.length === 0) {
    return <div className={styles.empty}>No activity recorded yet.</div>;
  }

  return (
    <div className={styles.log}>
      <div className={styles.header}>
        <span>Date</span>
        <span>Scope</span>
        <span>Status</span>
      </div>
      {entries.map((entry) => (
        <div key={entry.id}>
          <button
            className={styles.row}
            onClick={() => setExpanded(expanded === entry.id ? null : entry.id)}
          >
            <span className={styles.date}>
              {new Date(entry.createdAt).toLocaleString(undefined, {
                month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit',
              })}
            </span>
            <span className={styles.scope}>{translateScope(entry.scopeRequested)}</span>
            <span className={entry.status === 'approved' ? styles.approved : styles.denied}>
              {entry.status === 'approved' ? '✓' : '✗'} {entry.status}
            </span>
          </button>
          {expanded === entry.id && (
            <div className={styles.detail}>
              <div>Scope requested: <code>{entry.scopeRequested}</code></div>
              <div>Scope granted: <code>{entry.scopeGranted || 'none'}</code></div>
              {entry.resultSummary && <div>Result: {entry.resultSummary}</div>}
            </div>
          )}
        </div>
      ))}
    </div>
  );
}
