import { Category } from '../../types';
import { Button } from '../ui/Button';
import { EmptyState } from '../ui/EmptyState';
import { CategoryCard } from './CategoryCard';
import { CategoryRow } from './CategoryRow';
import { useLocalStorage } from '../../hooks/useLocalStorage';
import { cx } from '../../utils/classNames';
import styles from './CategoryList.module.css';

type ViewMode = 'list' | 'grid';

interface CategoryListProps {
  categories: Category[];
  onAdd: () => void;
  onEdit: (category: Category) => void;
  onDelete: (category: Category) => void;
}

export function CategoryList({ categories, onAdd, onEdit, onDelete }: CategoryListProps) {
  const [viewMode, setViewMode] = useLocalStorage<ViewMode>('pf-category-view', 'list');

  const income = categories.filter((c) => c.type === 'income' && !c.isArchived);
  const expense = categories.filter((c) => c.type === 'expense' && !c.isArchived);

  function renderCategories(items: Category[]) {
    if (viewMode === 'grid') {
      return (
        <div className={styles.grid}>
          {items.map((c) => <CategoryCard key={c.id} category={c} onEdit={onEdit} onDelete={onDelete} />)}
        </div>
      );
    }
    return (
      <div className={styles.list}>
        {items.map((c) => <CategoryRow key={c.id} category={c} onEdit={onEdit} onDelete={onDelete} />)}
      </div>
    );
  }

  return (
    <div>
      <div className={styles.header}>
        <div className={styles.headerLeft}>
          <span className={styles.title}>Categories</span>
          <div className={styles.toggleGroup}>
            <button
              className={cx(styles.toggleBtn, viewMode === 'list' && styles.toggleBtnActive)}
              onClick={() => setViewMode('list')}
              aria-label="List view"
              title="List view"
            >
              ☰
            </button>
            <button
              className={cx(styles.toggleBtn, viewMode === 'grid' && styles.toggleBtnActive)}
              onClick={() => setViewMode('grid')}
              aria-label="Grid view"
              title="Grid view"
            >
              ▦
            </button>
          </div>
        </div>
        <Button onClick={onAdd} size="sm">+ Add Category</Button>
      </div>

      {categories.length === 0 ? (
        <EmptyState title="No categories yet" action={<Button onClick={onAdd}>Add Category</Button>} />
      ) : (
        <>
          {income.length > 0 && (
            <div className={styles.section}>
              <div className={styles.sectionTitle}>Income</div>
              {renderCategories(income)}
            </div>
          )}
          {expense.length > 0 && (
            <div className={styles.section}>
              <div className={styles.sectionTitle}>Expense</div>
              {renderCategories(expense)}
            </div>
          )}
        </>
      )}
    </div>
  );
}
