import { invoke } from './invoke';
import { CategoryGroup } from '../types';

export const categoryGroupsApi = {
  list: () =>
    invoke<CategoryGroup[]>('list_category_groups', {}, []),
  create: (name: string, sortOrder?: number) =>
    invoke<CategoryGroup>('create_category_group', { name, sortOrder }),
};
