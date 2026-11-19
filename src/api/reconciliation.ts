import { invoke } from './invoke';
import {
  StartReconciliationInput,
  ReconciliationSummary,
  Reconciliation,
} from '../types';

const emptySummary: ReconciliationSummary = {
  reconciliation: { id: '', accountId: '', statementDate: '', statementBalance: 0, status: 'pending', createdAt: '' },
  items: [],
  clearedBalance: 0,
  statementBalance: 0,
  difference: 0,
};

export const reconciliationApi = {
  start: (input: StartReconciliationInput) =>
    invoke<ReconciliationSummary>('start_reconciliation', { input }, emptySummary),
  toggleItem: (reconciliationId: string, transactionId: string) =>
    invoke<ReconciliationSummary>('toggle_reconciliation_item', { reconciliationId, transactionId }, emptySummary),
  complete: (reconciliationId: string) =>
    invoke<Reconciliation>('complete_reconciliation', { reconciliationId }),
  cancel: (reconciliationId: string) =>
    invoke<void>('cancel_reconciliation', { reconciliationId }),
};
