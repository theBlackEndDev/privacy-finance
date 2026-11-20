import { invoke } from './invoke';
import {
  Transaction, CreateTransactionInput, UpdateTransactionInput,
  TransactionFilters, TransactionPage, CreateTransferInput,
} from '../types';

const emptyPage: TransactionPage = { items: [], total: 0, page: 0, pageSize: 50 };

export const transactionsApi = {
  getAll: (filters: TransactionFilters) =>
    invoke<TransactionPage>('get_transactions', { filters }, emptyPage),
  getOne: (id: string) =>
    invoke<Transaction>('get_transaction', { id }),
  create: (input: CreateTransactionInput) =>
    invoke<Transaction>('create_transaction', { input }),
  update: (id: string, input: UpdateTransactionInput) =>
    invoke<Transaction>('update_transaction', { id, input }),
  delete: (id: string) =>
    invoke<void>('delete_transaction', { id }),
  getPayees: (search: string) =>
    invoke<string[]>('get_payees', { search }, []),
  createTransfer: (input: CreateTransferInput) =>
    invoke<[Transaction, Transaction]>('create_transfer', { input }),
};
