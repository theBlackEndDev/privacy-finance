import { invoke } from '@tauri-apps/api/core';
import { Category, CreateCategoryInput, UpdateCategoryInput } from '../types';

export const categoriesApi = {
  getAll: () => invoke<Category[]>('get_categories'),
  create: (input: CreateCategoryInput) => invoke<Category>('create_category', { input }),
  update: (id: string, input: UpdateCategoryInput) => invoke<Category>('update_category', { id, input }),
  delete: (id: string) => invoke<void>('delete_category', { id }),
};
