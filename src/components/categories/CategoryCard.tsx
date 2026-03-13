import { Category } from '../../types';
import { Button } from '../ui/Button';
import { cx } from '../../utils/classNames';
import styles from './CategoryCard.module.css';

interface CategoryCardProps {
  category: Category;
  onEdit: (c: Category) => void;
  onDelete: (c: Category) => void;
}

export function CategoryCard({ category, onEdit, onDelete }: CategoryCardProps) {
  return (
    <div className={cx(styles.card, category.isArchived && styles.archived)}>
      <div className={styles.dot} style={{ background: category.color }} />
      <span className={styles.name}>{category.name}</span>
      <div className={styles.actions}>
        <Button variant="ghost" size="sm" onClick={() => onEdit(category)}>Edit</Button>
        <Button variant="danger" size="sm" onClick={() => onDelete(category)}>Delete</Button>
      </div>
    </div>
  );
}
