import { useState } from 'react';
import { formatDateISO } from '../../utils/formatDate';
import styles from './ReportDateRange.module.css';

interface Props {
  startDate: string;
  endDate: string;
  onChange: (start: string, end: string) => void;
}

type Preset = 'this-month' | 'last-month' | 'this-quarter' | 'this-year' | 'last-6-months' | 'custom';

function getPresetDates(preset: Preset): [string, string] {
  const now = new Date();
  const y = now.getFullYear();
  const m = now.getMonth(); // 0-indexed

  switch (preset) {
    case 'this-month': {
      const start = new Date(y, m, 1);
      const end = new Date(y, m + 1, 0);
      return [formatDateISO(start), formatDateISO(end)];
    }
    case 'last-month': {
      const start = new Date(y, m - 1, 1);
      const end = new Date(y, m, 0);
      return [formatDateISO(start), formatDateISO(end)];
    }
    case 'this-quarter': {
      const qStart = Math.floor(m / 3) * 3;
      const start = new Date(y, qStart, 1);
      const end = new Date(y, qStart + 3, 0);
      return [formatDateISO(start), formatDateISO(end)];
    }
    case 'this-year': {
      return [`${y}-01-01`, `${y}-12-31`];
    }
    case 'last-6-months': {
      const start = new Date(y, m - 5, 1);
      const end = new Date(y, m + 1, 0);
      return [formatDateISO(start), formatDateISO(end)];
    }
    default:
      return ['', ''];
  }
}

const PRESETS: { key: Preset; label: string }[] = [
  { key: 'this-month', label: 'This Month' },
  { key: 'last-month', label: 'Last Month' },
  { key: 'this-quarter', label: 'This Quarter' },
  { key: 'this-year', label: 'This Year' },
  { key: 'last-6-months', label: 'Last 6 Months' },
  { key: 'custom', label: 'Custom' },
];

export function ReportDateRange({ startDate, endDate, onChange }: Props) {
  const [activePreset, setActivePreset] = useState<Preset>('this-month');

  function handlePreset(preset: Preset) {
    setActivePreset(preset);
    if (preset !== 'custom') {
      const [s, e] = getPresetDates(preset);
      onChange(s, e);
    }
  }

  return (
    <div className={styles.wrapper}>
      <div className={styles.presets}>
        {PRESETS.map(({ key, label }) => (
          <button
            key={key}
            className={`${styles.presetBtn} ${activePreset === key ? styles.active : ''}`}
            onClick={() => handlePreset(key)}
          >
            {label}
          </button>
        ))}
      </div>
      {activePreset === 'custom' && (
        <div className={styles.custom}>
          <input
            type="date"
            className={styles.dateInput}
            value={startDate}
            onChange={(e) => onChange(e.target.value, endDate)}
          />
          <span className={styles.separator}>to</span>
          <input
            type="date"
            className={styles.dateInput}
            value={endDate}
            onChange={(e) => onChange(startDate, e.target.value)}
          />
        </div>
      )}
    </div>
  );
}
