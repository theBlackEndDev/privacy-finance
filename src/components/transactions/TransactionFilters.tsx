import { useState, useEffect } from 'react';
import { Account, Category, TransactionFilters } from '../../types';
import { useDebounce } from '../../hooks/useDebounce';
import styles from './TransactionFilters.module.css';

interface TransactionFiltersProps {
  filters: TransactionFilters;
  accounts: Account[];
  categories: Category[];
  onChange: (filters: TransactionFilters) => void;
}

export function TransactionFiltersBar({ filters, accounts, categories, onChange }: TransactionFiltersProps) {
  const [search, setSearch] = useState(filters.search ?? '');
  const debouncedSearch = useDebounce(search, 300);

  useEffect(() => {
    onChange({ ...filters, search: debouncedSearch || undefined, page: 0 });
  }, [debouncedSearch]);

  return (
    <div className={styles.bar}>
      <div className={styles.field}>
        <span className={styles.label}>Search</span>
        <input
          className={styles.searchInput}
          placeholder="Payee or description…"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />
      </div>

      <div className={styles.field}>
        <span className={styles.label}>Account</span>
        <select
          className={styles.select}
          value={filters.accountId ?? ''}
          onChange={(e) => onChange({ ...filters, accountId: e.target.value || undefined, page: 0 })}
        >
          <option value="">All Accounts</option>
          {accounts.filter(a => !a.isArchived).map((a) => (
            <option key={a.id} value={a.id}>{a.name}</option>
          ))}
        </select>
      </div>

      <div className={styles.field}>
        <span className={styles.label}>Kind</span>
        <select
          className={styles.select}
          value={filters.kind ?? ''}
          onChange={(e) => onChange({ ...filters, kind: e.target.value || undefined, page: 0 })}
        >
          <option value="">All</option>
          <option value="income">Income</option>
          <option value="expense">Expense</option>
          <option value="transfer">Transfer</option>
        </select>
      </div>

      <div className={styles.field}>
        <span className={styles.label}>Category</span>
        <select
          className={styles.select}
          value={filters.categoryId ?? ''}
          onChange={(e) => onChange({ ...filters, categoryId: e.target.value || undefined, page: 0 })}
        >
          <option value="">All</option>
          {categories.map((c) => (
            <option key={c.id} value={c.id}>{c.name}</option>
          ))}
        </select>
      </div>

      <div className={styles.field}>
        <span className={styles.label}>From</span>
        <input
          type="date"
          className={styles.dateInput}
          value={filters.dateFrom ?? ''}
          onChange={(e) => onChange({ ...filters, dateFrom: e.target.value || undefined, page: 0 })}
        />
      </div>

      <div className={styles.field}>
        <span className={styles.label}>To</span>
        <input
          type="date"
          className={styles.dateInput}
          value={filters.dateTo ?? ''}
          onChange={(e) => onChange({ ...filters, dateTo: e.target.value || undefined, page: 0 })}
        />
      </div>
    </div>
  );
}
