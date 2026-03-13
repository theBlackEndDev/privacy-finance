import { invoke } from '@tauri-apps/api/core';
import {
  Transaction, CreateTransactionInput, UpdateTransactionInput,
  TransactionFilters, TransactionPage, CreateTransferInput,
} from '../types';

export const transactionsApi = {
  getAll: (filters: TransactionFilters) =>
    invoke<TransactionPage>('get_transactions', { filters }),
  getOne: (id: string) =>
    invoke<Transaction>('get_transaction', { id }),
  create: (input: CreateTransactionInput) =>
    invoke<Transaction>('create_transaction', { input }),
  update: (id: string, input: UpdateTransactionInput) =>
    invoke<Transaction>('update_transaction', { id, input }),
  delete: (id: string) =>
    invoke<void>('delete_transaction', { id }),
  getPayees: (search: string) =>
    invoke<string[]>('get_payees', { search }),
  createTransfer: (input: CreateTransferInput) =>
    invoke<[Transaction, Transaction]>('create_transfer', { input }),
};
