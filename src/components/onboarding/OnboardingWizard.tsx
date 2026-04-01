import { useState, FormEvent } from 'react';
import { Account, Category } from '../../types';
import { accountsApi } from '../../api/accounts';
import { settingsApi } from '../../api/settings';
import { Input } from '../ui/Input';
import { Button } from '../ui/Button';
import { Modal } from '../ui/Modal';
import { ACCOUNT_TYPE_OPTIONS } from '../../utils/accountTypes';
import { parseCurrency } from '../../utils/formatCurrency';
import { cx } from '../../utils/classNames';
import styles from './OnboardingWizard.module.css';

const STEPS = { WELCOME: 0, CREATE_ACCOUNT: 1, CATEGORIES: 2 } as const;
type Step = (typeof STEPS)[keyof typeof STEPS];
const TOTAL_STEPS = 3;

interface OnboardingWizardProps {
  categories: Category[];
  onComplete: () => void;
}

export function OnboardingWizard({ categories, onComplete }: OnboardingWizardProps) {
  const [step, setStep] = useState<Step>(STEPS.WELCOME);
  const [name, setName] = useState('');
  const [type, setType] = useState<Account['type']>('checking');
  const [openingBalance, setOpeningBalance] = useState('0.00');
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState('');

  async function completeOnboarding() {
    await settingsApi.set('onboarding_complete', 'true');
    onComplete();
  }

  async function handleCreateAccount(e: FormEvent) {
    e.preventDefault();
    if (!name.trim()) {
      setError('Account name is required');
      return;
    }
    setSaving(true);
    setError('');
    try {
      await accountsApi.create({
        name: name.trim(),
        type,
        openingBalance: parseCurrency(openingBalance),
      });
      setStep(STEPS.CATEGORIES);
    } catch (err) {
      setError(String(err));
    } finally {
      setSaving(false);
    }
  }

  return (
    <Modal title="Welcome to PrivacyFinance" onClose={completeOnboarding}>
      <div data-testid="onboarding-wizard">
      <StepIndicator current={step} total={TOTAL_STEPS} />

      {step === STEPS.WELCOME && (
        <div className={styles.welcome}>
          <h2 className={styles.welcomeTitle}>Your finances, your privacy</h2>
          <p className={styles.welcomeDescription}>
            PrivacyFinance keeps all your financial data on your device.
            No cloud sync, no tracking, no third-party access. Let's set up
            your first account to get started.
          </p>
          <div className={styles.footer}>
            <Button variant="ghost" onClick={completeOnboarding} data-testid="onboarding-skip">Skip setup</Button>
            <Button onClick={() => setStep(STEPS.CREATE_ACCOUNT)} data-testid="onboarding-next">Get started</Button>
          </div>
        </div>
      )}

      {step === STEPS.CREATE_ACCOUNT && (
        <form className={styles.form} onSubmit={handleCreateAccount}>
          <Input
            label="Account Name"
            value={name}
            onChange={(e) => { setName(e.target.value); setError(''); }}
            placeholder="e.g. Primary Checking"
            autoFocus
            error={error && !name.trim() ? error : undefined}
            data-testid="onboarding-account-name"
          />

          <div>
            <label className={styles.selectLabel}>Account Type</label>
            <select
              className={styles.select}
              value={type}
              onChange={(e) => { setType(e.target.value as Account['type']); setError(''); }}
              data-testid="onboarding-account-type"
            >
              {ACCOUNT_TYPE_OPTIONS.map((t) => (
                <option key={t.value} value={t.value}>{t.label}</option>
              ))}
            </select>
          </div>

          <Input
            label="Opening Balance"
            value={openingBalance}
            onChange={(e) => { setOpeningBalance(e.target.value); setError(''); }}
            placeholder="0.00"
            data-testid="onboarding-balance"
          />

          {error && name.trim() && <p className={styles.error}>{error}</p>}

          <div className={styles.footer}>
            <Button variant="ghost" type="button" onClick={completeOnboarding} data-testid="onboarding-skip">Skip setup</Button>
            <div className={styles.footerRight}>
              <Button variant="ghost" type="button" onClick={() => setStep(STEPS.WELCOME)}>Back</Button>
              <Button type="submit" disabled={saving} data-testid="onboarding-next">
                {saving ? 'Creating...' : 'Create account'}
              </Button>
            </div>
          </div>
        </form>
      )}

      {step === STEPS.CATEGORIES && (
        <div className={styles.form}>
          <p className={styles.welcomeDescription}>
            Your account has been created. Here are the default categories
            you can use to organize transactions:
          </p>
          <div className={styles.categoryList}>
            {categories.map(c => (
              <div key={c.id} className={styles.categoryItem}>
                <span className={styles.categoryDot} style={{ background: c.color }} />
                {c.name}
              </div>
            ))}
          </div>
          <p className={styles.categoryHint}>
            You can add, edit, or remove categories anytime from the Categories page.
          </p>
          <div className={styles.footerEnd}>
            <Button onClick={completeOnboarding} data-testid="onboarding-finish">Start using PrivacyFinance</Button>
          </div>
        </div>
      )}
      </div>
    </Modal>
  );
}

function StepIndicator({ current, total }: { current: number; total: number }) {
  return (
    <div className={styles.stepIndicator}>
      {Array.from({ length: total }, (_, i) => (
        <div key={i} className={cx(styles.dot, i <= current && styles.dotActive)} />
      ))}
    </div>
  );
}
