import { useState, useEffect } from 'react';
import { Modal } from '../ui/Modal';
import { Button } from '../ui/Button';
import { settingsApi } from '../../api/settings';
import { useToast } from '../ui/Toast';
import { useApp } from '../../context/AppContext';

interface Props {
  onClose: () => void;
}

const KEYBOARD_SHORTCUTS = [
  ['Ctrl+1', 'Dashboard'],
  ['Ctrl+2', 'Accounts'],
  ['Ctrl+3', 'Transactions'],
  ['Ctrl+4', 'Categories'],
  ['Ctrl+5', 'Import'],
  ['Ctrl+N', 'Focus new transaction'],
  ['Ctrl+F', 'Focus search'],
  ['Escape', 'Close modal'],
];

export function SettingsModal({ onClose }: Props) {
  const [currencySymbol, setCurrencySymbol] = useState('$');
  const [dateFormat, setDateFormat] = useState('MMM D');
  const [saving, setSaving] = useState(false);
  const toast = useToast();
  const { theme, setTheme } = useApp();

  useEffect(() => {
    settingsApi.get('currency_symbol').then(v => { if (v) setCurrencySymbol(v); });
    settingsApi.get('date_format').then(v => { if (v) setDateFormat(v); });
  }, []);

  async function handleSave() {
    setSaving(true);
    try {
      await settingsApi.set('currency_symbol', currencySymbol);
      await settingsApi.set('date_format', dateFormat);
      toast('Settings saved');
      onClose();
    } catch (e) {
      toast(String(e), 'error');
    } finally {
      setSaving(false);
    }
  }

  const selectStyle: React.CSSProperties = {
    background: 'var(--bg-input)',
    border: '1px solid var(--border-color)',
    borderRadius: 6,
    color: 'var(--text-primary)',
    padding: '7px 10px',
    fontSize: 13,
    width: '100%',
  };

  return (
    <Modal title="Settings" onClose={onClose}>
      <div style={{ display: 'flex', flexDirection: 'column', gap: 20, minWidth: 340 }}>

        {/* Currency */}
        <div>
          <div style={{ fontSize: 12, color: 'var(--text-muted)', marginBottom: 6, fontWeight: 500 }}>Currency Symbol</div>
          <select style={selectStyle} value={currencySymbol} onChange={e => setCurrencySymbol(e.target.value)}>
            <option value="$">$ — US Dollar</option>
            <option value="€">€ — Euro</option>
            <option value="£">£ — British Pound</option>
            <option value="¥">¥ — Yen / Yuan</option>
            <option value="₹">₹ — Indian Rupee</option>
            <option value="₩">₩ — Korean Won</option>
            <option value="₣">₣ — Swiss Franc</option>
            <option value="kr">kr — Scandinavian Krone</option>
            <option value="R$">R$ — Brazilian Real</option>
          </select>
        </div>

        {/* Date format */}
        <div>
          <div style={{ fontSize: 12, color: 'var(--text-muted)', marginBottom: 6, fontWeight: 500 }}>Date Display Format</div>
          <select style={selectStyle} value={dateFormat} onChange={e => setDateFormat(e.target.value)}>
            <option value="MMM D">Jan 5</option>
            <option value="MMM D, YYYY">Jan 5, 2024</option>
            <option value="MM/DD/YYYY">01/05/2024</option>
            <option value="DD/MM/YYYY">05/01/2024</option>
            <option value="YYYY-MM-DD">2024-01-05</option>
          </select>
        </div>

        {/* Theme */}
        <div>
          <div style={{ fontSize: 12, color: 'var(--text-muted)', marginBottom: 6, fontWeight: 500 }}>Appearance</div>
          <div style={{ display: 'flex', gap: 8 }}>
            {(['dark', 'light'] as const).map(t => (
              <button
                key={t}
                onClick={() => setTheme(t)}
                style={{
                  flex: 1,
                  padding: '8px 0',
                  borderRadius: 6,
                  border: `1px solid ${theme === t ? 'var(--color-accent)' : 'var(--border-color)'}`,
                  background: theme === t ? 'var(--color-accent)' : 'var(--bg-input)',
                  color: theme === t ? '#fff' : 'var(--text-secondary)',
                  fontSize: 13,
                  cursor: 'pointer',
                  fontWeight: theme === t ? 600 : 400,
                  transition: 'var(--transition-fast)',
                }}
              >
                {t === 'dark' ? '🌙 Dark' : '☀️ Light'}
              </button>
            ))}
          </div>
        </div>

        {/* Keyboard shortcuts reference */}
        <div>
          <div style={{ fontSize: 12, color: 'var(--text-muted)', marginBottom: 8, fontWeight: 500 }}>Keyboard Shortcuts</div>
          <div style={{ border: '1px solid var(--border-color)', borderRadius: 6, overflow: 'hidden' }}>
            {KEYBOARD_SHORTCUTS.map(([key, label]) => (
              <div key={key} style={{
                display: 'flex',
                justifyContent: 'space-between',
                alignItems: 'center',
                padding: '6px 12px',
                borderBottom: '1px solid var(--border-color)',
                fontSize: 12,
              }}>
                <span style={{ color: 'var(--text-muted)' }}>{label}</span>
                <kbd style={{
                  background: 'var(--bg-hover)',
                  border: '1px solid var(--border-color)',
                  borderRadius: 4,
                  padding: '2px 6px',
                  fontFamily: 'var(--font-mono)',
                  fontSize: 11,
                  color: 'var(--text-secondary)',
                }}>
                  {key}
                </kbd>
              </div>
            ))}
          </div>
        </div>

        <div style={{ display: 'flex', justifyContent: 'flex-end', gap: 8 }}>
          <Button variant="ghost" onClick={onClose}>Cancel</Button>
          <Button variant="primary" onClick={handleSave} disabled={saving}>
            {saving ? 'Saving…' : 'Save'}
          </Button>
        </div>
      </div>
    </Modal>
  );
}
