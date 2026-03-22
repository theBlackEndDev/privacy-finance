import { cx } from '../../utils/classNames';
import logoTeal from '../../assets/privacyfinance-logo-teal.png';
import styles from './BrandWordmark.module.css';

interface BrandWordmarkProps {
  className?: string;
  compact?: boolean;
  showTagline?: boolean;
}

export function BrandWordmark({ className, compact = false, showTagline = false }: BrandWordmarkProps) {
  return (
    <div className={cx(styles.brand, compact && styles.compact, className)}>
      <div className={styles.row}>
        <span className={styles.markShell} aria-hidden="true">
          <img className={styles.markImage} src={logoTeal} alt="PrivacyFinance logo" />
        </span>
        <div className={styles.copy}>
          <div className={styles.wordmark}>
            <span>Privacy</span><span className={styles.accent}>Finance</span>
          </div>
          {showTagline && <div className={styles.tagline}>Your finances. Your data. Your machine.</div>}
        </div>
      </div>
    </div>
  );
}
