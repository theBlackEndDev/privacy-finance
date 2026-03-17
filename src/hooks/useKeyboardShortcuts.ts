import { useEffect } from 'react';
import { View } from '../types';

const VIEW_KEYS: Record<string, View> = {
  '1': 'dashboard',
  '2': 'accounts',
  '3': 'transactions',
  '4': 'categories',
  '5': 'import',
  '6': 'recurring',
  '7': 'budgets',
  '8': 'rules',
  '9': 'reports',
};

interface UseKeyboardShortcutsOptions {
  navigate: (view: View) => void;
  onFocusQuickEntry?: () => void;
  onFocusSearch?: () => void;
  onToggleShortcutHelp?: () => void;
}

export function useKeyboardShortcuts({ navigate, onFocusQuickEntry, onFocusSearch, onToggleShortcutHelp }: UseKeyboardShortcutsOptions) {
  useEffect(() => {
    function handler(e: KeyboardEvent) {
      const tag = (e.target as HTMLElement).tagName;
      const inInput = tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT';

      if (e.key === 'Escape') {
        // Let individual components handle Escape (modals, etc.)
        return;
      }

      if (e.ctrlKey || e.metaKey) {
        // Ctrl+1-5: navigate views
        if (VIEW_KEYS[e.key]) {
          e.preventDefault();
          navigate(VIEW_KEYS[e.key]);
          return;
        }
        // Ctrl+B: navigate to budgets
        if (e.key === 'b') {
          e.preventDefault();
          navigate('budgets');
          return;
        }
        // Ctrl+N: focus quick-entry
        if (e.key === 'n' && onFocusQuickEntry) {
          e.preventDefault();
          onFocusQuickEntry();
          return;
        }
        // Ctrl+? (Ctrl+Shift+/): keyboard shortcut help
        if (e.key === '?' && onToggleShortcutHelp) {
          e.preventDefault();
          onToggleShortcutHelp();
          return;
        }
        // Ctrl+F: focus search
        if (e.key === 'f') {
          e.preventDefault();
          if (onFocusSearch) {
            onFocusSearch();
          } else {
            window.dispatchEvent(new CustomEvent('focus-search'));
          }
          return;
        }
      }

      // Guard remaining shortcuts from firing in inputs
      if (inInput) return;
    }

    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [navigate, onFocusQuickEntry, onFocusSearch, onToggleShortcutHelp]);
}
