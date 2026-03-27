import { useState } from 'react';
import { Agent } from '../../types';
import { agentsApi } from '../../api/agents';
import { translateScope } from '../../hooks/useScopeTranslation';
import { Modal } from '../ui/Modal';
import { Button } from '../ui/Button';
import styles from './AgentApprovalModal.module.css';

interface AgentApprovalModalProps {
  agent: Agent;
  requestedScopes: string[];
  onClose: () => void;
  onDecision: () => void;
}

export function AgentApprovalModal({ agent, requestedScopes, onClose, onDecision }: AgentApprovalModalProps) {
  const [acting, setActing] = useState(false);

  async function handleApprove(permanent: boolean) {
    setActing(true);
    try {
      // Approve the agent
      await agentsApi.updateStatus(agent.id, 'approved');

      // Set scope grants
      for (const scope of requestedScopes) {
        await agentsApi.setScope(agent.id, scope, permanent ? 'always' : 'ask_each_time');
      }

      // Generate an API key with the approved scopes
      await agentsApi.generateKey(agent.id, requestedScopes);

      onDecision();
    } catch (err) {
      console.error('Failed to approve agent:', err);
    } finally {
      setActing(false);
    }
  }

  async function handleDeny() {
    setActing(true);
    try {
      await agentsApi.updateStatus(agent.id, 'denied');
      onDecision();
    } catch (err) {
      console.error('Failed to deny agent:', err);
    } finally {
      setActing(false);
    }
  }

  return (
    <Modal title="" onClose={onClose}>
      <div className={styles.approval}>
        <div className={styles.agentIcon}>🔵</div>
        <h2 className={styles.agentName}>{agent.name}</h2>
        <div className={styles.publisher}>by {agent.publisher}</div>

        <div className={styles.section}>
          <div className={styles.sectionLabel}>📋 What it wants:</div>
          <ul className={styles.scopeList}>
            {requestedScopes.map((scope) => (
              <li key={scope} className={styles.scopeItem}>
                {translateScope(scope)}
              </li>
            ))}
          </ul>
        </div>

        <div className={styles.section}>
          <div className={styles.sectionLabel}>🔒 How it works:</div>
          <ul className={styles.infoList}>
            <li>Runs on your device</li>
            <li>Your data never leaves your Mac</li>
          </ul>
        </div>

        <div className={styles.actions}>
          <Button
            variant="ghost"
            onClick={() => handleApprove(false)}
            disabled={acting}
          >
            Allow This Once
          </Button>
          <Button
            onClick={() => handleApprove(true)}
            disabled={acting}
          >
            {acting ? 'Approving...' : 'Allow & Trust'}
          </Button>
          <Button
            variant="danger"
            onClick={handleDeny}
            disabled={acting}
          >
            Deny
          </Button>
        </div>
      </div>
    </Modal>
  );
}
