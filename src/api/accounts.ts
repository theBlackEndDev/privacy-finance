import { invoke } from './invoke';
import { Account, CreateAccountInput, UpdateAccountInput } from '../types';

export const accountsApi = {
  getAll: () => invoke<Account[]>('get_accounts', {}, []),
  getOne: (id: string) => invoke<Account>('get_account', { id }),
  create: (input: CreateAccountInput) => invoke<Account>('create_account', { input }),
  update: (id: string, input: UpdateAccountInput) => invoke<Account>('update_account', { id, input }),
  delete: (id: string) => invoke<void>('delete_account', { id }),
};
