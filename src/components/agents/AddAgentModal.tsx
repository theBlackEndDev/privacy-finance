import { useState } from 'react';
import { Modal } from '../ui/Modal';
import { Button } from '../ui/Button';
import { agentsApi } from '../../api/agents';
import styles from './AddAgentModal.module.css';

interface AddAgentModalProps {
  onClose: () => void;
  onCreated: () => void;
}

export function AddAgentModal({ onClose, onCreated }: AddAgentModalProps) {
  const [name, setName] = useState('');
  const [publisher, setPublisher] = useState('');
  const [description, setDescription] = useState('');
  const [saving, setSaving] = useState(false);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!name.trim() || !publisher.trim()) return;

    setSaving(true);
    try {
      await agentsApi.create({
        name: name.trim(),
        publisher: publisher.trim(),
        description: description.trim() || undefined,
      });
      onCreated();
    } catch (err) {
      console.error('Failed to create agent:', err);
    } finally {
      setSaving(false);
    }
  }

  return (
    <Modal title="Add Agent" onClose={onClose}>
      <form onSubmit={handleSubmit} className={styles.form}>
        <div className={styles.field}>
          <label className={styles.label}>Agent Name *</label>
          <input
            className={styles.input}
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="e.g. Netflix Spend Analyzer"
            autoFocus
          />
        </div>

        <div className={styles.field}>
          <label className={styles.label}>Publisher *</label>
          <input
            className={styles.input}
            value={publisher}
            onChange={(e) => setPublisher(e.target.value)}
            placeholder="e.g. PrivacyFinance Labs"
          />
        </div>

        <div className={styles.field}>
          <label className={styles.label}>Description</label>
          <textarea
            className={styles.textarea}
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            placeholder="What does this agent do?"
            rows={3}
          />
        </div>

        <div className={styles.actions}>
          <Button variant="ghost" type="button" onClick={onClose}>Cancel</Button>
          <Button type="submit" disabled={!name.trim() || !publisher.trim() || saving}>
            {saving ? 'Creating...' : 'Add Agent'}
          </Button>
        </div>
      </form>
    </Modal>
  );
}
