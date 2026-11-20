import { Modal } from './Modal';
import styles from './KeyboardShortcutHelp.module.css';

interface Props {
  onClose: () => void;
}

export const SHORTCUTS = [
  ['Ctrl+1', 'Dashboard'],
  ['Ctrl+2', 'Accounts'],
  ['Ctrl+3', 'Transactions'],
  ['Ctrl+4', 'Categories'],
  ['Ctrl+5', 'Import'],
  ['Ctrl+6', 'Recurring'],
  ['Ctrl+7', 'Budgets'],
  ['Ctrl+8', 'Rules'],
  ['Ctrl+9', 'Reports'],
  ['Ctrl+B', 'Budgets (alt)'],
  ['Ctrl+N', 'Focus new transaction'],
  ['Ctrl+F', 'Focus search'],
  ['Ctrl+?', 'This help dialog'],
  ['Escape', 'Close modal / blur'],
];

export function KeyboardShortcutHelp({ onClose }: Props) {
  return (
    <Modal title="Keyboard Shortcuts" onClose={onClose}>
      <div className={styles.list}>
        {SHORTCUTS.map(([key, label]) => (
          <div key={key} className={styles.row}>
            <span className={styles.label}>{label}</span>
            <kbd className={styles.key}>{key}</kbd>
          </div>
        ))}
      </div>
    </Modal>
  );
}
