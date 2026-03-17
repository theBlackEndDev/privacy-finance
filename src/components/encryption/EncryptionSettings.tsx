import { useState, useEffect } from 'react';
import { encryptionApi } from '../../api/encryption';
import { useToast } from '../ui/Toast';
import { Button } from '../ui/Button';
import { Tooltip } from '../ui/Tooltip';

const INPUT_STYLE: React.CSSProperties = {
  background: 'var(--bg-input)',
  border: '1px solid var(--border-color)',
  borderRadius: 6,
  color: 'var(--text-primary)',
  padding: '7px 10px',
  fontSize: 13,
  width: '100%',
  boxSizing: 'border-box',
};

const LABEL_STYLE: React.CSSProperties = {
  fontSize: 12,
  color: 'var(--text-muted)',
  marginBottom: 4,
  display: 'block',
};

export function EncryptionSettings() {
  const [isEncrypted, setIsEncrypted] = useState(false);
  const [loading, setLoading] = useState(true);
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [currentPassword, setCurrentPassword] = useState('');
  const [newPassword, setNewPassword] = useState('');
  const [busy, setBusy] = useState(false);
  const [showChangeForm, setShowChangeForm] = useState(false);
  const toast = useToast();

  useEffect(() => {
    encryptionApi.checkStatus().then(status => {
      setIsEncrypted(status.isEncrypted);
      setLoading(false);
    }).catch(() => setLoading(false));
  }, []);

  async function handleEnable() {
    if (!password || password !== confirmPassword) {
      toast('Passwords do not match', 'error');
      return;
    }
    setBusy(true);
    try {
      await encryptionApi.enable(password);
      setIsEncrypted(true);
      setPassword('');
      setConfirmPassword('');
      toast('Encryption enabled');
    } catch (err) {
      toast(String(err), 'error');
    } finally {
      setBusy(false);
    }
  }

  async function handleDisable() {
    if (!password) return;
    setBusy(true);
    try {
      await encryptionApi.disable(password);
      setIsEncrypted(false);
      setPassword('');
      toast('Encryption disabled');
    } catch (err) {
      toast(String(err), 'error');
    } finally {
      setBusy(false);
    }
  }

  async function handleChangePassword() {
    if (!currentPassword || !newPassword) return;
    setBusy(true);
    try {
      await encryptionApi.changePassword(currentPassword, newPassword);
      setCurrentPassword('');
      setNewPassword('');
      setShowChangeForm(false);
      toast('Password changed');
    } catch (err) {
      toast(String(err), 'error');
    } finally {
      setBusy(false);
    }
  }

  if (loading) return null;

  return (
    <div>
      <div style={{ fontSize: 12, color: 'var(--text-muted)', marginBottom: 8, fontWeight: 500 }}>
        Database Encryption{' '}
        <Tooltip text="Encrypts your database file on disk using SQLCipher. Your data is unreadable without the password — even if someone copies the file. Keep your password safe; there's no recovery if you forget it." />
      </div>

      <div style={{
        padding: 12,
        border: '1px solid var(--border-color)',
        borderRadius: 6,
        background: 'var(--bg-hover)',
        fontSize: 12,
        display: 'flex',
        flexDirection: 'column',
        gap: 10,
      }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
          <span style={{
            width: 8,
            height: 8,
            borderRadius: '50%',
            background: isEncrypted ? 'var(--color-success)' : 'var(--text-muted)',
          }} />
          <span style={{ color: 'var(--text-primary)', fontWeight: 500 }}>
            {isEncrypted ? 'Encrypted' : 'Not encrypted'}
          </span>
        </div>

        {!isEncrypted && (
          <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
            <div>
              <label htmlFor="enc-password" style={LABEL_STYLE}>Password</label>
              <input
                id="enc-password"
                type="password"
                value={password}
                onChange={e => setPassword(e.target.value)}
                style={INPUT_STYLE}
                placeholder="Choose a strong password"
              />
            </div>
            <div>
              <label htmlFor="enc-confirm" style={LABEL_STYLE}>Confirm password</label>
              <input
                id="enc-confirm"
                type="password"
                value={confirmPassword}
                onChange={e => setConfirmPassword(e.target.value)}
                style={INPUT_STYLE}
                placeholder="Confirm password"
              />
            </div>
            <Button variant="primary" size="sm" disabled={busy || !password || password !== confirmPassword} onClick={handleEnable}>
              {busy ? 'Encrypting...' : 'Enable Encryption'}
            </Button>
          </div>
        )}

        {isEncrypted && (
          <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
            {!showChangeForm && (
              <div style={{ display: 'flex', gap: 8 }}>
                <Button variant="ghost" size="sm" onClick={() => setShowChangeForm(true)}>
                  Change Password
                </Button>
                <div style={{ display: 'flex', gap: 4, alignItems: 'center' }}>
                  <input
                    type="password"
                    value={password}
                    onChange={e => setPassword(e.target.value)}
                    style={{ ...INPUT_STYLE, width: 160 }}
                    placeholder="Current password"
                    aria-label="Current password to disable encryption"
                  />
                  <Button variant="ghost" size="sm" disabled={busy || !password} onClick={handleDisable}>
                    {busy ? 'Disabling...' : 'Disable'}
                  </Button>
                </div>
              </div>
            )}

            {showChangeForm && (
              <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
                <div>
                  <label htmlFor="enc-current" style={LABEL_STYLE}>Current password</label>
                  <input
                    id="enc-current"
                    type="password"
                    value={currentPassword}
                    onChange={e => setCurrentPassword(e.target.value)}
                    style={INPUT_STYLE}
                    placeholder="Current password"
                  />
                </div>
                <div>
                  <label htmlFor="enc-new" style={LABEL_STYLE}>New password</label>
                  <input
                    id="enc-new"
                    type="password"
                    value={newPassword}
                    onChange={e => setNewPassword(e.target.value)}
                    style={INPUT_STYLE}
                    placeholder="New password"
                  />
                </div>
                <div style={{ display: 'flex', gap: 8 }}>
                  <Button variant="ghost" size="sm" onClick={() => { setShowChangeForm(false); setCurrentPassword(''); setNewPassword(''); }}>
                    Cancel
                  </Button>
                  <Button variant="primary" size="sm" disabled={busy || !currentPassword || !newPassword} onClick={handleChangePassword}>
                    {busy ? 'Changing...' : 'Change Password'}
                  </Button>
                </div>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
