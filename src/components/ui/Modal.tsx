import { useEffect, useRef, useCallback } from 'react';
import styles from './Modal.module.css';

interface ModalProps {
  title: string;
  onClose: () => void;
  children: React.ReactNode;
}

let modalIdCounter = 0;

export function Modal({ title, onClose, children }: ModalProps) {
  const contentRef = useRef<HTMLDivElement>(null);
  const titleIdRef = useRef(`modal-title-${++modalIdCounter}`);

  const handleKeyDown = useCallback((e: KeyboardEvent) => {
    if (e.key === 'Escape') {
      onClose();
      return;
    }

    // Focus trap: Tab/Shift+Tab cycle within modal
    if (e.key === 'Tab') {
      const modal = contentRef.current;
      if (!modal) return;

      const focusable = modal.querySelectorAll<HTMLElement>(
        'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
      );
      if (focusable.length === 0) return;

      const first = focusable[0];
      const last = focusable[focusable.length - 1];

      if (e.shiftKey) {
        if (document.activeElement === first || document.activeElement === modal) {
          e.preventDefault();
          last.focus();
        }
      } else {
        if (document.activeElement === last || document.activeElement === modal) {
          e.preventDefault();
          first.focus();
        }
      }
    }
  }, [onClose]);

  useEffect(() => {
    const previous = document.activeElement as HTMLElement | null;
    contentRef.current?.focus();

    document.addEventListener('keydown', handleKeyDown);
    return () => {
      document.removeEventListener('keydown', handleKeyDown);
      previous?.focus();
    };
  }, [handleKeyDown]);

  return (
    <div className={styles.overlay} onClick={(e) => e.target === e.currentTarget && onClose()}>
      <div
        className={styles.content}
        ref={contentRef}
        tabIndex={-1}
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleIdRef.current}
      >
        <div className={styles.header}>
          <span id={titleIdRef.current} className={styles.title}>{title}</span>
          <button className={styles.closeBtn} onClick={onClose} aria-label="Close dialog">&times;</button>
        </div>
        <div className={styles.body}>{children}</div>
      </div>
    </div>
  );
}
