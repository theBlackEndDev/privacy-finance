import { useState, useEffect, useCallback } from 'react';
import { Category, CreateCategoryInput, UpdateCategoryInput } from '../types';
import { categoriesApi } from '../api/categories';
import { CategoryList } from '../components/categories/CategoryList';
import { CategoryForm } from '../components/categories/CategoryForm';
import { Spinner } from '../components/ui/Spinner';

export function CategoriesPage() {
  const [categories, setCategories] = useState<Category[]>([]);
  const [loading, setLoading] = useState(true);
  const [showForm, setShowForm] = useState(false);
  const [editing, setEditing] = useState<Category | undefined>();

  const load = useCallback(async () => {
    try {
      const data = await categoriesApi.getAll();
      setCategories(data);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { load(); }, [load]);

  async function handleSave(input: CreateCategoryInput | UpdateCategoryInput) {
    if (editing) {
      await categoriesApi.update(editing.id, input as UpdateCategoryInput);
    } else {
      await categoriesApi.create(input as CreateCategoryInput);
    }
    await load();
  }

  async function handleDelete(category: Category) {
    if (!confirm(`Delete "${category.name}"? Transactions using it will become uncategorized.`)) return;
    try {
      await categoriesApi.delete(category.id);
      await load();
    } catch (err) {
      alert(String(err));
    }
  }

  function openAdd() { setEditing(undefined); setShowForm(true); }
  function openEdit(category: Category) { setEditing(category); setShowForm(true); }
  function closeForm() { setShowForm(false); setEditing(undefined); }

  if (loading) return <div style={{ display: 'flex', justifyContent: 'center', paddingTop: 48 }}><Spinner /></div>;

  return (
    <>
      <CategoryList categories={categories} onAdd={openAdd} onEdit={openEdit} onDelete={handleDelete} />
      {showForm && <CategoryForm category={editing} onSave={handleSave} onClose={closeForm} />}
    </>
  );
}
