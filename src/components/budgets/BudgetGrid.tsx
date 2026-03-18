import { useCallback, useMemo } from 'react';
import { BudgetCategoryRow as RowData, BudgetSummary } from '../../types';
import { formatCurrency } from '../../utils/formatCurrency';
import { cx } from '../../utils/classNames';
import { BudgetCategoryRow } from './BudgetCategoryRow';
import { Tooltip } from '../ui/Tooltip';
import { useLocalStorage } from '../../hooks/useLocalStorage';
import styles from './BudgetGrid.module.css';

const UNGROUPED = '__ungrouped__';

interface GroupData {
  groupId: string;
  groupName: string;
  items: RowData[];
  budgeted: number;
  spent: number;
  available: number;
}

function groupItems(items: RowData[]): GroupData[] {
  const map = new Map<string, GroupData>();

  for (const item of items) {
    const gid = item.groupId ?? UNGROUPED;
    const gname = item.groupName ?? 'Ungrouped';
    const group = map.get(gid);
    if (!group) {
      map.set(gid, { groupId: gid, groupName: gname, items: [item], budgeted: item.budgeted, spent: item.spent, available: item.available });
    } else {
      map.set(gid, {
        ...group,
        items: [...group.items, item],
        budgeted: group.budgeted + item.budgeted,
        spent: group.spent + item.spent,
        available: group.available + item.available,
      });
    }
  }

  // Sort: named groups first (by insertion order from sorted query), ungrouped last
  const values = [...map.values()];
  return [
    ...values.filter(g => g.groupId !== UNGROUPED),
    ...values.filter(g => g.groupId === UNGROUPED),
  ];
}

interface Props {
  items: RowData[];
  summary: BudgetSummary | null;
  onUpdateBudgeted: (itemId: string, cents: number) => void;
  onToggleRollover: (itemId: string, rollover: boolean) => void;
}

export function BudgetGrid({ items, summary, onUpdateBudgeted, onToggleRollover }: Props) {
  const toBeBudgeted = summary?.toBeBudgeted ?? 0;
  const tbbClass = toBeBudgeted > 0 ? styles.positive : toBeBudgeted < 0 ? styles.negative : styles.zero;

  const [collapsedArr, setCollapsedArr] = useLocalStorage<string[]>('budget-collapsed-groups', []);
  const collapsed = useMemo(() => new Set(collapsedArr), [collapsedArr]);

  const toggleGroup = useCallback((groupId: string) => {
    setCollapsedArr(
      collapsed.has(groupId)
        ? collapsedArr.filter(id => id !== groupId)
        : [...collapsedArr, groupId]
    );
  }, [collapsed, collapsedArr, setCollapsedArr]);

  const groups = useMemo(() => groupItems(items), [items]);
  const hasGroups = groups.some(g => g.groupId !== UNGROUPED);

  return (
    <div className={styles.container}>
      {/* To Be Budgeted banner */}
      <div className={styles.toBeBudgeted} aria-live="polite">
        <span className={styles.toBeBudgetedLabel}>
          To Be Budgeted{' '}
          <Tooltip text="Money you've received but haven't assigned to a category yet. Aim for zero — every dollar should have a job." />
        </span>
        <span className={cx(styles.toBeBudgetedAmount, tbbClass)}>
          {formatCurrency(toBeBudgeted)}
        </span>
      </div>

      <div role="table" aria-label="Budget grid">
        {/* Header */}
        <div role="row">
          <span className={styles.headerCell} role="columnheader">Category</span>
          <span className={styles.headerRight} role="columnheader">
            Budgeted <Tooltip text="How much you plan to spend in this category this month. Set this based on your priorities." />
          </span>
          <span className={styles.headerRight} role="columnheader">Spent</span>
          <span className={styles.headerRight} role="columnheader">
            Available <Tooltip text="What's left to spend: budgeted minus spent. Negative means you've overspent — consider moving money from another category." />
          </span>
          <span className={styles.headerCenter} role="columnheader" title="Rollover" aria-label="Rollover">
            {'\u21BB'} <Tooltip text="When enabled, unspent money carries forward to next month instead of resetting to zero." />
          </span>
        </div>

        {/* Rows */}
        {items.length === 0 ? (
          <div className={styles.empty}>No budget items. Create a budget to get started.</div>
        ) : hasGroups ? (
          groups.map((group) => {
            const isCollapsed = collapsed.has(group.groupId);
            return (
              <div key={group.groupId}>
                <div
                  className={styles.groupHeader}
                  role="row"
                  onClick={() => toggleGroup(group.groupId)}
                  aria-expanded={!isCollapsed}
                >
                  <span className={styles.groupHeaderName} role="cell">
                    <span className={isCollapsed ? styles.chevronCollapsed : styles.chevronExpanded}>▼</span>
                    {group.groupName}
                  </span>
                  <span className={styles.groupSubtotal} role="cell">{formatCurrency(group.budgeted)}</span>
                  <span className={styles.groupSubtotal} role="cell">{formatCurrency(group.spent)}</span>
                  <span className={styles.groupSubtotal} role="cell">{formatCurrency(group.available)}</span>
                  <span role="cell" />
                </div>
                {!isCollapsed && group.items.map((row) => (
                  <BudgetCategoryRow
                    key={row.itemId}
                    row={row}
                    onUpdateBudgeted={onUpdateBudgeted}
                    onToggleRollover={onToggleRollover}
                  />
                ))}
              </div>
            );
          })
        ) : (
          items.map((row) => (
            <BudgetCategoryRow
              key={row.itemId}
              row={row}
              onUpdateBudgeted={onUpdateBudgeted}
              onToggleRollover={onToggleRollover}
            />
          ))
        )}

        {/* Summary row */}
        {summary && (
          <div className={styles.summaryRow} role="row">
            <span className={styles.summaryLabel} role="cell">Totals</span>
            <span className={styles.summaryAmount} role="cell">{formatCurrency(summary.totalBudgeted)}</span>
            <span className={styles.summaryAmount} role="cell">{formatCurrency(summary.totalSpent)}</span>
            <span className={styles.summaryAmount} role="cell">{formatCurrency(summary.totalAvailable)}</span>
            <span role="cell" />
          </div>
        )}
      </div>
    </div>
  );
}
