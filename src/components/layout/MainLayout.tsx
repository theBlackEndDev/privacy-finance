import { ReactNode } from 'react';
import { Sidebar } from './Sidebar';
import { Header } from './Header';
import styles from './MainLayout.module.css';

export function MainLayout({ children }: { children: ReactNode }) {
  return (
    <div className={styles.layout}>
      <Sidebar />
      <div className={styles.main}>
        <Header />
        <main className={styles.content}>
          {children}
        </main>
      </div>
    </div>
  );
}
