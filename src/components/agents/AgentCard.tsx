import { Agent } from '../../types';
import { Button } from '../ui/Button';
import styles from './AgentCard.module.css';

const STATUS_CONFIG: Record<string, { dot: string; label: string; color: string }> = {
  approved: { dot: '●', label: 'Active', color: 'var(--color-accent)' },
  pending:  { dot: '○', label: 'Pending', color: 'var(--color-warning, #f59e0b)' },
  revoked:  { dot: '●', label: 'Revoked', color: 'var(--color-danger, #ef4444)' },
  denied:   { dot: '●', label: 'Denied', color: 'var(--color-danger, #ef4444)' },
};

interface AgentCardProps {
  agent: Agent;
  onManage: () => void;
  onRevoke: () => void;
  onApprove: () => void;
  onDelete: () => void;
}

export function AgentCard({ agent, onManage, onRevoke, onApprove, onDelete }: AgentCardProps) {
  const status = STATUS_CONFIG[agent.status] ?? STATUS_CONFIG.pending;
  const lastActive = agent.updatedAt
    ? new Date(agent.updatedAt).toLocaleDateString()
    : 'Never';

  return (
    <div className={styles.card}>
      <div className={styles.info}>
        <div className={styles.nameRow}>
          <span className={styles.icon}>🤖</span>
          <span className={styles.name}>{agent.name}</span>
          <span className={styles.status} style={{ color: status.color }}>
            {status.dot} {status.label}
          </span>
        </div>
        <div className={styles.publisher}>{agent.publisher}</div>
        {agent.description && (
          <div className={styles.description}>{agent.description}</div>
        )}
        <div className={styles.meta}>Last active: {lastActive}</div>
      </div>
      <div className={styles.actions}>
        {agent.status === 'pending' && (
          <Button variant="primary" size="sm" onClick={onApprove}>Approve</Button>
        )}
        {agent.status === 'approved' && (
          <>
            <Button variant="ghost" size="sm" onClick={onManage}>Manage</Button>
            <Button variant="danger" size="sm" onClick={onRevoke}>Revoke</Button>
          </>
        )}
        {(agent.status === 'revoked' || agent.status === 'denied') && (
          <>
            <Button variant="ghost" size="sm" onClick={onApprove}>Re-authorize</Button>
            <Button variant="danger" size="sm" onClick={onDelete}>Remove</Button>
          </>
        )}
      </div>
    </div>
  );
}
