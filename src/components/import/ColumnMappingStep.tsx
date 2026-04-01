import { useState } from 'react';
import { ColumnMapping, ImportProfile } from '../../types';
import { importApi } from '../../api/import';
import { Button } from '../ui/Button';

const TARGET_FIELDS = [
  { value: '', label: '— skip —' },
  { value: 'date', label: 'Date' },
  { value: 'payee', label: 'Payee' },
  { value: 'description', label: 'Description' },
  { value: 'amount', label: 'Amount (single)' },
  { value: 'debit', label: 'Debit (outflow)' },
  { value: 'credit', label: 'Credit (inflow)' },
];

const DATE_FORMATS = [
  { value: '%Y-%m-%d', label: 'YYYY-MM-DD  (2024-01-31)' },
  { value: '%m/%d/%Y', label: 'MM/DD/YYYY  (01/31/2024)' },
  { value: '%d/%m/%Y', label: 'DD/MM/YYYY  (31/01/2024)' },
  { value: '%m-%d-%Y', label: 'MM-DD-YYYY  (01-31-2024)' },
  { value: '%d-%m-%Y', label: 'DD-MM-YYYY  (31-01-2024)' },
  { value: '%Y/%m/%d', label: 'YYYY/MM/DD  (2024/01/31)' },
  { value: '%b %d, %Y', label: 'Mon DD, YYYY  (Jan 31, 2024)' },
  { value: '%d %b %Y', label: 'DD Mon YYYY  (31 Jan 2024)' },
];

interface Props {
  headers: string[];
  profiles: ImportProfile[];
  onBack: () => void;
  onNext: (mappings: ColumnMapping[], dateFormat: string, amountFormat: string) => void;
  onProfilesChange: (profiles: ImportProfile[]) => void;
}

