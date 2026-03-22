import { useState, useCallback } from 'react';
import { encryptionApi } from '../../api/encryption';
import { BrandWordmark } from '../brand/BrandWordmark';
import styles from './UnlockScreen.module.css';

interface Props {
  onUnlocked: () => void;
}

export function UnlockScreen({ onUnlocked }: Props) {
  const [password, setPassword] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  const handleSubmit = useCallback(async (e: React.FormEvent) => {
    e.preventDefault();
    if (!password) return;

    setLoading(true);
    setError('');

    try {
      await encryptionApi.unlock(password);
      onUnlocked();
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }, [password, onUnlocked]);

  return (
    <div className={styles.container}>
      <div className={styles.card}>
        <div className={styles.brandBlock}>
          <BrandWordmark showTagline />
        </div>
        <p className={styles.subtitle}>Your database is encrypted</p>

        <form onSubmit={handleSubmit} className={styles.form}>
          <label htmlFor="unlock-password" className={styles.label}>
            Enter your password to unlock
          </label>
          <input
            id="unlock-password"
            type="password"
            value={password}
            onChange={e => setPassword(e.target.value)}
            className={styles.input}
            placeholder="Database password"
            autoFocus
            aria-required="true"
            aria-describedby={error ? 'unlock-error' : undefined}
          />

          {error && (
            <div id="unlock-error" className={styles.error} role="alert">
              {error}
            </div>
          )}

          <button
            type="submit"
            className={styles.button}
            disabled={loading || !password}
          >
            {loading ? 'Unlocking...' : 'Unlock'}
          </button>
        </form>

        <div className={styles.warning} role="note">
          If you forget your password, data recovery is impossible.
          There is no password reset — your data is encrypted locally and
          only accessible with your password.
        </div>
      </div>
    </div>
  );
}
