import { Modal } from './Modal';
import styles from './AboutDialog.module.css';
import { version as APP_VERSION } from '../../../package.json';

interface Props {
  onClose: () => void;
}

export function AboutDialog({ onClose }: Props) {
  return (
    <Modal title="About PrivacyFinance" onClose={onClose}>
      <div className={styles.container}>
        <div className={styles.name}>PrivacyFinance</div>
        <div className={styles.version}>Version {APP_VERSION}</div>
        <div className={styles.tagline}>
          Privacy-first. Offline-only. No subscriptions.
        </div>
        <p className={styles.description}>
          A personal finance manager that keeps your data on your device.
          No cloud sync, no telemetry, no accounts required.
        </p>
      </div>
    </Modal>
  );
}
