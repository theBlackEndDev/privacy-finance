import { useState, useEffect } from 'react';
import { ExchangeRate } from '../../types';
import { exchangeRatesApi } from '../../api/exchange_rates';
import { todayISO } from '../../utils/formatDate';
import { Button } from '../ui/Button';
import styles from './ExchangeRateSettings.module.css';

export function ExchangeRateSettings() {
  const [rates, setRates] = useState<ExchangeRate[]>([]);
  const [fromCurrency, setFromCurrency] = useState('USD');
  const [toCurrency, setToCurrency] = useState('EUR');
  const [rate, setRate] = useState('');
  const [date, setDate] = useState(todayISO());
  const [error, setError] = useState('');

  useEffect(() => {
    exchangeRatesApi.getAll().then(setRates);
  }, []);

  async function handleAdd() {
    const rateNum = parseFloat(rate);
    if (!fromCurrency || !toCurrency || isNaN(rateNum) || rateNum <= 0) {
      setError('Please fill in all fields with valid values.');
      return;
    }
    if (fromCurrency === toCurrency) {
      setError('Currencies must be different.');
      return;
    }

    setError('');
    try {
      const created = await exchangeRatesApi.create({
        fromCurrency: fromCurrency.toUpperCase(),
        toCurrency: toCurrency.toUpperCase(),
        rate: rateNum,
        date,
      });
      setRates((prev) => [created, ...prev]);
      setRate('');
    } catch (err) {
      setError(String(err));
    }
  }

  async function handleDelete(id: string) {
    await exchangeRatesApi.delete(id);
    setRates((prev) => prev.filter((r) => r.id !== id));
  }

  return (
    <div className={styles.section}>
      <h3 className={styles.heading}>Exchange Rates</h3>
      <p className={styles.description}>
        Manually enter exchange rates for currency conversion.
      </p>

      {error && <div className={styles.error}>{error}</div>}

      <div className={styles.form}>
        <div className={styles.fieldGroup}>
          <label className={styles.label}>From</label>
          <input
            className={styles.input}
            value={fromCurrency}
            onChange={(e) => setFromCurrency(e.target.value)}
            maxLength={3}
            placeholder="USD"
          />
        </div>
        <div className={styles.fieldGroup}>
          <label className={styles.label}>To</label>
          <input
            className={styles.input}
            value={toCurrency}
            onChange={(e) => setToCurrency(e.target.value)}
            maxLength={3}
            placeholder="EUR"
          />
        </div>
        <div className={styles.fieldGroup}>
          <label className={styles.label}>Rate</label>
          <input
            className={styles.input}
            type="number"
            step="0.0001"
            value={rate}
            onChange={(e) => setRate(e.target.value)}
            placeholder="0.85"
          />
        </div>
        <div className={styles.fieldGroup}>
          <label className={styles.label}>Date</label>
          <input
            className={styles.input}
            type="date"
            value={date}
            onChange={(e) => setDate(e.target.value)}
          />
        </div>
        <Button variant="primary" size="sm" onClick={handleAdd}>
          Add Rate
        </Button>
      </div>

      {rates.length > 0 && (
        <div className={styles.rateList}>
          {rates.map((r) => (
            <div key={r.id} className={styles.rateRow}>
              <span className={styles.pair}>
                {r.fromCurrency}/{r.toCurrency}
              </span>
              <span className={styles.rateValue}>{r.rate.toFixed(4)}</span>
              <span className={styles.rateDate}>{r.date}</span>
              <button
                className={styles.deleteBtn}
                onClick={() => handleDelete(r.id)}
                title="Delete rate"
              >
                x
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
