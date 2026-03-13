import { InputHTMLAttributes, forwardRef } from 'react';
import { cx } from '../../utils/classNames';
import styles from './Input.module.css';

interface InputProps extends InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  error?: string;
}

export const Input = forwardRef<HTMLInputElement, InputProps>(
  ({ label, error, className, id, ...props }, ref) => {
    const inputId = id ?? label?.toLowerCase().replace(/\s+/g, '-');
    return (
      <div className={styles.wrapper}>
        {label && <label className={styles.label} htmlFor={inputId}>{label}</label>}
        <input
          ref={ref}
          id={inputId}
          className={cx(styles.input, error && styles.error, className)}
          {...props}
        />
        {error && <span className={styles.errorMsg}>{error}</span>}
      </div>
    );
  }
);
Input.displayName = 'Input';
