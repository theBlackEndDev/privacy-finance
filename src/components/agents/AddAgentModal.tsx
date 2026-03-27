import { useState } from 'react';
import { ParsedManifest } from '../../types';
import { Modal } from '../ui/Modal';
import { Button } from '../ui/Button';
import { agentsApi } from '../../api/agents';
import { translateScope } from '../../hooks/useScopeTranslation';
import styles from './AddAgentModal.module.css';

type Mode = 'choose' | 'manual' | 'manifest';

interface AddAgentModalProps {
  onClose: () => void;
  onCreated: () => void;
}

export function AddAgentModal({ onClose, onCreated }: AddAgentModalProps) {
  const [mode, setMode] = useState<Mode>('choose');

  // Manual mode state
  const [name, setName] = useState('');
  const [publisher, setPublisher] = useState('');
  const [description, setDescription] = useState('');

  // Manifest mode state
  const [manifestInput, setManifestInput] = useState('');
  const [parsed, setParsed] = useState<ParsedManifest | null>(null);
  const [parseError, setParseError] = useState('');

  const [saving, setSaving] = useState(false);

  async function handleManualSubmit(e: React.FormEvent) {
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

  async function handleParseManifest() {
    setParseError('');
    setParsed(null);
    try {
      const result = await agentsApi.parseManifest(manifestInput.trim());
      setParsed(result);
      if (!result.valid) {
        setParseError(result.errors.join(', '));
      }
    } catch (err) {
      setParseError(String(err));
    }
  }

  async function handleManifestSubmit() {
    if (!parsed?.valid) return;
    setSaving(true);
    try {
      await agentsApi.registerFromManifest(manifestInput.trim());
      onCreated();
    } catch (err) {
      console.error('Failed to register from manifest:', err);
    } finally {
      setSaving(false);
    }
  }

  if (mode === 'choose') {
    return (
      <Modal title="Add Agent" onClose={onClose}>
        <div className={styles.chooseMode}>
          <button className={styles.modeCard} onClick={() => setMode('manifest')}>
            <span className={styles.modeIcon}>📋</span>
            <span className={styles.modeTitle}>From Manifest</span>
            <span className={styles.modeDesc}>Paste an agent manifest JSON</span>
          </button>
          <button className={styles.modeCard} onClick={() => setMode('manual')}>
            <span className={styles.modeIcon}>✏️</span>
            <span className={styles.modeTitle}>Manual Entry</span>
            <span className={styles.modeDesc}>Enter agent details by hand</span>
          </button>
        </div>
      </Modal>
    );
  }

  if (mode === 'manifest') {
    return (
      <Modal title="Add Agent from Manifest" onClose={onClose}>
        <div className={styles.form}>
          <div className={styles.field}>
            <label className={styles.label}>Agent Manifest JSON</label>
            <textarea
              className={styles.textarea}
              value={manifestInput}
              onChange={(e) => { setManifestInput(e.target.value); setParsed(null); setParseError(''); }}
              placeholder={'{\n  "name": "Netflix Spend Analyzer",\n  "publisher": "PrivacyFinance Labs",\n  "requested_scopes": [\n    "transactions:read:category:streaming"\n  ]\n}'}
              rows={8}
              autoFocus
            />
          </div>

          {parseError && <div className={styles.error}>{parseError}</div>}

          {parsed?.valid && (
            <div className={styles.preview}>
              <div className={styles.previewName}>{parsed.manifest.name}</div>
              <div className={styles.previewPublisher}>by {parsed.manifest.publisher}</div>
              {parsed.scopeDescriptions.length > 0 && (
                <div className={styles.previewScopes}>
                  <div className={styles.label}>Requested access:</div>
                  <ul className={styles.scopeList}>
                    {parsed.scopeDescriptions.map((sd) => (
                      <li key={sd.scope}>{translateScope(sd.scope)}</li>
                    ))}
                  </ul>
                </div>
              )}
            </div>
          )}

          <div className={styles.actions}>
            <Button variant="ghost" type="button" onClick={() => setMode('choose')}>Back</Button>
            {!parsed?.valid ? (
              <Button onClick={handleParseManifest} disabled={!manifestInput.trim()}>
                Parse Manifest
              </Button>
            ) : (
              <Button onClick={handleManifestSubmit} disabled={saving}>
                {saving ? 'Registering...' : 'Register Agent'}
              </Button>
            )}
          </div>
        </div>
      </Modal>
    );
  }

  // Manual mode
  return (
    <Modal title="Add Agent Manually" onClose={onClose}>
      <form onSubmit={handleManualSubmit} className={styles.form}>
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
          <Button variant="ghost" type="button" onClick={() => setMode('choose')}>Back</Button>
          <Button type="submit" disabled={!name.trim() || !publisher.trim() || saving}>
            {saving ? 'Creating...' : 'Add Agent'}
          </Button>
        </div>
      </form>
    </Modal>
  );
}
