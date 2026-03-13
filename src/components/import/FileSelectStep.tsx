import { useState } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import { Account } from '../../types';
import { Button } from '../ui/Button';
import { isTauri } from '../../api/invoke';

interface Props {
  accounts: Account[];
  onNext: (filePath: string, accountId: string) => void;
}

export function FileSelectStep({ accounts, onNext }: Props) {
  const [filePath, setFilePath] = useState('');
  const [accountId, setAccountId] = useState(accounts[0]?.id ?? '');
  const [error, setError] = useState('');

  async function pickFile() {
    if (!isTauri()) {
      setFilePath('/example/bank-export.csv');
      return;
    }
    const selected = await open({
      filters: [{ name: 'CSV', extensions: ['csv', 'tsv', 'txt'] }],
      multiple: false,
    });
    if (typeof selected === 'string') setFilePath(selected);
  }

  function handleNext() {
    if (!filePath) { setError('Please select a file.'); return; }
    if (!accountId) { setError('Please select an account.'); return; }
    setError('');
    onNext(filePath, accountId);
  }

  const filename = filePath ? filePath.split(/[\\/]/).pop() : '';

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 24 }}>
      <div>
        <div style={{ fontSize: 13, color: 'var(--text-secondary)', marginBottom: 8, fontWeight: 500 }}>
          CSV File
        </div>
        <div style={{ display: 'flex', gap: 10, alignItems: 'center' }}>
          <Button variant="ghost" onClick={pickFile}>Browse…</Button>
          {filename && (
            <span style={{ fontSize: 13, color: 'var(--text-primary)', fontFamily: 'var(--font-mono)' }}>
              {filename}
            </span>
          )}
          {!filename && (
            <span style={{ fontSize: 13, color: 'var(--text-muted)' }}>No file selected</span>
          )}
        </div>
      </div>

      <div>
        <div style={{ fontSize: 13, color: 'var(--text-secondary)', marginBottom: 8, fontWeight: 500 }}>
          Import into Account
        </div>
        <select
          value={accountId}
          onChange={e => setAccountId(e.target.value)}
          style={{
            background: 'var(--bg-input)',
            border: '1px solid var(--border-color)',
            borderRadius: 6,
            color: 'var(--text-primary)',
            padding: '8px 12px',
            fontSize: 13,
            minWidth: 220,
          }}
        >
          {accounts.map(a => (
            <option key={a.id} value={a.id}>{a.name}</option>
          ))}
        </select>
      </div>

      {error && <div style={{ fontSize: 13, color: 'var(--color-expense)' }}>{error}</div>}

      <div style={{ display: 'flex', justifyContent: 'flex-end' }}>
        <Button variant="primary" onClick={handleNext}>Preview →</Button>
      </div>
    </div>
  );
}
