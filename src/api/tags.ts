import { invoke } from './invoke';
import { CreateTagInput, Tag } from '../types';

export const tagsApi = {
  create: (input: CreateTagInput) =>
    invoke<Tag>('create_tag', { input }),

  getAll: () =>
    invoke<Tag[]>('get_all_tags'),

  update: (id: string, name?: string, color?: string) =>
    invoke<Tag>('update_tag', { id, name, color }),

  delete: (id: string) =>
    invoke<void>('delete_tag', { id }),

  tagTransaction: (transactionId: string, tagId: string) =>
    invoke<void>('tag_transaction', { transactionId, tagId }),

  untagTransaction: (transactionId: string, tagId: string) =>
    invoke<void>('untag_transaction', { transactionId, tagId }),
};
