import { Button } from '../ui/Button';
import styles from './BudgetMonthNav.module.css';

const MONTH_NAMES = [
  'January', 'February', 'March', 'April', 'May', 'June',
  'July', 'August', 'September', 'October', 'November', 'December',
];

interface Props {
  year: number;
  month: number;
  onNavigate: (year: number, month: number) => void;
  onCopy: () => void;
  onRollover: () => void;
  copyDisabled?: boolean;
  onUndo?: () => void;
  onRedo?: () => void;
  canUndo?: boolean;
  canRedo?: boolean;
}

export function BudgetMonthNav({ year, month, onNavigate, onCopy, onRollover, copyDisabled, onUndo, onRedo, canUndo, canRedo }: Props) {
  function goPrev() {
    if (month === 1) {
      onNavigate(year - 1, 12);
    } else {
      onNavigate(year, month - 1);
    }
  }

  function goNext() {
    if (month === 12) {
      onNavigate(year + 1, 1);
    } else {
      onNavigate(year, month + 1);
    }
  }

  return (
    <div className={styles.nav}>
      <div className={styles.actions}>
        <button className={styles.navBtn} onClick={goPrev}>◀</button>
      </div>
      <span className={styles.monthDisplay}>{MONTH_NAMES[month - 1]} {year}</span>
      <div className={styles.actions}>
        <button className={styles.navBtn} onClick={goNext}>▶</button>
        {onUndo && (
          <button
            className={styles.navBtn}
            onClick={onUndo}
            disabled={!canUndo}
            title="Undo (Ctrl+Z)"
            aria-label="Undo"
          >
            ↩
          </button>
        )}
        {onRedo && (
          <button
            className={styles.navBtn}
            onClick={onRedo}
            disabled={!canRedo}
            title="Redo (Ctrl+Shift+Z)"
            aria-label="Redo"
          >
            ↪
          </button>
        )}
        <Button variant="ghost" size="sm" onClick={onCopy} disabled={copyDisabled}>
          Copy Last Month
        </Button>
        <Button variant="ghost" size="sm" onClick={onRollover}>
          Apply Rollover
        </Button>
      </div>
    </div>
  );
}
