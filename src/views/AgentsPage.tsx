import { useState, useEffect } from 'react';
import { Agent } from '../types';
import { agentsApi } from '../api/agents';
import { AgentCard } from '../components/agents/AgentCard';
import { AddAgentModal } from '../components/agents/AddAgentModal';
import { AgentDetailModal } from '../components/agents/AgentDetailModal';
import { EmptyState } from '../components/ui/EmptyState';
import { Button } from '../components/ui/Button';
import styles from './AgentsPage.module.css';

export function AgentsPage() {
  const [agents, setAgents] = useState<Agent[]>([]);
  const [loading, setLoading] = useState(true);
  const [showAdd, setShowAdd] = useState(false);
  const [selectedAgent, setSelectedAgent] = useState<Agent | null>(null);

  async function loadAgents() {
    setLoading(true);
    const data = await agentsApi.getAll();
    setAgents(data);
    setLoading(false);
  }

  useEffect(() => { loadAgents(); }, []);

  async function handleDelete(id: string) {
    await agentsApi.delete(id);
    loadAgents();
  }

  async function handleStatusChange(id: string, status: string) {
    await agentsApi.updateStatus(id, status);
    loadAgents();
  }

  if (loading) return <div className={styles.loading}>Loading agents...</div>;

  return (
    <div className={styles.page}>
      <div className={styles.header}>
        <h2 className={styles.title}>Agent Gateway</h2>
        <Button onClick={() => setShowAdd(true)}>+ Add Agent</Button>
      </div>

      {agents.length === 0 ? (
        <EmptyState
          title="No agents yet"
          description="PrivacyFinance protects your financial data from AI agents — on your device. Add an agent to get started."
          action={<Button onClick={() => setShowAdd(true)}>Add Agent Manually</Button>}
        />
      ) : (
        <div className={styles.grid}>
          {agents.map((agent) => (
            <AgentCard
              key={agent.id}
              agent={agent}
              onManage={() => setSelectedAgent(agent)}
              onRevoke={() => handleStatusChange(agent.id, 'revoked')}
              onApprove={() => handleStatusChange(agent.id, 'approved')}
              onDelete={() => handleDelete(agent.id)}
            />
          ))}
        </div>
      )}

      {showAdd && (
        <AddAgentModal
          onClose={() => setShowAdd(false)}
          onCreated={() => { setShowAdd(false); loadAgents(); }}
        />
      )}

      {selectedAgent && (
        <AgentDetailModal
          agent={selectedAgent}
          onClose={() => { setSelectedAgent(null); loadAgents(); }}
        />
      )}
    </div>
  );
}
