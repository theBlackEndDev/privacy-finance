import { invoke } from './invoke';
import { EncryptionStatus } from '../types';

export const encryptionApi = {
  checkStatus: () =>
    invoke<EncryptionStatus>('check_encryption_status', {}, { isEncrypted: false }),

  enable: (password: string) =>
    invoke<void>('enable_encryption', { password }),

  unlock: (password: string) =>
    invoke<void>('unlock_database', { password }),

  changePassword: (currentPassword: string, newPassword: string) =>
    invoke<void>('change_password', { currentPassword, newPassword }),

  disable: (password: string) =>
    invoke<void>('disable_encryption', { password }),
};
