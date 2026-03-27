import { useState, useRef, useEffect, useCallback } from 'react';
import { searchApi } from '../../api/search';
import { SearchResult } from '../../types';
import { useApp } from '../../context/AppContext';
import { useDebounce } from '../../hooks/useDebounce';
import { useAISearch } from '../../hooks/useAISearch';
import { isQuestion } from '../../utils/isQuestion';
import { formatCurrency } from '../../utils/formatCurrency';
import { AISearchResults } from './AISearchResults';
import styles from './GlobalSearch.module.css';

const AMOUNT_PREFIX: Record<string, string> = {
  income: '+',
  expense: '-',
  transfer: '',
};

function sanitizeSnippet(html: string): string {
  const div = document.createElement('div');
  div.textContent = html;
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
  const [mode, setMode] = useState<'keyword' | 'ai'>('keyword');
  const inputRef = useRef<HTMLInputElement>(null);
  const { result: aiResult, loading: aiLoading, search: aiDoSearch, clear: aiClear } = useAISearch();

  const debouncedQuery = useDebounce(query, 250);

  const doSearch = useCallback(async (q: string) => {
    if (q.trim().length === 0) {
      setResults([]);
      setIsOpen(false);
      aiClear();
      setMode('keyword');
      return;
    }

    if (isQuestion(q)) {
      setMode('ai');
      setResults([]);
      aiClear(); // Clear previous AI result before new search
      aiDoSearch(q);
      setIsOpen(true);
    } else {
      setMode('keyword');
      aiClear();
      try {
        const data = await searchApi.searchFull(q, 10);
        setResults(data);
        setIsOpen(data.length > 0);
        setActiveIndex(-1);
      } catch {
        setResults([]);
      }
    }
  }, [aiDoSearch, aiClear]);

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
    if (e.key === 'Escape') {
      e.preventDefault();
      setIsOpen(false);
      inputRef.current?.blur();
      return;
    }

    // Only handle arrow/enter for keyword mode
    if (mode !== 'keyword' || !isOpen || results.length === 0) return;

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
    }
  }

  useEffect(() => {
    function handleFocusSearch() {
      inputRef.current?.focus();
    }
    window.addEventListener('focus-search', handleFocusSearch);
    return () => window.removeEventListener('focus-search', handleFocusSearch);
  }, []);

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

  // Keep dropdown open when AI results arrive
  useEffect(() => {
    if (mode === 'ai' && aiResult) {
      setIsOpen(true);
    }
  }, [mode, aiResult]);

  const activeOptionId = activeIndex >= 0 ? `search-result-${activeIndex}` : undefined;
  const showAI = mode === 'ai' && (aiLoading || aiResult);
  const showKeyword = mode === 'keyword' && results.length > 0;

  return (
    <div className={styles.wrapper}>
      <input
        ref={inputRef}
        type="text"
        className={styles.input}
        placeholder="Search or ask a question… (Ctrl+F)"
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        onKeyDown={handleKeyDown}
        onFocus={() => {
          if (results.length > 0 || aiResult) setIsOpen(true);
        }}
        role="combobox"
        aria-expanded={isOpen}
        aria-controls={LISTBOX_ID}
        aria-activedescendant={activeOptionId}
        aria-label="Search or ask a question"
        aria-autocomplete="list"
      />
      {isOpen && (
        <div className={`${styles.dropdown} ${showAI ? styles.dropdownWide : ''}`} id={LISTBOX_ID}>
          {showAI && aiResult && (
            <AISearchResults result={aiResult} loading={aiLoading} />
          )}
          {showAI && aiLoading && !aiResult && (
            <div className={styles.empty}>Searching...</div>
          )}
          {showKeyword && (
            <div role="listbox" aria-label="Search results">
              {results.map((r, i) => (
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
              ))}
            </div>
          )}
          {mode === 'keyword' && results.length === 0 && query.trim().length > 0 && !aiLoading && (
            <div className={styles.empty}>No results found</div>
          )}
        </div>
      )}
    </div>
  );
}
