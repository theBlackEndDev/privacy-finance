import { invoke } from './invoke';

export const settingsApi = {
  get: (key: string) =>
    invoke<string | null>('get_setting', { key }, null),
  set: (key: string, value: string) =>
    invoke<void>('set_setting', { key, value }),
  deleteAllData: () =>
    invoke<void>('delete_all_data'),
};
