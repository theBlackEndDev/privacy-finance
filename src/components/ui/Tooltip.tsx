import { useId } from 'react';
import styles from './Tooltip.module.css';

interface TooltipProps {
  text: string;
  align?: 'center' | 'right';
}

export function Tooltip({ text, align = 'center' }: TooltipProps) {
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
      <span id={id} role="tooltip" className={align === 'right' ? styles.contentRight : styles.content}>
        {text}
      </span>
    </span>
  );
}
