import { Category } from '../../types';
import { Button } from '../ui/Button';
import { EmptyState } from '../ui/EmptyState';
import { CategoryCard } from './CategoryCard';
import styles from './CategoryList.module.css';

interface CategoryListProps {
  categories: Category[];
  onAdd: () => void;
  onEdit: (category: Category) => void;
  onDelete: (category: Category) => void;
}

export function CategoryList({ categories, onAdd, onEdit, onDelete }: CategoryListProps) {
  const income = categories.filter((c) => c.type === 'income' && !c.isArchived);
  const expense = categories.filter((c) => c.type === 'expense' && !c.isArchived);

  return (
    <div>
      <div className={styles.header}>
        <span className={styles.title}>Categories</span>
        <Button onClick={onAdd} size="sm">+ Add Category</Button>
      </div>

      {categories.length === 0 ? (
        <EmptyState title="No categories yet" action={<Button onClick={onAdd}>Add Category</Button>} />
      ) : (
        <>
          {income.length > 0 && (
            <div className={styles.section}>
              <div className={styles.sectionTitle}>Income</div>
              <div className={styles.grid}>
                {income.map((c) => <CategoryCard key={c.id} category={c} onEdit={onEdit} onDelete={onDelete} />)}
              </div>
            </div>
          )}
          {expense.length > 0 && (
            <div className={styles.section}>
              <div className={styles.sectionTitle}>Expense</div>
              <div className={styles.grid}>
                {expense.map((c) => <CategoryCard key={c.id} category={c} onEdit={onEdit} onDelete={onDelete} />)}
              </div>
            </div>
          )}
        </>
      )}
    </div>
  );
}
