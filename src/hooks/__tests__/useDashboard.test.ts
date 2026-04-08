import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act, waitFor } from '@testing-library/react';
import { useDashboard } from '../useDashboard';

const mockGetDashboard = vi.fn();
const mockGetTrend = vi.fn();
const mockGetBalances = vi.fn();

vi.mock('../../api/dashboard', () => ({
  dashboardApi: {
    getDashboardData: (...args: unknown[]) => mockGetDashboard(...args),
    getMonthlyTrend: (...args: unknown[]) => mockGetTrend(...args),
    getAccountBalances: (...args: unknown[]) => mockGetBalances(...args),
  },
}));

const fakeDashboard = { income: 5000, expense: 3000, net: 2000, byCategory: [] };
const fakeTrend = [{ month: '2026-01', income: 5000, expense: 3000 }];
const fakeBalances = [{ accountId: '1', name: 'Checking', balance: 10000 }];

describe('useDashboard', () => {
  beforeEach(() => {
    mockGetDashboard.mockReset().mockResolvedValue(fakeDashboard);
    mockGetTrend.mockReset().mockResolvedValue(fakeTrend);
    mockGetBalances.mockReset().mockResolvedValue(fakeBalances);
  });

  it('loads data on mount', async () => {
    const { result } = renderHook(() => useDashboard());

    expect(result.current.loading).toBe(true);

    await waitFor(() => expect(result.current.loading).toBe(false));

    expect(result.current.data).toEqual(fakeDashboard);
    expect(result.current.trend).toEqual(fakeTrend);
    expect(result.current.balances).toEqual(fakeBalances);
    expect(result.current.error).toBeNull();
  });

  it('sets error on failure', async () => {
    mockGetDashboard.mockRejectedValue(new Error('fail'));

    const { result } = renderHook(() => useDashboard());

    await waitFor(() => expect(result.current.loading).toBe(false));

    expect(result.current.error).toBe('Error: fail');
  });

  it('reloads when month changes', async () => {
    const { result } = renderHook(() => useDashboard());

    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(mockGetDashboard).toHaveBeenCalledTimes(1);

    act(() => {
      result.current.setMonth('2026-02');
    });

    await waitFor(() => expect(mockGetDashboard).toHaveBeenCalledTimes(2));
    expect(mockGetDashboard).toHaveBeenLastCalledWith('2026-02');
  });

  it('exposes a refresh function', async () => {
    const { result } = renderHook(() => useDashboard());

    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(mockGetDashboard).toHaveBeenCalledTimes(1);

    await act(async () => {
      await result.current.refresh();
    });

    expect(mockGetDashboard).toHaveBeenCalledTimes(2);
  });
});
