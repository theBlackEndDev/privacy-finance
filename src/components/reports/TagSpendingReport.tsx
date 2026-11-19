import { useState, useEffect } from 'react';
import { TagSpendingRow } from '../../types';
import { reportsApi } from '../../api/reports';
import { formatCurrency } from '../../utils/formatCurrency';
import { useToast } from '../ui/Toast';
import styles from './TagSpendingReport.module.css';

interface Props {
  startDate: string;
  endDate: string;
  onData?: (data: unknown) => void;
}

export function TagSpendingReport({ startDate, endDate, onData }: Props) {
  const [data, setData] = useState<TagSpendingRow[]>([]);
  const toast = useToast();

  useEffect(() => {
    if (!startDate || !endDate) return;
    let cancelled = false;
    reportsApi.getTagSpending(startDate, endDate).then((d) => {
      if (cancelled) return;
      setData(d);
      onData?.(d);
    }).catch((err) => {
      if (cancelled) return;
      toast(String(err), 'error');
    });
    return () => { cancelled = true; };
  }, [startDate, endDate, onData, toast]);

  if (data.length === 0) return <div className={styles.empty}>No tag spending data</div>;

  return (
    <div className={styles.table}>
      {data.map((row) => (
        <div key={row.tagId} className={styles.tableRow}>
          <span className={styles.dot} style={{ background: row.tagColor ?? '#0d9488' }} />
          <span className={styles.name}>{row.tagName}</span>
          <span className={styles.amount}>{formatCurrency(row.total)}</span>
        </div>
      ))}
    </div>
  );
}
