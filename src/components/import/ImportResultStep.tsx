import { ImportResult } from '../../types';
import { Button } from '../ui/Button';
import { importApi } from '../../api/import';
import { useState } from 'react';

interface Props {
  result: ImportResult;
  onViewTransactions: () => void;
  onImportAnother: () => void;
}

export function ImportResultStep({ result, onViewTransactions, onImportAnother }: Props) {
  const [undone, setUndone] = useState(false);
  const [undoing, setUndoing] = useState(false);

  async function handleUndo() {
    setUndoing(true);
    try {
      await importApi.rollbackImport(result.batchId);
      setUndone(true);
    } finally {
      setUndoing(false);
    }
  }

  if (undone) {
    return (
      <div style={{ display: 'flex', flexDirection: 'column', gap: 20, alignItems: 'center', paddingTop: 16 }}>
        <div style={{ fontSize: 32 }}>↩</div>
        <div style={{ fontSize: 16, fontWeight: 600, color: 'var(--text-primary)' }}>Import Rolled Back</div>
        <div style={{ fontSize: 13, color: 'var(--text-muted)' }}>
          {result.imported} transaction{result.imported !== 1 ? 's' : ''} removed.
        </div>
        <Button variant="primary" onClick={onImportAnother}>Import Another File</Button>
      </div>
    );
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: 12 }} data-testid="import-result-summary">
        <StatCard label="Imported" value={result.imported} color="var(--color-income)" />
        <StatCard label="Duplicates" value={result.duplicates} color="var(--text-muted)" />
        <StatCard label="Errors" value={result.errors.length} color={result.errors.length > 0 ? 'var(--color-expense)' : 'var(--text-muted)'} />
      </div>

      {result.errors.length > 0 && (
        <div style={{ border: '1px solid var(--border-color)', borderRadius: 6, overflow: 'hidden' }}>
          <div style={{ padding: '8px 14px', background: 'var(--bg-sidebar)', fontSize: 12, color: 'var(--text-secondary)', fontWeight: 600, borderBottom: '1px solid var(--border-color)' }}>
            Row Errors
          </div>
          <div style={{ maxHeight: 160, overflowY: 'auto' }}>
            {result.errors.map((e, i) => (
              <div key={i} style={{ display: 'flex', gap: 12, padding: '6px 14px', borderBottom: '1px solid var(--border-color)', fontSize: 12 }}>
                <span style={{ color: 'var(--text-muted)', fontFamily: 'var(--font-mono)', minWidth: 40 }}>Row {e.row}</span>
                <span style={{ color: 'var(--color-expense)' }}>{e.message}</span>
              </div>
            ))}
          </div>
        </div>
      )}

      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <Button
          variant="danger"
          size="sm"
          onClick={handleUndo}
          disabled={undoing}
        >
          {undoing ? 'Undoing…' : 'Undo Import'}
        </Button>
        <div style={{ display: 'flex', gap: 10 }}>
          <Button variant="ghost" onClick={onImportAnother}>Import Another</Button>
          <Button variant="primary" onClick={onViewTransactions}>View Transactions</Button>
        </div>
      </div>
    </div>
  );
}

function StatCard({ label, value, color }: { label: string; value: number; color: string }) {
  return (
    <div style={{
      background: 'var(--bg-card)',
      border: '1px solid var(--border-color)',
      borderRadius: 8,
      padding: '16px',
      textAlign: 'center',
    }}>
      <div style={{ fontSize: 28, fontWeight: 700, color, fontFamily: 'var(--font-mono)' }}>{value}</div>
      <div style={{ fontSize: 12, color: 'var(--text-muted)', marginTop: 4 }}>{label}</div>
    </div>
  );
}
