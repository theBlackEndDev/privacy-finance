import { useEffect, useRef } from 'react';
import styles from './Modal.module.css';

interface ModalProps {
  title: string;
  onClose: () => void;
  children: React.ReactNode;
}

export function Modal({ title, onClose, children }: ModalProps) {
  const contentRef = useRef<HTMLDivElement>(null);

  // Focus trap + Escape key
  useEffect(() => {
    const previous = document.activeElement as HTMLElement | null;
    contentRef.current?.focus();

    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    document.addEventListener('keydown', onKey);
    return () => {
      document.removeEventListener('keydown', onKey);
      previous?.focus();
    };
  }, [onClose]);

  return (
    <div className={styles.overlay} onClick={(e) => e.target === e.currentTarget && onClose()}>
      <div className={styles.content} ref={contentRef} tabIndex={-1} role="dialog" aria-modal aria-label={title}>
        <div className={styles.header}>
          <span className={styles.title}>{title}</span>
          <button className={styles.closeBtn} onClick={onClose} aria-label="Close">×</button>
        </div>
        <div className={styles.body}>{children}</div>
      </div>
    </div>
  );
}
