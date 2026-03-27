import { useApp } from '../../context/AppContext';
import { View } from '../../types';
import { GlobalSearch } from '../search/GlobalSearch';
import styles from './Header.module.css';

const VIEW_TITLES: Record<View, string> = {
  dashboard:        'Dashboard',
  accounts:         'Accounts',
  'account-detail': 'Account',
  transactions:     'Transactions',
  categories:   'Categories',
  import:       'Import',
  recurring:    'Recurring',
  budgets:      'Budgets',
  rules:        'Rules',
  reports:      'Reports',
  reconciliation: 'Reconciliation',
  agents:         'Agent Gateway',
};

export function Header() {
  const { currentView } = useApp();

  return (
    <header className={styles.header}>
      <h1 className={styles.title}>{VIEW_TITLES[currentView]}</h1>
      <div className={styles.right}>
        <GlobalSearch />
      </div>
    </header>
  );
}
