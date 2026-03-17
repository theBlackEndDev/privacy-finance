import { useId } from 'react';
import styles from './Tooltip.module.css';

interface TooltipProps {
  text: string;
}

export function Tooltip({ text }: TooltipProps) {
  const id = useId();

  return (
    <span className={styles.wrapper}>
      <button
        type="button"
        className={styles.trigger}
        aria-describedby={id}
      >
        ?
      </button>
      <span id={id} role="tooltip" className={styles.content}>
        {text}
      </span>
    </span>
  );
}
