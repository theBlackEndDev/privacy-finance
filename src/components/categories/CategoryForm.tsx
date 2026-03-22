import { useState, FormEvent } from 'react';
import { Category, CreateCategoryInput, UpdateCategoryInput } from '../../types';
import { Button } from '../ui/Button';
import { Input } from '../ui/Input';
import { Modal } from '../ui/Modal';
import { ColorPicker } from '../ui/ColorPicker';
import styles from './CategoryForm.module.css';

interface CategoryFormProps {
  category?: Category;
  onSave: (input: CreateCategoryInput | UpdateCategoryInput) => Promise<void>;
  onClose: () => void;
}

export function CategoryForm({ category, onSave, onClose }: CategoryFormProps) {
  const [name, setName] = useState(category?.name ?? '');
  const [type, setType] = useState<'income' | 'expense'>(category?.type ?? 'expense');
  const [color, setColor] = useState(category?.color ?? '#0d9488');
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState('');

  async function handleSubmit(e: FormEvent) {
    e.preventDefault();
    if (!name.trim()) { setError('Name is required'); return; }
    setSaving(true);
    try {
      await onSave({ name: name.trim(), type, color });
      onClose();
    } catch (err) {
      setError(String(err));
    } finally {
      setSaving(false);
    }
  }

  return (
    <Modal title={category ? 'Edit Category' : 'Add Category'} onClose={onClose}>
      <form className={styles.form} onSubmit={handleSubmit}>
        <Input
          label="Category Name"
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="e.g. Groceries"
          autoFocus
          error={error && !name.trim() ? error : undefined}
        />

        <div>
          <label className={styles.selectLabel}>Type</label>
          <select className={styles.select} value={type} onChange={(e) => setType(e.target.value as 'income' | 'expense')}>
            <option value="expense">Expense</option>
            <option value="income">Income</option>
          </select>
        </div>

        <ColorPicker label="Color" value={color} onChange={setColor} />

        {error && name.trim() && <p style={{ color: 'var(--color-expense)', fontSize: 13 }}>{error}</p>}

        <div className={styles.footer}>
          <Button variant="ghost" type="button" onClick={onClose}>Cancel</Button>
          <Button type="submit" disabled={saving}>{saving ? 'Saving…' : 'Save'}</Button>
        </div>
      </form>
    </Modal>
  );
}
