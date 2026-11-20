import { useState } from 'react';
import { Category, CreateCategoryInput, UpdateCategoryInput } from '../types';
import { categoriesApi } from '../api/categories';
import { useCategories } from '../context/CategoryContext';
import { useConfirmDelete } from '../hooks/useConfirmDelete';
import { CategoryList } from '../components/categories/CategoryList';
import { CategoryForm } from '../components/categories/CategoryForm';
import { ConfirmDialog } from '../components/ui/ConfirmDialog';
import { Spinner } from '../components/ui/Spinner';
import { useToast } from '../components/ui/Toast';

export function CategoriesPage() {
  const { categories, loading, refresh: load } = useCategories();
  const [showForm, setShowForm] = useState(false);
  const [editing, setEditing] = useState<Category | undefined>();
  const toast = useToast();

  async function handleSave(input: CreateCategoryInput | UpdateCategoryInput) {
    if (editing) {
      await categoriesApi.update(editing.id, input as UpdateCategoryInput);
      toast('Category updated');
    } else {
      await categoriesApi.create(input as CreateCategoryInput);
      toast('Category created');
    }
    await load();
  }

  const deleteFlow = useConfirmDelete<Category>({
    onConfirm: async (category) => {
      try {
        await categoriesApi.delete(category.id);
        await load();
        toast('Category deleted');
      } catch (err) {
        toast(String(err), 'error');
      }
    },
  });

  function openAdd() { setEditing(undefined); setShowForm(true); }
  function openEdit(category: Category) { setEditing(category); setShowForm(true); }
  function closeForm() { setShowForm(false); setEditing(undefined); }

  if (loading) return <div style={{ display: 'flex', justifyContent: 'center', paddingTop: 48 }}><Spinner /></div>;

  return (
    <>
      <CategoryList categories={categories} onAdd={openAdd} onEdit={openEdit} onDelete={deleteFlow.request} />
      {showForm && <CategoryForm category={editing} onSave={handleSave} onClose={closeForm} />}
      {deleteFlow.pending && (
        <ConfirmDialog
          title="Delete Category"
          message={`Delete "${deleteFlow.pending.name}"? Transactions using it will become uncategorized.`}
          confirmLabel="Delete"
          onConfirm={deleteFlow.confirm}
          onCancel={deleteFlow.cancel}
        />
      )}
    </>
  );
}
