import { invoke } from './invoke';
import {
  CsvPreview, ImportInput, ImportResult,
  ImportProfile, ColumnMapping,
} from '../types';

export const importApi = {
  previewCsv: (filePath: string) =>
    invoke<CsvPreview>('preview_csv', { filePath }),
  importCsv: (input: ImportInput) =>
    invoke<ImportResult>('import_csv', { input }),
  rollbackImport: (batchId: string) =>
    invoke<void>('rollback_import', { batchId }),
  getImportProfiles: () =>
    invoke<ImportProfile[]>('get_import_profiles', {}, []),
  saveImportProfile: (name: string, mappings: ColumnMapping[]) =>
    invoke<ImportProfile>('save_import_profile', { name, mappings }),
};
