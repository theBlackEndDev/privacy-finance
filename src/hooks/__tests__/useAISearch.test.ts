import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useAISearch } from '../useAISearch';

const mockSearch = vi.fn();
const mockConfirm = vi.fn();

vi.mock('../../api/ai_search', () => ({
  aiSearchApi: {
    search: (...args: unknown[]) => mockSearch(...args),
    confirmAISearch: (...args: unknown[]) => mockConfirm(...args),
  },
}));

describe('useAISearch', () => {
  beforeEach(() => {
    mockSearch.mockReset();
    mockConfirm.mockReset();
  });

  it('starts with null result and loading false', () => {
    const { result } = renderHook(() => useAISearch());
    expect(result.current.result).toBeNull();
    expect(result.current.loading).toBe(false);
    expect(result.current.error).toBeNull();
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

  it('sets result to null and keeps the error on failure', async () => {
    mockSearch.mockRejectedValue(new Error('network error'));

    const { result } = renderHook(() => useAISearch());

    await act(async () => {
      await result.current.search('fail query');
    });

    expect(result.current.loading).toBe(false);
    expect(result.current.result).toBeNull();
    expect(result.current.error).toBe('Error: network error');
  });

  it('resets the error when a new search starts', async () => {
    mockSearch.mockRejectedValueOnce(new Error('boom'));
    mockSearch.mockResolvedValueOnce({ matched: true, textAnswer: 'ok' });

    const { result } = renderHook(() => useAISearch());

    await act(async () => {
      await result.current.search('first');
    });
    expect(result.current.error).not.toBeNull();

    await act(async () => {
      await result.current.search('second');
    });
    expect(result.current.error).toBeNull();
    expect(result.current.result).not.toBeNull();
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

  it('clears the error', async () => {
    mockSearch.mockRejectedValue(new Error('boom'));

    const { result } = renderHook(() => useAISearch());

    await act(async () => {
      await result.current.search('query');
    });
    expect(result.current.error).not.toBeNull();

    act(() => {
      result.current.clear();
    });
    expect(result.current.error).toBeNull();
  });

  describe('proposal flow', () => {
    const intentMatch = {
      intent: 'SPENDING_BY_CATEGORY' as const,
      confidence: 0.9,
      entities: {
        categoryId: 'cat-1',
        categoryName: 'Groceries',
        accountId: null,
        accountName: null,
        datePeriod: null,
        comparisonPeriod: null,
        amountThreshold: null,
        amountOperator: null,
        merchant: null,
        wantsPercentage: false,
      },
    };
    const proposalResult = {
      matched: false,
      intent: null,
      cardType: null,
      cardData: null,
      textAnswer: null,
      source: 'modelRouter',
      interpretation: null,
      proposal: { interpretation: 'Spending on Groceries', intentMatch },
      suggestions: [],
      queryTimeMs: 5,
    };

    it('confirm sends the intent match and replaces the proposal with the answer', async () => {
      const answer = { ...proposalResult, matched: true, proposal: null, textAnswer: 'You spent $50' };
      mockSearch.mockResolvedValue(proposalResult);
      mockConfirm.mockResolvedValue(answer);

      const { result } = renderHook(() => useAISearch());
      await act(async () => {
        await result.current.search('groceries lately');
      });
      expect(result.current.result?.proposal).not.toBeNull();

      await act(async () => {
        await result.current.confirm(result.current.result!.proposal!.intentMatch);
      });

      expect(mockConfirm).toHaveBeenCalledWith(intentMatch);
      expect(result.current.loading).toBe(false);
      expect(result.current.result).toEqual(answer);
      expect(result.current.error).toBeNull();
    });

    it('confirm keeps the error and clears the result on failure', async () => {
      mockSearch.mockResolvedValue(proposalResult);
      mockConfirm.mockRejectedValue(new Error('not found'));

      const { result } = renderHook(() => useAISearch());
      await act(async () => {
        await result.current.search('q');
      });
      await act(async () => {
        await result.current.confirm(intentMatch);
      });

      expect(result.current.result).toBeNull();
      expect(result.current.error).toBe('Error: not found');
      expect(result.current.loading).toBe(false);
    });

    it('decline replaces the proposal with the decline text and six suggestions, without calling confirm', async () => {
      mockSearch.mockResolvedValue(proposalResult);

      const { result } = renderHook(() => useAISearch());
      await act(async () => {
        await result.current.search('q');
      });

      act(() => {
        result.current.decline();
      });

      expect(mockConfirm).not.toHaveBeenCalled();
      expect(result.current.result?.proposal).toBeNull();
      expect(result.current.result?.matched).toBe(false);
      expect(result.current.result?.source).toBe('none');
      expect(result.current.result?.textAnswer).toBe("I can't answer that yet. Try one of these:");
      expect(result.current.result?.suggestions).toHaveLength(6);
      expect(result.current.error).toBeNull();
    });
  });
});
