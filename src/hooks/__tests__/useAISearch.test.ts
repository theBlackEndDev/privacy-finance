import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useAISearch } from '../useAISearch';

const mockSearch = vi.fn();

vi.mock('../../api/ai_search', () => ({
  aiSearchApi: {
    search: (...args: unknown[]) => mockSearch(...args),
  },
}));

describe('useAISearch', () => {
  beforeEach(() => {
    mockSearch.mockReset();
  });

  it('starts with null result and loading false', () => {
    const { result } = renderHook(() => useAISearch());
    expect(result.current.result).toBeNull();
    expect(result.current.loading).toBe(false);
  });

  it('sets loading during search and returns result', async () => {
    const fakeResult = {
      matched: true,
      intent: null,
      cardType: null,
      cardData: null,
      textAnswer: 'You spent $50',
      suggestions: [],
      queryTimeMs: 42,
    };
    mockSearch.mockResolvedValue(fakeResult);

    const { result } = renderHook(() => useAISearch());

    await act(async () => {
      await result.current.search('how much did I spend');
    });

    expect(result.current.loading).toBe(false);
    expect(result.current.result).toEqual(fakeResult);
    expect(mockSearch).toHaveBeenCalledWith('how much did I spend');
  });

  it('sets result to null on error', async () => {
    mockSearch.mockRejectedValue(new Error('network error'));

    const { result } = renderHook(() => useAISearch());

    await act(async () => {
      await result.current.search('fail query');
    });

    expect(result.current.loading).toBe(false);
    expect(result.current.result).toBeNull();
  });

  it('clears the result', async () => {
    mockSearch.mockResolvedValue({ matched: true, textAnswer: 'answer' });

    const { result } = renderHook(() => useAISearch());

    await act(async () => {
      await result.current.search('query');
    });
    expect(result.current.result).not.toBeNull();

    act(() => {
      result.current.clear();
    });
    expect(result.current.result).toBeNull();
  });
});
