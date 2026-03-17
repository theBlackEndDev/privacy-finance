import { useState, useEffect } from 'react';
import { Tag } from '../../types';
import { tagsApi } from '../../api/tags';
import { useToast } from '../ui/Toast';
import { useConfirmDelete } from '../../hooks/useConfirmDelete';
import { Button } from '../ui/Button';
import { ConfirmDialog } from '../ui/ConfirmDialog';
import styles from './TagManager.module.css';

export function TagManager() {
  const [tags, setTags] = useState<Tag[]>([]);
  const [newName, setNewName] = useState('');
  const [newColor, setNewColor] = useState('#6366f1');
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editName, setEditName] = useState('');
  const [editColor, setEditColor] = useState('');
  const toast = useToast();

  useEffect(() => {
    loadTags();
  }, []);

  async function loadTags() {
    try {
      const result = await tagsApi.getAll();
      setTags(result);
    } catch (err) {
      toast(String(err));
    }
  }

  async function handleCreate() {
    if (!newName.trim()) return;
    try {
      await tagsApi.create({ name: newName.trim(), color: newColor });
      setNewName('');
      setNewColor('#6366f1');
      await loadTags();
      toast('Tag created');
    } catch (err) {
      toast(String(err));
    }
  }

  async function handleUpdate(id: string) {
    try {
      await tagsApi.update(id, editName.trim() || undefined, editColor || undefined);
      setEditingId(null);
      await loadTags();
      toast('Tag updated');
    } catch (err) {
      toast(String(err));
    }
  }

  const deleteFlow = useConfirmDelete<{ id: string; name: string }>({
    onConfirm: async (tag) => {
      await tagsApi.delete(tag.id);
      await loadTags();
      toast('Tag deleted');
    },
  });

  function startEdit(tag: Tag) {
    setEditingId(tag.id);
    setEditName(tag.name);
    setEditColor(tag.color ?? '#6366f1');
  }

  return (
    <div className={styles.manager}>
      <div className={styles.header}>
        <span className={styles.title}>Manage Tags</span>
      </div>

      <div className={styles.createRow}>
        <input
          className={styles.nameInput}
          value={newName}
          onChange={(e) => setNewName(e.target.value)}
          placeholder="New tag name..."
          onKeyDown={(e) => e.key === 'Enter' && handleCreate()}
        />
        <input
          type="color"
          className={styles.colorInput}
          value={newColor}
          onChange={(e) => setNewColor(e.target.value)}
        />
        <Button size="sm" onClick={handleCreate} disabled={!newName.trim()}>
          Add
        </Button>
      </div>

      <div className={styles.list}>
        {tags.length === 0 && (
          <div className={styles.empty}>No tags yet. Create one above.</div>
        )}

        {tags.map((tag) => (
          <div key={tag.id} className={styles.tagRow}>
            {editingId === tag.id ? (
              <>
                <input
                  type="color"
                  className={styles.colorInput}
                  value={editColor}
                  onChange={(e) => setEditColor(e.target.value)}
                />
                <input
                  className={styles.editInput}
                  value={editName}
                  onChange={(e) => setEditName(e.target.value)}
                  onKeyDown={(e) => e.key === 'Enter' && handleUpdate(tag.id)}
                  autoFocus
                />
                <div className={styles.actions}>
                  <button className={styles.iconBtn} onClick={() => handleUpdate(tag.id)} title="Save">
                    &#10003;
                  </button>
                  <button className={styles.iconBtn} onClick={() => setEditingId(null)} title="Cancel">
                    &#10005;
                  </button>
                </div>
              </>
            ) : (
              <>
                <span
                  className={styles.tagDot}
                  style={{ background: tag.color ?? '#6366f1' }}
                />
                <span className={styles.tagName}>{tag.name}</span>
                <div className={styles.actions}>
                  <button className={styles.iconBtn} onClick={() => startEdit(tag)} title="Edit">
                    &#9998;
                  </button>
                  <button
                    className={`${styles.iconBtn} ${styles.deleteBtn}`}
                    onClick={() => deleteFlow.request({ id: tag.id, name: tag.name })}
                    title="Delete"
                  >
                    &times;
                  </button>
                </div>
              </>
            )}
          </div>
        ))}
      </div>
      {deleteFlow.pending && (
        <ConfirmDialog
          title="Delete Tag"
          message={`Delete tag "${deleteFlow.pending.name}"? This will untag all transactions.`}
          confirmLabel="Delete"
          onConfirm={deleteFlow.confirm}
          onCancel={deleteFlow.cancel}
        />
      )}
    </div>
  );
}
