import { cx } from '../../utils/classNames';
import styles from './ColorPicker.module.css';

const PRESET_COLORS = [
  '#ef4444', '#f97316', '#eab308', '#22c55e', '#14b8a6',
  '#06b6d4', '#3b82f6', '#6366f1', '#8b5cf6', '#a855f7',
  '#ec4899', '#f43f5e', '#0ea5e9', '#6b7280', '#ffffff',
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
