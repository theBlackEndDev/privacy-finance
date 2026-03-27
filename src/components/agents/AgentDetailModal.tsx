import { useState, useEffect } from 'react';
import { Agent, AgentKey, AgentKeyCreated, ScopeGrant } from '../../types';
import { agentsApi } from '../../api/agents';
import { translateScope } from '../../hooks/useScopeTranslation';
import { Modal } from '../ui/Modal';
import { Button } from '../ui/Button';
import styles from './AgentDetailModal.module.css';

interface AgentDetailModalProps {
  agent: Agent;
  onClose: () => void;
}

type Tab = 'info' | 'permissions' | 'keys';

export function AgentDetailModal({ agent, onClose }: AgentDetailModalProps) {
  const [tab, setTab] = useState<Tab>('info');
  const [keys, setKeys] = useState<AgentKey[]>([]);
  const [scopes, setScopes] = useState<ScopeGrant[]>([]);
  const [newKey, setNewKey] = useState<AgentKeyCreated | null>(null);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    agentsApi.getKeys(agent.id).then(setKeys);
    agentsApi.getScopes(agent.id).then(setScopes);
  }, [agent.id]);

  async function handleGenerateKey() {
    const scopeStrings = scopes
      .filter((s) => s.accessMode === 'always')
      .map((s) => s.scope);
    if (scopeStrings.length === 0) {
      scopeStrings.push('transactions:read');
    }
    const created = await agentsApi.generateKey(agent.id, scopeStrings);
    setNewKey(created);
    agentsApi.getKeys(agent.id).then(setKeys);
  }

  async function handleRevokeKey(keyId: string) {
    await agentsApi.revokeKey(keyId);
    agentsApi.getKeys(agent.id).then(setKeys);
  }

  async function handleScopeChange(scope: string, accessMode: string) {
    await agentsApi.setScope(agent.id, scope, accessMode);
    agentsApi.getScopes(agent.id).then(setScopes);
  }

  function copyKey() {
    if (newKey) {
      navigator.clipboard.writeText(newKey.rawKey);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  }

  return (
    <Modal title={agent.name} onClose={onClose}>
      <div className={styles.tabs}>
        {(['info', 'permissions', 'keys'] as Tab[]).map((t) => (
          <button
            key={t}
            className={`${styles.tab} ${tab === t ? styles.tabActive : ''}`}
            onClick={() => setTab(t)}
          >
            {t === 'info' ? 'Info' : t === 'permissions' ? 'Permissions' : 'API Keys'}
          </button>
        ))}
      </div>

      {tab === 'info' && (
        <div className={styles.section}>
          <div className={styles.row}>
            <span className={styles.label}>Publisher</span>
            <span>{agent.publisher}</span>
          </div>
          <div className={styles.row}>
            <span className={styles.label}>Version</span>
            <span>{agent.version}</span>
          </div>
          <div className={styles.row}>
            <span className={styles.label}>Status</span>
            <span>{agent.status}</span>
          </div>
          {agent.description && (
            <div className={styles.row}>
              <span className={styles.label}>Description</span>
              <span>{agent.description}</span>
            </div>
          )}
          <div className={styles.row}>
            <span className={styles.label}>Added</span>
            <span>{new Date(agent.createdAt).toLocaleDateString()}</span>
          </div>
        </div>
      )}

      {tab === 'permissions' && (
        <div className={styles.section}>
          {scopes.length === 0 ? (
            <div className={styles.empty}>No permissions configured yet.</div>
          ) : (
            scopes.map((grant) => (
              <div key={grant.id} className={styles.scopeRow}>
                <span className={styles.scopeName}>{translateScope(grant.scope)}</span>
                <select
                  className={styles.select}
                  value={grant.accessMode}
                  onChange={(e) => handleScopeChange(grant.scope, e.target.value)}
                >
                  <option value="always">Always</option>
                  <option value="ask_each_time">Ask Each Time</option>
                  <option value="never">Never</option>
                </select>
              </div>
            ))
          )}
          <div className={styles.addScope}>
            <Button variant="ghost" size="sm" onClick={() => {
              handleScopeChange('transactions:read', 'always');
            }}>
              + Add Default Scope
            </Button>
          </div>
        </div>
      )}

      {tab === 'keys' && (
        <div className={styles.section}>
          {newKey && (
            <div className={styles.newKeyBanner}>
              <div className={styles.newKeyLabel}>New API Key (shown once only):</div>
              <code className={styles.keyValue}>{newKey.rawKey}</code>
              <Button variant="ghost" size="sm" onClick={copyKey}>
                {copied ? 'Copied!' : 'Copy'}
              </Button>
            </div>
          )}

          {keys.length === 0 && !newKey ? (
            <div className={styles.empty}>No API keys generated yet.</div>
          ) : (
            keys.map((key) => (
              <div key={key.id} className={styles.keyRow}>
                <div>
                  <code className={styles.keyMasked}>pf_live_sk_••••••••</code>
                  <div className={styles.keyMeta}>
                    Created: {new Date(key.createdAt).toLocaleDateString()}
                    {key.expiresAt && ` · Expires: ${new Date(key.expiresAt).toLocaleDateString()}`}
                    {key.lastUsedAt && ` · Last used: ${new Date(key.lastUsedAt).toLocaleDateString()}`}
                  </div>
                </div>
                <Button variant="danger" size="sm" onClick={() => handleRevokeKey(key.id)}>
                  Revoke
                </Button>
              </div>
            ))
          )}

          {agent.status === 'approved' && (
            <div className={styles.generateKey}>
              <Button onClick={handleGenerateKey}>Generate New Key</Button>
            </div>
          )}
        </div>
      )}
    </Modal>
  );
}
