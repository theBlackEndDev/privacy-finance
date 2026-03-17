import { invoke } from './invoke';
import { CreateExchangeRateInput, ExchangeRate } from '../types';

export const exchangeRatesApi = {
  create: (input: CreateExchangeRateInput) =>
    invoke<ExchangeRate>('create_rate', { input }),
  getRate: (fromCurrency: string, toCurrency: string, date: string) =>
    invoke<ExchangeRate | null>('get_rate', { fromCurrency, toCurrency, date }, null),
  getAll: () =>
    invoke<ExchangeRate[]>('get_all_rates', {}, []),
  delete: (id: string) =>
    invoke<void>('delete_rate', { id }),
  convert: (amount: number, fromCurrency: string, toCurrency: string, date: string) =>
    invoke<number>('convert_amount', { amount, fromCurrency, toCurrency, date }, 0),
};
