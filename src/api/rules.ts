import { invoke } from './invoke';
import {
  CategorizationRule,
  CreateRuleInput,
  UpdateRuleInput,
  RuleTestResult,
} from '../types';

export const rulesApi = {
  getAll: () => invoke<CategorizationRule[]>('get_all_rules', {}, []),
  create: (input: CreateRuleInput) => invoke<CategorizationRule>('create_rule', { input }),
  update: (id: string, input: UpdateRuleInput) => invoke<CategorizationRule>('update_rule', { id, input }),
  delete: (id: string) => invoke<void>('delete_rule', { id }),
  reorder: (ids: string[]) => invoke<void>('reorder_rules', { ids }),
  matchTransaction: (payee: string, description: string, amount: number) =>
    invoke<string | null>('match_transaction', { payee, description, amount }),
  applyToUncategorized: () => invoke<number>('apply_rules_to_uncategorized', {}),
  testRule: (matchField: string, matchType: string, pattern: string) =>
    invoke<RuleTestResult[]>('test_rule', { matchField, matchType, pattern }),
};
