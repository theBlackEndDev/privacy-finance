import { useState, useEffect, useCallback } from 'react';
import { BudgetCategoryRow, BudgetSummary, BudgetWithItems } from '../types';
import { budgetApi } from '../api/budgets';
import { useToast } from '../components/ui/Toast';
import { Spinner } from '../components/ui/Spinner';
import { BudgetMonthNav } from '../components/budgets/BudgetMonthNav';
import { BudgetGrid } from '../components/budgets/BudgetGrid';

function currentYearMonth(): [number, number] {
  const d = new Date();
  return [d.getFullYear(), d.getMonth() + 1];
}

export function BudgetsPage() {
  const [initYear, initMonth] = currentYearMonth();
  const [year, setYear] = useState(initYear);
  const [month, setMonth] = useState(initMonth);
  const [items, setItems] = useState<BudgetCategoryRow[]>([]);
  const [summary, setSummary] = useState<BudgetSummary | null>(null);
  const [loading, setLoading] = useState(true);
  const toast = useToast();

  const loadBudget = useCallback(async (y: number, m: number) => {
    setLoading(true);
    try {
      let data: BudgetWithItems | null = await budgetApi.get(y, m);

      // Auto-create budget if none exists for this month
      if (!data) {
        await budgetApi.create(y, m);
        data = await budgetApi.get(y, m);
      }

      setItems(data?.items ?? []);

      const sum = await budgetApi.getSummary(y, m);
      setSummary(sum);
    } catch (err) {
      toast(`Failed to load budget: ${err}`, 'error');
    } finally {
      setLoading(false);
    }
  }, [toast]);

  useEffect(() => {
    loadBudget(year, month);
  }, [year, month, loadBudget]);

  function handleNavigate(y: number, m: number) {
    setYear(y);
    setMonth(m);
  }

  async function handleUpdateBudgeted(itemId: string, cents: number) {
    try {
      await budgetApi.updateItem(itemId, { budgeted: cents });
      await loadBudget(year, month);
    } catch (err) {
      toast(`Failed to update: ${err}`, 'error');
    }
  }

  async function handleToggleRollover(itemId: string, rollover: boolean) {
    try {
      await budgetApi.updateItem(itemId, { rollover });
      await loadBudget(year, month);
    } catch (err) {
      toast(`Failed to toggle rollover: ${err}`, 'error');
    }
  }

  async function handleCopy() {
    try {
      await budgetApi.copy(year, month);
      await loadBudget(year, month);
      toast('Copied from last month');
    } catch (err) {
      toast(`Failed to copy: ${err}`, 'error');
    }
  }

  async function handleRollover() {
    try {
      const count = await budgetApi.calculateRollover(year, month);
      await loadBudget(year, month);
      toast(`Rollover applied to ${count} categories`);
    } catch (err) {
      toast(`Failed to apply rollover: ${err}`, 'error');
    }
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 0 }}>
      <BudgetMonthNav
        year={year}
        month={month}
        onNavigate={handleNavigate}
        onCopy={handleCopy}
        onRollover={handleRollover}
      />

      {loading ? (
        <div style={{ display: 'flex', justifyContent: 'center', paddingTop: 32 }}>
          <Spinner />
        </div>
      ) : (
        <div style={{ padding: 16 }}>
          <BudgetGrid
            items={items}
            summary={summary}
            onUpdateBudgeted={handleUpdateBudgeted}
            onToggleRollover={handleToggleRollover}
          />
        </div>
      )}
    </div>
  );
}
