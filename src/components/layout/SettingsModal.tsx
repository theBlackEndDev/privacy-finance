import { useState, useEffect } from 'react';
import { save, open } from '@tauri-apps/plugin-dialog';
import { Modal } from '../ui/Modal';
import { Button } from '../ui/Button';
import { settingsApi } from '../../api/settings';
import { backupApi } from '../../api/backup';
import { useToast } from '../ui/Toast';
import { useApp } from '../../context/AppContext';
import { todayISO } from '../../utils/formatDate';
import { BackupInfo } from '../../types';
import { ExchangeRateSettings } from '../settings/ExchangeRateSettings';
import { EncryptionSettings } from '../encryption/EncryptionSettings';
import { AboutDialog } from '../ui/AboutDialog';

interface Props {
  onClose: () => void;
}

const KEYBOARD_SHORTCUTS = [
  ['Ctrl+1', 'Dashboard'],
  ['Ctrl+2', 'Accounts'],
  ['Ctrl+3', 'Transactions'],
  ['Ctrl+4', 'Categories'],
  ['Ctrl+5', 'Import'],
  ['Ctrl+7', 'Budgets'],
  ['Ctrl+B', 'Budgets (alt)'],
  ['Ctrl+N', 'Focus new transaction'],
  ['Ctrl+F', 'Focus search'],
  ['Escape', 'Close modal'],
];

