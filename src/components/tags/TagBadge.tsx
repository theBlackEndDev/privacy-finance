import { TagBrief } from '../../types';
import styles from './TagBadge.module.css';

interface TagBadgeProps {
  tag: TagBrief;
  onRemove?: () => void;
}

export function TagBadge({ tag, onRemove }: TagBadgeProps) {
  return (
    <span className={styles.badge}>
      {tag.color && <span className={styles.dot} style={{ background: tag.color }} />}
      {tag.name}
      {onRemove && (
        <button className={styles.remove} onClick={onRemove} type="button" title="Remove tag">
          &times;
        </button>
      )}
    </span>
  );
}
