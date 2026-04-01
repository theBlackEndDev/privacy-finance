import { ReactNode } from 'react';
import { Sidebar } from './Sidebar';
import { Header } from './Header';
import styles from './MainLayout.module.css';

export function MainLayout({ children }: { children: ReactNode }) {
  return (
    <div className={styles.layout}>
      <a href="#main-content" className={styles.skipLink}>Skip to content</a>
      <Sidebar />
      <div className={styles.main}>
        <Header />
        <main id="main-content" className={styles.content} data-testid="main-content">
          {children}
        </main>
      </div>
    </div>
  );
}
