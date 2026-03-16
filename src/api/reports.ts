import { invoke } from './invoke';
import {
  IncomeExpenseReport,
  CategorySpendingRow,
  CashFlowPoint,
  NetWorthPoint,
  TagSpendingRow,
  PayeeSpendingRow,
} from '../types';

const emptyIE: IncomeExpenseReport = { months: [], previousIncome: 0, previousExpense: 0 };

export const reportsApi = {
  getIncomeExpense: (startDate: string, endDate: string) =>
    invoke<IncomeExpenseReport>('get_income_expense_report', { range: { startDate, endDate } }, emptyIE),
  getCategorySpending: (startDate: string, endDate: string) =>
    invoke<CategorySpendingRow[]>('get_category_spending_report', { range: { startDate, endDate } }, []),
  getCashFlow: (startDate: string, endDate: string) =>
    invoke<CashFlowPoint[]>('get_cash_flow_report', { range: { startDate, endDate } }, []),
  getNetWorth: (startDate: string, endDate: string) =>
    invoke<NetWorthPoint[]>('get_net_worth_report', { range: { startDate, endDate } }, []),
  getTagSpending: (startDate: string, endDate: string) =>
    invoke<TagSpendingRow[]>('get_tag_spending_report', { range: { startDate, endDate } }, []),
  getPayeeSpending: (startDate: string, endDate: string, limit?: number) =>
    invoke<PayeeSpendingRow[]>('get_payee_spending_report', { range: { startDate, endDate }, limit }, []),
};
