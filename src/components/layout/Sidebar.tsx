import { useState } from 'react';
import { useApp } from '../../context/AppContext';
import { useAccounts } from '../../context/AccountContext';
import { View } from '../../types';
import { cx } from '../../utils/classNames';
import { SettingsModal } from './SettingsModal';
import styles from './Sidebar.module.css';

const NAV_ITEMS: { view: View; label: string; icon: string }[] = [
  { view: 'dashboard',    label: 'Dashboard',    icon: '◈' },
  { view: 'accounts',     label: 'Accounts',     icon: '◉' },
  { view: 'transactions', label: 'Transactions', icon: '⇅' },
  { view: 'categories',   label: 'Categories',   icon: '◫' },
  { view: 'import',       label: 'Import',       icon: '↑' },
  { view: 'recurring',    label: 'Recurring',    icon: '↻' },
];

export function Sidebar() {
  const { currentView, navigate, setActiveAccountId } = useApp();
  const { accounts } = useAccounts();
  const [showSettings, setShowSettings] = useState(false);

  const activeAccounts = accounts.filter((a) => !a.isArchived);

  function handleAccountClick(id: string) {
    setActiveAccountId(id);
    navigate('transactions');
  }

  return (
    <>
      <aside className={styles.sidebar}>
        <div className={styles.logo}>
          Privacy<span className={styles.logoAccent}>Finance</span>
        </div>

        <nav className={styles.nav}>
          {NAV_ITEMS.map(({ view, label, icon }) => (
            <button
              key={view}
              className={cx(styles.navItem, currentView === view && styles.navItemActive)}
              onClick={() => navigate(view)}
            >
              <span className={styles.navIcon}>{icon}</span>
              {label}
            </button>
          ))}
        </nav>

        {activeAccounts.length > 0 && (
          <div className={styles.accountsSection}>
            <div className={styles.sectionLabel}>Accounts</div>
            {activeAccounts.map((account) => (
              <button
                key={account.id}
                className={styles.accountItem}
                onClick={() => handleAccountClick(account.id)}
                title={account.name}
              >
                <span className={styles.accountDot} />
                <span className={styles.accountName}>{account.name}</span>
              </button>
            ))}
          </div>
        )}

        <button className={styles.settingsBtn} onClick={() => setShowSettings(true)}>
          <span className={styles.navIcon}>⚙</span>
          Settings
        </button>
      </aside>

      {showSettings && <SettingsModal onClose={() => setShowSettings(false)} />}
    </>
  );
}
