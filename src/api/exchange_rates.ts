import { invoke } from './invoke';
import { CreateExchangeRateInput, ExchangeRate } from '../types';

export const exchangeRatesApi = {
  create: (input: CreateExchangeRateInput) =>
    invoke<ExchangeRate>('create_rate', { input }),
  getAll: () =>
    invoke<ExchangeRate[]>('get_all_rates', {}, []),
  delete: (id: string) =>
    invoke<void>('delete_rate', { id }),
};
