import { Transaction, TransactionPage } from '../../types';
import { TransactionRow } from './TransactionRow';
import { EmptyState } from '../ui/EmptyState';
import { Button } from '../ui/Button';
import styles from './TransactionList.module.css';

interface TransactionListProps {
  page: TransactionPage | null;
  onPageChange: (page: number) => void;
  onRowClick: (t: Transaction) => void;
}

export function TransactionList({ page, onPageChange, onRowClick }: TransactionListProps) {
  if (!page || page.items.length === 0) {
    return <EmptyState title="No transactions" description="Add your first transaction above." />;
  }

  const totalPages = Math.ceil(page.total / page.pageSize);
  const currentPage = page.page;
  const start = currentPage * page.pageSize + 1;
  const end = Math.min(start + page.items.length - 1, page.total);

  return (
    <div className={styles.wrapper} role="table" aria-label="Transactions">
      <div className={styles.thead} role="row">
        <span className={styles.th} role="columnheader">Date</span>
        <span className={styles.th} role="columnheader">Payee / Description</span>
        <span className={styles.th} role="columnheader">Category</span>
        <span className={`${styles.th} ${styles.thAmount}`} role="columnheader">Amount</span>
        <span className={styles.th} role="columnheader">Cleared</span>
      </div>

      {page.items.map((t) => (
        <TransactionRow key={t.id} transaction={t} onClick={onRowClick} />
      ))}

      {totalPages > 1 && (
        <div className={styles.pagination}>
          <span className={styles.pageInfo}>{start}&ndash;{end} of {page.total}</span>
          <div className={styles.pageButtons}>
            <Button variant="ghost" size="sm" disabled={currentPage === 0}
              onClick={() => onPageChange(currentPage - 1)}
              aria-label="Go to previous page">
              &larr; Prev
            </Button>
            <Button variant="ghost" size="sm" disabled={currentPage >= totalPages - 1}
              onClick={() => onPageChange(currentPage + 1)}
              aria-label="Go to next page">
              Next &rarr;
            </Button>
          </div>
        </div>
      )}
    </div>
  );
}
