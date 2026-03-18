import { useState, useRef, useEffect, useCallback } from 'react';
import { searchApi } from '../../api/search';
import { SearchResult } from '../../types';
import { useApp } from '../../context/AppContext';
import { useDebounce } from '../../hooks/useDebounce';
import { formatCurrency } from '../../utils/formatCurrency';
import styles from './GlobalSearch.module.css';

const AMOUNT_PREFIX: Record<string, string> = {
  income: '+',
  expense: '-',
  transfer: '',
};

function sanitizeSnippet(html: string): string {
  const div = document.createElement('div');
  div.textContent = html;
  // Re-inject only <mark> tags from FTS5 snippet()
  return div.innerHTML
    .replace(/&lt;mark&gt;/g, '<mark>')
    .replace(/&lt;\/mark&gt;/g, '</mark>');
}

const LISTBOX_ID = 'global-search-results';

export function GlobalSearch() {
  const { navigate, setPendingEditTransactionId } = useApp();
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<SearchResult[]>([]);
  const [isOpen, setIsOpen] = useState(false);
  const [activeIndex, setActiveIndex] = useState(-1);
  const inputRef = useRef<HTMLInputElement>(null);

  const debouncedQuery = useDebounce(query, 300);

  const doSearch = useCallback(async (q: string) => {
    if (q.trim().length === 0) {
      setResults([]);
      setIsOpen(false);
      return;
    }
    try {
      const data = await searchApi.searchFull(q, 10);
      setResults(data);
      setIsOpen(data.length > 0);
      setActiveIndex(-1);
    } catch {
      setResults([]);
    }
  }, []);

  useEffect(() => {
    doSearch(debouncedQuery);
  }, [debouncedQuery, doSearch]);

  function handleSelect(result: SearchResult) {
    setIsOpen(false);
    setQuery('');
    setResults([]);
    setPendingEditTransactionId(result.id);
    navigate('transactions');
  }

  function handleKeyDown(e: React.KeyboardEvent) {
    if (!isOpen || results.length === 0) {
      if (e.key === 'Escape') {
        inputRef.current?.blur();
        setIsOpen(false);
      }
      return;
    }

    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        setActiveIndex((prev) => (prev < results.length - 1 ? prev + 1 : 0));
        break;
      case 'ArrowUp':
        e.preventDefault();
        setActiveIndex((prev) => (prev > 0 ? prev - 1 : results.length - 1));
        break;
      case 'Enter':
        e.preventDefault();
        if (activeIndex >= 0 && activeIndex < results.length) {
          handleSelect(results[activeIndex]);
        }
        break;
      case 'Escape':
        e.preventDefault();
        setIsOpen(false);
        inputRef.current?.blur();
        break;
    }
  }

  // Listen for custom focus-search event (fired by Ctrl+F shortcut)
  useEffect(() => {
    function handleFocusSearch() {
      inputRef.current?.focus();
    }
    window.addEventListener('focus-search', handleFocusSearch);
    return () => window.removeEventListener('focus-search', handleFocusSearch);
  }, []);

  // Close dropdown when clicking outside
  useEffect(() => {
    function handleClickOutside(e: MouseEvent) {
      const target = e.target as HTMLElement;
      if (!target.closest(`.${styles.wrapper}`)) {
        setIsOpen(false);
      }
    }
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  const activeOptionId = activeIndex >= 0 ? `search-result-${activeIndex}` : undefined;

  return (
    <div className={styles.wrapper}>
      <input
        ref={inputRef}
        type="text"
        className={styles.input}
        placeholder="Search transactions… (Ctrl+F)"
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        onKeyDown={handleKeyDown}
        onFocus={() => { if (results.length > 0) setIsOpen(true); }}
        role="combobox"
        aria-expanded={isOpen}
        aria-controls={LISTBOX_ID}
        aria-activedescendant={activeOptionId}
        aria-label="Search transactions"
        aria-autocomplete="list"
      />
      {isOpen && (
        <div className={styles.dropdown} role="listbox" id={LISTBOX_ID} aria-label="Search results">
          {results.length === 0 ? (
            <div className={styles.empty}>No results found</div>
          ) : (
            results.map((r, i) => (
              <div
                key={r.id}
                id={`search-result-${i}`}
                role="option"
                aria-selected={i === activeIndex}
                className={`${styles.item} ${i === activeIndex ? styles.itemActive : ''}`}
                onClick={() => handleSelect(r)}
                onMouseEnter={() => setActiveIndex(i)}
              >
                <div className={styles.itemLeft}>
                  <span className={styles.payee}>{r.payee || 'No payee'}</span>
                  <span
                    className={styles.snippet}
                    dangerouslySetInnerHTML={{ __html: sanitizeSnippet(r.snippet) }}
                  />
                </div>
                <div className={styles.meta}>
                  <span className={styles.amount}>
                    {AMOUNT_PREFIX[r.kind]}{formatCurrency(r.amount)}
                  </span>
                  <span className={styles.date}>{r.date}</span>
                </div>
              </div>
            ))
          )}
        </div>
      )}
    </div>
  );
}
