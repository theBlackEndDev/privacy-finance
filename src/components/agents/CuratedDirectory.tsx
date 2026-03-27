import { useState, useEffect } from 'react';
import { CuratedAgent } from '../../types';
import { agentsApi } from '../../api/agents';
import { translateScope } from '../../hooks/useScopeTranslation';
import { Button } from '../ui/Button';
import styles from './CuratedDirectory.module.css';

interface CuratedDirectoryProps {
  onInstalled: () => void;
  installedIds: string[];
}

export function CuratedDirectory({ onInstalled, installedIds }: CuratedDirectoryProps) {
  const [agents, setAgents] = useState<CuratedAgent[]>([]);
  const [installing, setInstalling] = useState<string | null>(null);

  useEffect(() => {
    agentsApi.getCuratedAgents().then(setAgents);
  }, []);

  async function handleInstall(id: string) {
    setInstalling(id);
    try {
      await agentsApi.installCurated(id);
      onInstalled();
    } catch (err) {
      console.error('Failed to install curated agent:', err);
    } finally {
      setInstalling(null);
    }
  }

  if (agents.length === 0) return null;

  return (
    <div className={styles.directory}>
      <h3 className={styles.title}>Agent Directory</h3>
      <p className={styles.subtitle}>Pre-verified agents ready for one-click install</p>
      <div className={styles.grid}>
        {agents.map((agent) => {
          const isInstalled = installedIds.includes(agent.id);
          return (
            <div key={agent.id} className={styles.card}>
              <div className={styles.cardHeader}>
                <span className={styles.icon}>🤖</span>
                <div>
                  <div className={styles.name}>
                    {agent.name}
                    {agent.verified && <span className={styles.badge}>✓ Verified</span>}
                  </div>
                  <div className={styles.publisher}>{agent.publisher}</div>
                </div>
              </div>
              <div className={styles.description}>{agent.description}</div>
              <div className={styles.scopes}>
                {agent.requestedScopes.map((s) => (
                  <span key={s} className={styles.scopeTag}>{translateScope(s)}</span>
                ))}
              </div>
              <div className={styles.cardAction}>
                {isInstalled ? (
                  <span className={styles.installed}>Installed</span>
                ) : (
                  <Button
                    size="sm"
                    onClick={() => handleInstall(agent.id)}
                    disabled={installing === agent.id}
                  >
                    {installing === agent.id ? 'Installing...' : 'Install'}
                  </Button>
                )}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
