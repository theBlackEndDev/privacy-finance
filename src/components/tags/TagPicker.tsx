import { useState, useRef, useEffect } from 'react';
import { Tag, TagBrief } from '../../types';
import { tagsApi } from '../../api/tags';
import { TagBadge } from './TagBadge';
import { useToast } from '../ui/Toast';
import styles from './TagPicker.module.css';

interface TagPickerProps {
  selectedTags: TagBrief[];
  onTagsChange: (tags: TagBrief[]) => void;
  transactionId?: string;
}

export function TagPicker({ selectedTags, onTagsChange, transactionId }: TagPickerProps) {
  const [open, setOpen] = useState(false);
  const [allTags, setAllTags] = useState<Tag[]>([]);
  const [search, setSearch] = useState('');
  const wrapperRef = useRef<HTMLDivElement>(null);
  const toast = useToast();

  useEffect(() => {
    tagsApi.getAll().then(setAllTags).catch(() => {});
  }, []);

  useEffect(() => {
    function handleClickOutside(e: MouseEvent) {
      if (wrapperRef.current && !wrapperRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    }
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  const selectedIds = new Set(selectedTags.map((t) => t.id));

  const filtered = allTags.filter(
    (t) => t.name.toLowerCase().includes(search.toLowerCase())
  );

  const canCreate = search.trim() !== '' &&
    !allTags.some((t) => t.name.toLowerCase() === search.trim().toLowerCase());

  async function toggleTag(tag: Tag) {
    try {
      if (selectedIds.has(tag.id)) {
        if (transactionId) {
          await tagsApi.untagTransaction(transactionId, tag.id);
        }
        onTagsChange(selectedTags.filter((t) => t.id !== tag.id));
      } else {
        if (transactionId) {
          await tagsApi.tagTransaction(transactionId, tag.id);
        }
        onTagsChange([...selectedTags, { id: tag.id, name: tag.name, color: tag.color }]);
      }
    } catch (err) {
      toast(String(err), 'error');
    }
  }

  async function handleCreate() {
    try {
      const newTag = await tagsApi.create({ name: search.trim() });
      setAllTags((prev) => [...prev, newTag].sort((a, b) => a.name.localeCompare(b.name)));
      if (transactionId) {
        await tagsApi.tagTransaction(transactionId, newTag.id);
      }
      onTagsChange([...selectedTags, { id: newTag.id, name: newTag.name, color: newTag.color }]);
      setSearch('');
    } catch (err) {
      toast(String(err), 'error');
    }
  }

  async function handleRemove(tagId: string) {
    try {
      if (transactionId) {
        await tagsApi.untagTransaction(transactionId, tagId);
      }
      onTagsChange(selectedTags.filter((t) => t.id !== tagId));
    } catch (err) {
      toast(String(err), 'error');
    }
  }

  return (
    <div className={styles.wrapper} ref={wrapperRef} data-testid="tag-picker">
      <span className={styles.label}>Tags</span>

      {selectedTags.length > 0 && (
        <div className={styles.selectedTags}>
          {selectedTags.map((tag) => (
            <TagBadge key={tag.id} tag={tag} onRemove={() => handleRemove(tag.id)} />
          ))}
        </div>
      )}

      <div className={styles.dropdown}>
        <button type="button" className={styles.trigger} onClick={() => setOpen(!open)}>
          + Add tag
        </button>

        {open && (
          <div className={styles.menu}>
            <input
              className={styles.searchInput}
              placeholder="Search or create tag..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              autoFocus
              aria-label="Search tags"
              data-testid="tag-input"
            />

            <div role="listbox" aria-multiselectable="true" aria-label="Available tags">
            {filtered.map((tag) => (
              <div key={tag.id} className={styles.option} onClick={() => toggleTag(tag)} role="option" aria-selected={selectedIds.has(tag.id)}>
                <input
                  type="checkbox"
                  className={styles.checkbox}
                  checked={selectedIds.has(tag.id)}
                  readOnly
                  tabIndex={-1}
                  aria-hidden="true"
                />
                {tag.color && <span className={styles.colorDot} style={{ background: tag.color }} />}
                {tag.name}
              </div>
            ))}
            </div>

            {filtered.length === 0 && !canCreate && (
              <div className={styles.empty}>No tags found</div>
            )}

            {canCreate && (
              <div className={styles.createOption} onClick={handleCreate} data-testid="tag-create-btn">
                + Create &ldquo;{search.trim()}&rdquo;
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
