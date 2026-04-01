import { useState } from 'react';
import { CategorizationRule, CreateRuleInput, MatchField, MatchType, Category } from '../../types';
import { Button } from '../ui/Button';
import styles from './RuleForm.module.css';

interface RuleFormProps {
  initial?: CategorizationRule;
  categories: Category[];
  onSubmit: (input: CreateRuleInput) => void;
  onCancel: () => void;
}

export function RuleForm({ initial, categories, onSubmit, onCancel }: RuleFormProps) {
  const [name, setName] = useState(initial?.name ?? '');
  const [matchField, setMatchField] = useState<MatchField>(initial?.matchField ?? 'payee');
  const [matchType, setMatchType] = useState<MatchType>(initial?.matchType ?? 'contains');
  const [pattern, setPattern] = useState(initial?.pattern ?? '');
  const [categoryId, setCategoryId] = useState(initial?.categoryId ?? '');

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!name.trim() || !pattern.trim() || !categoryId) return;
    onSubmit({ name: name.trim(), matchField, matchType, pattern: pattern.trim(), categoryId });
  }

  const expenseCategories = categories.filter((c) => !c.isArchived);

  return (
    <form className={styles.form} onSubmit={handleSubmit}>
      <div className={styles.field}>
        <label>Rule Name</label>
        <input
          type="text"
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="e.g. Netflix → Subscriptions"
          required
          data-testid="rule-name-input"
        />
      </div>

      <div className={styles.row}>
        <div className={styles.field}>
          <label>Match Field</label>
          <select value={matchField} onChange={(e) => setMatchField(e.target.value as MatchField)} data-testid="rule-field-select">
            <option value="payee">Payee</option>
            <option value="description">Description</option>
            <option value="amount">Amount</option>
          </select>
        </div>

        <div className={styles.field}>
          <label>Match Type</label>
          <select value={matchType} onChange={(e) => setMatchType(e.target.value as MatchType)} data-testid="rule-match-type-select">
            <option value="contains">Contains</option>
            <option value="starts_with">Starts With</option>
            <option value="ends_with">Ends With</option>
            <option value="exact">Exact Match</option>
            <option value="regex">Regex</option>
          </select>
        </div>
      </div>

      <div className={styles.field}>
        <label>Pattern</label>
        <input
          type="text"
          value={pattern}
          onChange={(e) => setPattern(e.target.value)}
          placeholder={matchType === 'regex' ? 'e.g. netflix\\s*\\d+' : 'e.g. NETFLIX'}
          required
          data-testid="rule-pattern-input"
        />
      </div>

      <div className={styles.field}>
        <label>Assign Category</label>
        <select value={categoryId} onChange={(e) => setCategoryId(e.target.value)} required data-testid="rule-category-select">
          <option value="">Select a category...</option>
          {expenseCategories.map((cat) => (
            <option key={cat.id} value={cat.id}>
              {cat.name}
            </option>
          ))}
        </select>
      </div>

      <div className={styles.actions}>
        <Button type="button" onClick={onCancel}>
          Cancel
        </Button>
        <Button type="submit" data-testid="rule-submit-btn">
          {initial ? 'Update Rule' : 'Create Rule'}
        </Button>
      </div>
    </form>
  );
}
