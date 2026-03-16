import { invoke } from './invoke';
import { BackupInfo } from '../types';

export const backupApi = {
  createBackup: (path: string) =>
    invoke<void>('create_backup', { path }),
  restoreBackup: (path: string) =>
    invoke<void>('restore_backup', { path }),
  getBackupInfo: (path: string) =>
    invoke<BackupInfo>('get_backup_info', { path }),
};
