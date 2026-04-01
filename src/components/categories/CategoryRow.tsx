import { Category } from '../../types';
import { Button } from '../ui/Button';
import { cx } from '../../utils/classNames';
import styles from './CategoryRow.module.css';

interface CategoryRowProps {
  category: Category;
  onEdit: (c: Category) => void;
  onDelete: (c: Category) => void;
}

export function CategoryRow({ category, onEdit, onDelete }: CategoryRowProps) {
  return (
    <div className={cx(styles.row, category.isArchived && styles.archived)} data-testid={`cat-row-${category.id}`}>
      <span className={styles.dot} style={{ background: category.color }} />
      <span className={styles.name}>{category.name}</span>
      <span className={styles.type}>{category.type}</span>
      <div className={styles.actions}>
        <Button variant="ghost" size="sm" onClick={() => onEdit(category)}>Edit</Button>
        <Button variant="danger" size="sm" onClick={() => onDelete(category)}>Delete</Button>
      </div>
    </div>
  );
}
