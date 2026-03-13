import { createContext, useContext, useState, useEffect, useCallback, ReactNode } from 'react';
import { Category } from '../types';
import { categoriesApi } from '../api/categories';

interface CategoryContextValue {
  categories: Category[];
  loading: boolean;
  refresh: () => Promise<void>;
}

const CategoryContext = createContext<CategoryContextValue | null>(null);

export function CategoryProvider({ children }: { children: ReactNode }) {
  const [categories, setCategories] = useState<Category[]>([]);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    try {
      const data = await categoriesApi.getAll();
      setCategories(data);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { refresh(); }, [refresh]);

  return (
    <CategoryContext.Provider value={{ categories, loading, refresh }}>
      {children}
    </CategoryContext.Provider>
  );
}

export function useCategories() {
  const ctx = useContext(CategoryContext);
  if (!ctx) throw new Error('useCategories must be used within CategoryProvider');
  return ctx;
}
