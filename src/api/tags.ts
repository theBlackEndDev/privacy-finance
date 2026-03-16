import { invoke } from '@tauri-apps/api/core';
import { CreateTagInput, Tag, TagBrief, Transaction } from '../types';

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

  getTransactionTags: (transactionId: string) =>
    invoke<TagBrief[]>('get_transaction_tags', { transactionId }),

  getTransactionsByTag: (tagId: string) =>
    invoke<Transaction[]>('get_transactions_by_tag', { tagId }),
};
