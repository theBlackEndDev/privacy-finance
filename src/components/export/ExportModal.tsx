import { useState } from 'react';
import { save } from '@tauri-apps/plugin-dialog';
import { Modal } from '../ui/Modal';
import { Button } from '../ui/Button';
import { useAccounts } from '../../context/AccountContext';
import { useToast } from '../ui/Toast';
import { exportApi } from '../../api/export';
import { todayISO } from '../../utils/formatDate';
import { ExportFormat, ExportFilters } from '../../types';
import styles from './ExportModal.module.css';

interface Props {
  onClose: () => void;
}

const FORMAT_OPTIONS: { value: ExportFormat; label: string; ext: string }[] = [
  { value: 'csv', label: 'CSV (Spreadsheet)', ext: 'csv' },
  { value: 'json', label: 'JSON (Transactions)', ext: 'json' },
  { value: 'full-json', label: 'JSON (Full Data Dump)', ext: 'json' },
];

export function ExportModal({ onClose }: Props) {
  const { accounts } = useAccounts();
  const toast = useToast();
  const [format, setFormat] = useState<ExportFormat>('csv');
  const [dateFrom, setDateFrom] = useState('');
  const [dateTo, setDateTo] = useState('');
  const [accountId, setAccountId] = useState('');
  const [exporting, setExporting] = useState(false);
  const [status, setStatus] = useState('');

  const isFullDump = format === 'full-json';

  async function handleExport() {
    const selected = FORMAT_OPTIONS.find(f => f.value === format)!;
    const defaultName = `privacyfinance-export-${todayISO()}.${selected.ext}`;

    const path = await save({
      defaultPath: defaultName,
      filters: [{ name: selected.label, extensions: [selected.ext] }],
    });

    if (!path) return;

    setExporting(true);
    setStatus('');

    try {
      const filters: ExportFilters = {
        dateFrom: dateFrom || undefined,
        dateTo: dateTo || undefined,
        accountId: accountId || undefined,
      };

      if (format === 'csv') {
        const count = await exportApi.exportCsv(path, filters);
        setStatus(`Exported ${count} transactions`);
      } else if (format === 'json') {
        const count = await exportApi.exportJson(path, filters);
        setStatus(`Exported ${count} transactions`);
      } else {
        await exportApi.exportAll(path);
        setStatus('Full data export complete');
      }
      toast('Export saved successfully');
    } catch (e) {
      toast(String(e), 'error');
      setStatus('');
    } finally {
      setExporting(false);
    }
  }

  return (
    <Modal title="Export Data" onClose={onClose}>
      <div className={styles.form}>
        {/* Format selector */}
        <div className={styles.field}>
          <span className={styles.label}>Export Format</span>
          <select
            className={styles.select}
            value={format}
            onChange={e => setFormat(e.target.value as ExportFormat)}
            data-testid="export-format-select"
          >
            {FORMAT_OPTIONS.map(opt => (
              <option key={opt.value} value={opt.value}>{opt.label}</option>
            ))}
          </select>
        </div>

        {/* Date range filters (hidden for full dump) */}
        {!isFullDump && (
          <div className={styles.field}>
            <span className={styles.label}>Date Range (optional)</span>
            <div className={styles.dateRow}>
              <input
                type="date"
                className={styles.input}
                value={dateFrom}
                onChange={e => setDateFrom(e.target.value)}
                placeholder="From"
              />
              <input
                type="date"
                className={styles.input}
                value={dateTo}
                onChange={e => setDateTo(e.target.value)}
                placeholder="To"
              />
            </div>
          </div>
        )}

        {/* Account filter (hidden for full dump) */}
        {!isFullDump && (
          <div className={styles.field}>
            <span className={styles.label}>Account (optional)</span>
            <select
              className={styles.select}
              value={accountId}
              onChange={e => setAccountId(e.target.value)}
            >
              <option value="">All Accounts</option>
              {accounts.map(a => (
                <option key={a.id} value={a.id}>{a.name}</option>
              ))}
            </select>
          </div>
        )}

        {/* Status */}
        {status && (
          <div className={`${styles.status} ${styles.statusSuccess}`}>{status}</div>
        )}

        {/* Actions */}
        <div className={styles.actions}>
          <Button variant="ghost" onClick={onClose}>Cancel</Button>
          <Button variant="primary" onClick={handleExport} disabled={exporting} data-testid="export-download-btn">
            {exporting ? 'Exporting...' : 'Export'}
          </Button>
        </div>
      </div>
    </Modal>
  );
}
