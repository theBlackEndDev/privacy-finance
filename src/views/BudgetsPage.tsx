import { useState, useEffect, useCallback } from 'react';
import { BudgetCategoryRow, BudgetSummary } from '../types';
import { budgetApi } from '../api/budgets';
import { useToast } from '../components/ui/Toast';
import { Spinner } from '../components/ui/Spinner';
import { BudgetMonthNav } from '../components/budgets/BudgetMonthNav';
import { BudgetGrid } from '../components/budgets/BudgetGrid';
import { BudgetDetailPanel } from '../components/budgets/BudgetDetailPanel';
import { useBudgetHistory, BudgetField } from '../hooks/useBudgetHistory';
import { UpdateBudgetItemInput } from '../types';

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
  const [selectedRow, setSelectedRow] = useState<BudgetCategoryRow | null>(null);
  const toast = useToast();

  const loadBudget = useCallback(async (y: number, m: number) => {
    setLoading(true);
    try {
      let [data, sum] = await Promise.all([
        budgetApi.get(y, m),
        budgetApi.getSummary(y, m),
      ]);

      // Auto-create budget if none exists for this month
      if (!data) {
        await budgetApi.create(y, m);
        [data, sum] = await Promise.all([
          budgetApi.get(y, m),
          budgetApi.getSummary(y, m),
        ]);
      }

      const newItems = data?.items ?? [];
      setItems(newItems);
      setSummary(sum);
      setSelectedRow(prev =>
        prev ? newItems.find(i => i.itemId === prev.itemId) ?? null : null
      );
    } catch (err) {
      toast(`Failed to load budget: ${err}`, 'error');
    } finally {
      setLoading(false);
    }
  }, [toast]);

  const reloadCurrent = useCallback(() => {
    loadBudget(year, month);
  }, [year, month, loadBudget]);

  const history = useBudgetHistory({ onApplied: reloadCurrent, toast });

  useEffect(() => {
    loadBudget(year, month);
  }, [year, month, loadBudget]);

  // Clear history when navigating months
  function handleNavigate(y: number, m: number) {
    history.clear();
    setYear(y);
    setMonth(m);
  }

  // Shared handler for undoable budget item updates
  async function handleUndoableUpdate(
    itemId: string,
    field: BudgetField,
    newValue: number | boolean | null,
    payload: UpdateBudgetItemInput,
    errMsg: string,
  ) {
    const row = items.find(i => i.itemId === itemId);
    try {
      await budgetApi.updateItem(itemId, payload);
      // Only push to history after successful API call
      if (row) {
        const oldValue = field === 'targetAmount' ? (row.targetAmount ?? null) : row[field];
        history.push({ itemId, categoryName: row.categoryName, field, oldValue, newValue });
      }
      await loadBudget(year, month);
    } catch (err) {
      toast(`${errMsg}: ${err}`, 'error');
    }
  }

  function handleUpdateBudgeted(itemId: string, cents: number) {
    handleUndoableUpdate(itemId, 'budgeted', cents, { budgeted: cents }, 'Failed to update');
  }

  function handleToggleRollover(itemId: string, rollover: boolean) {
    handleUndoableUpdate(itemId, 'rollover', rollover, { rollover }, 'Failed to toggle rollover');
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

  function handleUpdateTarget(itemId: string, targetAmount: number | null) {
    handleUndoableUpdate(itemId, 'targetAmount', targetAmount, { targetAmount }, 'Failed to update target');
  }

  function handleSelectCategory(row: BudgetCategoryRow) {
    setSelectedRow(row);
  }

  function handleClosePanel() {
    setSelectedRow(null);
  }

  // Keyboard shortcuts for undo/redo (scoped to budget page)
  useEffect(() => {
    function handler(e: KeyboardEvent) {
      if (!(e.ctrlKey || e.metaKey)) return;
      const tag = (e.target as HTMLElement).tagName;
      if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return;

      if (e.key === 'z' && !e.shiftKey) {
        e.preventDefault();
        history.undo();
      } else if (e.key === 'Z' && e.shiftKey) {
        e.preventDefault();
        history.redo();
      }
    }
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [history.undo, history.redo]);

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 0 }}>
      <BudgetMonthNav
        year={year}
        month={month}
        onNavigate={handleNavigate}
        onCopy={handleCopy}
        onRollover={handleRollover}
        onUndo={history.undo}
        onRedo={history.redo}
        canUndo={history.canUndo}
        canRedo={history.canRedo}
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
            onSelectCategory={handleSelectCategory}
          />
        </div>
      )}

      {selectedRow && (
        <BudgetDetailPanel
          row={selectedRow}
          year={year}
          month={month}
          onClose={handleClosePanel}
          onUpdateTarget={handleUpdateTarget}
        />
      )}
    </div>
  );
}
