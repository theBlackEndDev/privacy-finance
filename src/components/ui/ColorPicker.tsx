import { cx } from '../../utils/classNames';
import styles from './ColorPicker.module.css';

const PRESET_COLORS = [
  '#0d9488', '#0f766e', '#10b981', '#14b8a6', '#0ea5e9',
  '#3b82f6', '#64748b', '#f59e0b', '#f97316', '#ef4444',
  '#ec4899', '#8b5cf6', '#a855f7', '#6b7280', '#ffffff',
];

interface ColorPickerProps {
  value: string;
  onChange: (color: string) => void;
  label?: string;
}

export function ColorPicker({ value, onChange, label }: ColorPickerProps) {
  return (
    <div className={styles.wrapper}>
      {label && <span className={styles.label}>{label}</span>}
      <div className={styles.grid}>
        {PRESET_COLORS.map((color) => (
          <button
            key={color}
            type="button"
            className={cx(styles.swatch, value === color && styles.selected)}
            style={{ background: color }}
            onClick={() => onChange(color)}
            aria-label={color}
            title={color}
          />
        ))}
      </div>
      <div className={styles.custom}>
        <span className={styles.label}>Custom</span>
        <input
          type="color"
          value={value}
          onChange={(e) => onChange(e.target.value)}
          className={styles.colorInput}
        />
        <span className={styles.hex}>{value}</span>
      </div>
    </div>
  );
}
