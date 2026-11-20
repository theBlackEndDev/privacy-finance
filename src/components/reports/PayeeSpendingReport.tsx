import { useState, useEffect } from 'react';
import { PayeeSpendingRow } from '../../types';
import { reportsApi } from '../../api/reports';
import { formatCurrency } from '../../utils/formatCurrency';
import { useToast } from '../ui/Toast';
import styles from './PayeeSpendingReport.module.css';

interface Props {
  startDate: string;
  endDate: string;
  onData?: (data: unknown) => void;
}

export function PayeeSpendingReport({ startDate, endDate, onData }: Props) {
  const [data, setData] = useState<PayeeSpendingRow[]>([]);
  const toast = useToast();

  useEffect(() => {
    if (!startDate || !endDate) return;
    let cancelled = false;
    reportsApi.getPayeeSpending(startDate, endDate).then((d) => {
      if (cancelled) return;
      setData(d);
      onData?.(d);
    }).catch((err) => {
      if (cancelled) return;
      toast(String(err), 'error');
    });
    return () => { cancelled = true; };
  }, [startDate, endDate, onData, toast]);

  if (data.length === 0) return <div className={styles.empty}>No payee spending data</div>;

  return (
    <div className={styles.table}>
      {data.map((row) => (
        <div key={row.payee} className={styles.tableRow}>
          <span className={styles.name}>{row.payee}</span>
          <span className={styles.count}>{row.count} txns</span>
          <span className={styles.amount}>{formatCurrency(row.total)}</span>
        </div>
      ))}
    </div>
  );
}
