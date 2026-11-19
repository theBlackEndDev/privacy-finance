import { TableCardData } from '../../../types';
import { formatCurrency } from '../../../utils/formatCurrency';
import styles from './Cards.module.css';

const CENT_COLUMN = /amount|total|balance/i;

function formatCell(value: unknown, column: string): string {
  if (value === null || value === undefined) return '';
  if (typeof value === 'number' && Number.isInteger(value) && CENT_COLUMN.test(column)) {
    return formatCurrency(value);
  }
  if (typeof value === 'object') return JSON.stringify(value);
  return String(value);
}

export function TableCard({ data }: { data: TableCardData }) {
  return (
    <div className={styles.card}>
      <div className={styles.tableWrap}>
        <table className={styles.table}>
          <thead>
            <tr>
              {data.columns.map((c, i) => <th key={i}>{c}</th>)}
            </tr>
          </thead>
          <tbody>
            {data.rows.map((row, r) => (
              <tr key={r}>
                {row.map((cell, c) => (
                  <td key={c} className={typeof cell === 'number' ? styles.tableNum : undefined}>
                    {formatCell(cell, data.columns[c] ?? '')}
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