export function SettingsModal({ onClose }: Props) {
  const [currencySymbol, setCurrencySymbol] = useState('$');
  const [dateFormat, setDateFormat] = useState('MMM D');
  const [saving, setSaving] = useState(false);
  const [backupBusy, setBackupBusy] = useState(false);
  const [backupInfo, setBackupInfo] = useState<BackupInfo | null>(null);
  const [confirmRestore, setConfirmRestore] = useState<string | null>(null);
  const [showAbout, setShowAbout] = useState(false);
  const [deleteStep, setDeleteStep] = useState(0); // 0=hidden, 1=first confirm, 2=final confirm
  const [deleting, setDeleting] = useState(false);
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

  return (<>
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

        {/* Backup & Restore */}
        <div>
          <div style={{ fontSize: 12, color: 'var(--text-muted)', marginBottom: 8, fontWeight: 500 }}>Backup & Restore</div>
          <div style={{ display: 'flex', gap: 8 }}>
            <Button
              variant="ghost"
              size="sm"
              disabled={backupBusy}
              onClick={async () => {
                const path = await save({
                  defaultPath: `privacyfinance-backup-${todayISO()}.db`,
                  filters: [{ name: 'SQLite Database', extensions: ['db'] }],
                });
                if (!path) return;
                setBackupBusy(true);
                try {
                  await backupApi.createBackup(path);
                  toast('Backup created successfully');
                } catch (e) {
                  toast(String(e), 'error');
                } finally {
                  setBackupBusy(false);
                }
              }}
            >
              {backupBusy ? 'Working...' : 'Create Backup'}
            </Button>
            <Button
              variant="ghost"
              size="sm"
              disabled={backupBusy}
              onClick={async () => {
                const path = await open({
                  filters: [{ name: 'SQLite Database', extensions: ['db'] }],
                  multiple: false,
                  directory: false,
                });
                if (!path) return;
                setBackupBusy(true);
                try {
                  const info = await backupApi.getBackupInfo(path);
                  setBackupInfo(info);
                  setConfirmRestore(path);
                } catch (e) {
                  toast(String(e), 'error');
                } finally {
                  setBackupBusy(false);
                }
              }}
            >
              Restore from Backup
            </Button>
          </div>

          {/* Restore confirmation */}
          {confirmRestore && backupInfo && (
            <div style={{
              marginTop: 8,
              padding: 12,
              border: '1px solid var(--border-color)',
              borderRadius: 6,
              background: 'var(--bg-hover)',
              fontSize: 12,
            }}>
              <div style={{ fontWeight: 500, marginBottom: 6, color: 'var(--text-primary)' }}>
                Restore this backup?
              </div>
              <div style={{ color: 'var(--text-muted)', lineHeight: 1.6 }}>
                Accounts: {backupInfo.accountCount} | Categories: {backupInfo.categoryCount}<br />
                Transactions: {backupInfo.transactionCount} | Recurring: {backupInfo.recurringCount}<br />
                {backupInfo.dateRangeFrom && backupInfo.dateRangeTo && (
                  <>Date range: {backupInfo.dateRangeFrom} to {backupInfo.dateRangeTo}<br /></>
                )}
                Size: {(backupInfo.fileSizeBytes / 1024).toFixed(1)} KB
              </div>
              <div style={{ display: 'flex', gap: 8, marginTop: 8 }}>
                <Button variant="ghost" size="sm" onClick={() => { setConfirmRestore(null); setBackupInfo(null); }}>
                  Cancel
                </Button>
                <Button
                  variant="primary"
                  size="sm"
                  disabled={backupBusy}
                  onClick={async () => {
                    setBackupBusy(true);
                    try {
                      await backupApi.restoreBackup(confirmRestore);
                      toast('Backup restored — please restart the app');
                      setConfirmRestore(null);
                      setBackupInfo(null);
                    } catch (e) {
                      toast(String(e), 'error');
                    } finally {
                      setBackupBusy(false);
                    }
                  }}
                >
                  Confirm Restore
                </Button>
              </div>
            </div>
          )}
        </div>

        {/* Database Encryption */}
        <EncryptionSettings />

        {/* Exchange Rates */}
        <ExchangeRateSettings />

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

        {/* Danger Zone */}
        <div>
          <div style={{ fontSize: 12, color: 'var(--color-danger, #ef4444)', marginBottom: 8, fontWeight: 600 }}>Danger Zone</div>
          {deleteStep === 0 && (
            <Button
              variant="danger"
              size="sm"
              onClick={() => setDeleteStep(1)}
            >
              Delete All Data
            </Button>
          )}
          {deleteStep === 1 && (
            <div style={{
              padding: 12,
              border: '1px solid var(--color-danger, #ef4444)',
              borderRadius: 6,
              background: 'rgba(239, 68, 68, 0.05)',
            }}>
              <div style={{ fontSize: 13, fontWeight: 600, color: 'var(--color-danger, #ef4444)', marginBottom: 6 }}>
                Are you sure?
              </div>
              <div style={{ fontSize: 12, color: 'var(--text-muted)', marginBottom: 10, lineHeight: 1.5 }}>
                This will permanently delete all accounts, transactions, categories, budgets, agents, and settings. Default categories will be re-created. This cannot be undone.
              </div>
              <div style={{ display: 'flex', gap: 8 }}>
                <Button variant="ghost" size="sm" onClick={() => setDeleteStep(0)}>Cancel</Button>
                <Button variant="danger" size="sm" onClick={() => setDeleteStep(2)}>
                  Yes, I want to delete everything
                </Button>
              </div>
            </div>
          )}
          {deleteStep === 2 && (
            <div style={{
              padding: 12,
              border: '2px solid var(--color-danger, #ef4444)',
              borderRadius: 6,
              background: 'rgba(239, 68, 68, 0.1)',
            }}>
              <div style={{ fontSize: 13, fontWeight: 700, color: 'var(--color-danger, #ef4444)', marginBottom: 6 }}>
                Final confirmation
              </div>
              <div style={{ fontSize: 12, color: 'var(--text-muted)', marginBottom: 10, lineHeight: 1.5 }}>
                This is your last chance. All data will be permanently erased. There is no undo.
              </div>
              <div style={{ display: 'flex', gap: 8 }}>
                <Button variant="ghost" size="sm" onClick={() => setDeleteStep(0)}>Cancel</Button>
                <Button
                  variant="danger"
                  size="sm"
                  disabled={deleting}
                  onClick={async () => {
                    setDeleting(true);
                    try {
                      await settingsApi.deleteAllData();
                      toast('All data deleted. Please restart the app.');
                      setDeleteStep(0);
                      onClose();
                    } catch (e) {
                      toast(String(e), 'error');
                    } finally {
                      setDeleting(false);
                    }
                  }}
                >
                  {deleting ? 'Deleting...' : 'DELETE EVERYTHING'}
                </Button>
              </div>
            </div>
          )}
        </div>

        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <Button variant="ghost" size="sm" onClick={() => setShowAbout(true)}>
            About PrivacyFinance
          </Button>
          <div style={{ display: 'flex', gap: 8 }}>
            <Button variant="ghost" onClick={onClose}>Cancel</Button>
            <Button variant="primary" onClick={handleSave} disabled={saving}>
              {saving ? 'Saving…' : 'Save'}
            </Button>
          </div>
        </div>
      </div>

    </Modal>
    {showAbout && <AboutDialog onClose={() => setShowAbout(false)} />}
  </>
  );
}
