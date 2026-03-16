import { invoke } from './invoke';
import {
  RecurringTransaction,
  CreateRecurringInput,
  UpdateRecurringInput,
  DueRecurringItem,
  GenerateResult,
} from '../types';

export const recurringApi = {
  getAll: () => invoke<RecurringTransaction[]>('get_all_recurring', {}, []),
  getOne: (id: string) => invoke<RecurringTransaction>('get_recurring', { id }),
  create: (input: CreateRecurringInput) => invoke<RecurringTransaction>('create_recurring', { input }),
  update: (id: string, input: UpdateRecurringInput) => invoke<RecurringTransaction>('update_recurring', { id, input }),
  delete: (id: string) => invoke<void>('delete_recurring', { id }),
  toggleActive: (id: string) => invoke<RecurringTransaction>('toggle_recurring_active', { id }),
  checkDue: () => invoke<DueRecurringItem[]>('check_due_recurring', {}, []),
  generateAll: () => invoke<GenerateResult>('generate_recurring_transactions', {}),
};
