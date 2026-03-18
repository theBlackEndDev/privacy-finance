import { useState, useCallback, useRef } from 'react';
import { budgetApi } from '../api/budgets';

const MAX_HISTORY = 50;

export type BudgetField = 'budgeted' | 'rollover' | 'targetAmount';

const FIELD_LABELS: Record<BudgetField, string> = {
  budgeted: 'budgeted amount',
  rollover: 'rollover',
  targetAmount: 'target',
};

export interface BudgetAction {
  itemId: string;
  categoryName: string;
  field: BudgetField;
  oldValue: number | boolean | null;
  newValue: number | boolean | null;
}

interface UseBudgetHistoryOptions {
  onApplied: () => void;
  toast: (msg: string, variant?: 'error' | 'success') => void;
}

async function applyValue(itemId: string, field: BudgetField, value: number | boolean | null) {
  switch (field) {
    case 'budgeted':
      await budgetApi.updateItem(itemId, { budgeted: value as number });
      break;
    case 'rollover':
      await budgetApi.updateItem(itemId, { rollover: value as boolean });
      break;
    case 'targetAmount':
      await budgetApi.updateItem(itemId, { targetAmount: value as number | null });
      break;
  }
}

export function useBudgetHistory({ onApplied, toast }: UseBudgetHistoryOptions) {
  const [undoStack, setUndoStack] = useState<BudgetAction[]>([]);
  const [redoStack, setRedoStack] = useState<BudgetAction[]>([]);
  const busyRef = useRef(false);

  const canUndo = undoStack.length > 0;
  const canRedo = redoStack.length > 0;

  const push = useCallback((action: BudgetAction) => {
    setUndoStack(prev => {
      const next = [...prev, action];
      return next.length > MAX_HISTORY ? next.slice(next.length - MAX_HISTORY) : next;
    });
    setRedoStack([]);
  }, []);

  const clear = useCallback(() => {
    setUndoStack([]);
    setRedoStack([]);
  }, []);

  const undo = useCallback(async () => {
    if (busyRef.current) return;
    const action = undoStack[undoStack.length - 1];
    if (!action) return;

    busyRef.current = true;
    setUndoStack(prev => prev.slice(0, -1));
    try {
      await applyValue(action.itemId, action.field, action.oldValue);
      setRedoStack(r => [...r, action]);
      toast(`Undo ${action.categoryName} ${FIELD_LABELS[action.field]}`);
      onApplied();
    } catch (err) {
      setUndoStack(s => [...s, action]);
      toast(`Undo failed: ${err}`, 'error');
    } finally {
      busyRef.current = false;
    }
  }, [undoStack, onApplied, toast]);

  const redo = useCallback(async () => {
    if (busyRef.current) return;
    const action = redoStack[redoStack.length - 1];
    if (!action) return;

    busyRef.current = true;
    setRedoStack(prev => prev.slice(0, -1));
    try {
      await applyValue(action.itemId, action.field, action.newValue);
      setUndoStack(u => [...u, action]);
      toast(`Redo ${action.categoryName} ${FIELD_LABELS[action.field]}`);
      onApplied();
    } catch (err) {
      setRedoStack(s => [...s, action]);
      toast(`Redo failed: ${err}`, 'error');
    } finally {
      busyRef.current = false;
    }
  }, [redoStack, onApplied, toast]);

  return { push, undo, redo, clear, canUndo, canRedo };
}
