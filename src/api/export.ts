import { invoke } from './invoke';
import { ExportFilters } from '../types';

export const exportApi = {
  exportCsv: (path: string, filters: ExportFilters) =>
    invoke<number>('export_transactions_csv', { path, filters }),
  exportJson: (path: string, filters: ExportFilters) =>
    invoke<number>('export_transactions_json', { path, filters }),
  exportAll: (path: string) =>
    invoke<void>('export_all_data_json', { path }),
};
