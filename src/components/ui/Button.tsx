import { ButtonHTMLAttributes } from 'react';
import { cx } from '../../utils/classNames';
import styles from './Button.module.css';

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'ghost' | 'danger';
  size?: 'sm' | 'md' | 'lg';
}

export function Button({ variant = 'primary', size = 'md', className, children, ...props }: ButtonProps) {
  return (
    <button
      className={cx(
        styles.btn,
        styles[variant],
        size !== 'md' && styles[size],
        className,
      )}
      {...props}
    >
      {children}
    </button>
  );
}
