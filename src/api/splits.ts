import { invoke } from '@tauri-apps/api/core';
import { CreateSplitInput, TransactionSplit } from '../types';

export const splitsApi = {
  create: (transactionId: string, splits: CreateSplitInput[]) =>
    invoke<TransactionSplit[]>('create_splits', { transactionId, splits }),

  getAll: (transactionId: string) =>
    invoke<TransactionSplit[]>('get_splits', { transactionId }),

  update: (transactionId: string, splits: CreateSplitInput[]) =>
    invoke<TransactionSplit[]>('update_splits', { transactionId, splits }),

  delete: (transactionId: string) =>
    invoke<void>('delete_splits', { transactionId }),
};