export function ColumnMappingStep({ headers, profiles, onBack, onNext, onProfilesChange }: Props) {
  const [fieldMap, setFieldMap] = useState<Record<string, string>>(() => {
    // Auto-guess by header name
    const guess: Record<string, string> = {};
    headers.forEach(h => {
      const lower = h.toLowerCase();
      if (lower.includes('date')) guess[h] = 'date';
      else if (lower.includes('payee') || lower.includes('description') || lower.includes('merchant') || lower.includes('memo')) {
        guess[h] = guess[h] ?? (lower.includes('payee') || lower.includes('merchant') ? 'payee' : 'description');
      }
      else if (lower === 'amount' || lower === 'transaction amount') guess[h] = 'amount';
      else if (lower.includes('debit') || lower === 'withdrawal') guess[h] = 'debit';
      else if (lower.includes('credit') || lower === 'deposit') guess[h] = 'credit';
    });
    return guess;
  });
  const [dateFormat, setDateFormat] = useState('%Y-%m-%d');
  const [amountFormat, setAmountFormat] = useState('single');
  const [profileName, setProfileName] = useState('');
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState('');

  const hasDebitCredit = Object.values(fieldMap).includes('debit') || Object.values(fieldMap).includes('credit');

  function loadProfile(profile: ImportProfile) {
    try {
      const mappings: ColumnMapping[] = JSON.parse(profile.mapping);
      const newMap: Record<string, string> = {};
      mappings.forEach(m => { newMap[m.csvColumn] = m.field; });
      setFieldMap(newMap);
    } catch { /* ignore malformed */ }
  }

  async function saveProfile() {
    if (!profileName.trim()) return;
    setSaving(true);
    try {
      const mappings = buildMappings();
      await importApi.saveImportProfile(profileName.trim(), mappings);
      const updated = await importApi.getImportProfiles();
      onProfilesChange(updated);
      setProfileName('');
    } finally {
      setSaving(false);
    }
  }

  function buildMappings(): ColumnMapping[] {
    return headers
      .filter(h => fieldMap[h])
      .map(h => ({ csvColumn: h, field: fieldMap[h] }));
  }

  function handleNext() {
    const mappings = buildMappings();
    const hasDate = mappings.some(m => m.field === 'date');
    const hasAmount = mappings.some(m => m.field === 'amount') ||
      (mappings.some(m => m.field === 'debit') || mappings.some(m => m.field === 'credit'));
    if (!hasDate) { setError('Map at least one column to Date.'); return; }
    if (!hasAmount) { setError('Map at least one column to Amount, Debit, or Credit.'); return; }
    setError('');
    const fmt = hasDebitCredit ? 'debit_credit' : amountFormat;
    onNext(mappings, dateFormat, fmt);
  }

  const selectStyle: React.CSSProperties = {
    background: 'var(--bg-input)',
    border: '1px solid var(--border-color)',
    borderRadius: 6,
    color: 'var(--text-primary)',
    padding: '6px 8px',
    fontSize: 12,
    minWidth: 180,
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>

      {/* Saved profiles */}
      {profiles.length > 0 && (
        <div>
          <div style={{ fontSize: 12, color: 'var(--text-muted)', marginBottom: 6 }}>Load saved profile</div>
          <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap' }}>
            {profiles.map(p => (
              <Button key={p.id} variant="ghost" size="sm" onClick={() => loadProfile(p)}>
                {p.name}
              </Button>
            ))}
          </div>
        </div>
      )}

      {/* Column mapping table */}
      <div style={{ border: '1px solid var(--border-color)', borderRadius: 6, overflow: 'hidden' }}>
        <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 13 }}>
          <thead>
            <tr style={{ background: 'var(--bg-sidebar)' }}>
              <th style={{ padding: '8px 14px', textAlign: 'left', color: 'var(--text-secondary)', fontWeight: 600, borderBottom: '1px solid var(--border-color)' }}>CSV Column</th>
              <th style={{ padding: '8px 14px', textAlign: 'left', color: 'var(--text-secondary)', fontWeight: 600, borderBottom: '1px solid var(--border-color)' }}>Maps to</th>
            </tr>
          </thead>
          <tbody>
            {headers.map(h => (
              <tr key={h} style={{ borderBottom: '1px solid var(--border-color)' }}>
                <td style={{ padding: '8px 14px', color: 'var(--text-primary)', fontFamily: 'var(--font-mono)', fontSize: 12 }}>{h}</td>
                <td style={{ padding: '6px 14px' }}>
                  <select
                    style={selectStyle}
                    value={fieldMap[h] ?? ''}
                    onChange={e => setFieldMap(prev => ({ ...prev, [h]: e.target.value }))}
                  >
                    {TARGET_FIELDS.map(f => (
                      <option key={f.value} value={f.value}>{f.label}</option>
                    ))}
                  </select>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {/* Date format + amount format */}
      <div style={{ display: 'flex', gap: 24, flexWrap: 'wrap' }}>
        <div>
          <div style={{ fontSize: 12, color: 'var(--text-muted)', marginBottom: 6 }}>Date format</div>
          <select style={selectStyle} value={dateFormat} onChange={e => setDateFormat(e.target.value)}>
            {DATE_FORMATS.map(f => (
              <option key={f.value} value={f.value}>{f.label}</option>
            ))}
          </select>
        </div>
        {!hasDebitCredit && (
          <div>
            <div style={{ fontSize: 12, color: 'var(--text-muted)', marginBottom: 6 }}>Amount format</div>
            <select style={selectStyle} value={amountFormat} onChange={e => setAmountFormat(e.target.value)}>
              <option value="single">Single column (negative = expense)</option>
              <option value="debit_credit">Debit / Credit columns</option>
            </select>
          </div>
        )}
      </div>

      {/* Save profile */}
      <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
        <input
          type="text"
          placeholder="Save as profile…"
          value={profileName}
          onChange={e => setProfileName(e.target.value)}
          style={{
            background: 'var(--bg-input)',
            border: '1px solid var(--border-color)',
            borderRadius: 6,
            color: 'var(--text-primary)',
            padding: '6px 10px',
            fontSize: 12,
            width: 180,
          }}
        />
        <Button variant="ghost" size="sm" onClick={saveProfile} disabled={saving || !profileName.trim()}>
          Save
        </Button>
      </div>

      {error && <div style={{ fontSize: 13, color: 'var(--color-expense)' }}>{error}</div>}

      <div style={{ display: 'flex', justifyContent: 'space-between' }}>
        <Button variant="ghost" onClick={onBack}>← Back</Button>
        <Button variant="primary" onClick={handleNext} data-testid="import-mapping-next">Import →</Button>
      </div>
    </div>
  );
}
