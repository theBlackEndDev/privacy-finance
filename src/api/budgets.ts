import { invoke } from './invoke';
import { Budget, BudgetWithItems, BudgetSummary, UpdateBudgetItemInput } from '../types';

export const budgetApi = {
  create: (year: number, month: number) =>
    invoke<Budget>('create_budget', { year, month }),
  get: (year: number, month: number) =>
    invoke<BudgetWithItems | null>('get_budget', { year, month }),
  updateItem: (itemId: string, input: UpdateBudgetItemInput) =>
    invoke<void>('update_budget_item', { itemId, input }),
  copy: (year: number, month: number) =>
    invoke<Budget>('copy_budget', { year, month }),
  getSummary: (year: number, month: number) =>
    invoke<BudgetSummary>('get_budget_summary', { year, month }),
  calculateRollover: (year: number, month: number) =>
    invoke<number>('calculate_rollover', { year, month }),
};
