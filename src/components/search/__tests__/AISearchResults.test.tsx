import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { AISearchResults } from '../AISearchResults';
import { useAISearch } from '../../../hooks/useAISearch';
import type { AISearchResult, IntentMatch } from '../../../types';

const mockSearch = vi.fn();
const mockConfirm = vi.fn();

vi.mock('../../../api/ai_search', () => ({
  aiSearchApi: {
    search: (...args: unknown[]) => mockSearch(...args),
    confirmAISearch: (...args: unknown[]) => mockConfirm(...args),
  },
}));

const noop = { onConfirm: () => {}, onDecline: () => {} };

function makeResult(overrides: Partial<AISearchResult>): AISearchResult {
  return {
    matched: true,
    intent: 'SPENDING_TOTAL',
    cardType: null,
    cardData: null,
    textAnswer: 'You spent $120.00 on Food.',
    source: 'template',
    interpretation: 'Spending in Food, last month',
    proposal: null,
    suggestions: [],
    queryTimeMs: 0,
    ...overrides,
  };
}

describe('AISearchResults interpretation', () => {
  it('shows the Understood as line for a template answer', () => {
    render(<AISearchResults result={makeResult({ source: 'template' })} loading={false} {...noop} />);
    expect(screen.getByText('Understood as: Spending in Food, last month')).toBeInTheDocument();
  });

  it('shows the Understood as line for a model router answer', () => {
    render(
      <AISearchResults
        result={makeResult({ source: 'modelRouter', interpretation: 'Transactions at Starbucks over $20' })}
        loading={false}
        {...noop}
      />,
    );
    expect(screen.getByText('Understood as: Transactions at Starbucks over $20')).toBeInTheDocument();
  });

  it('hides the Understood as line when the source is none', () => {
    render(<AISearchResults result={makeResult({ source: 'none' })} loading={false} {...noop} />);
    expect(screen.getByText('You spent $120.00 on Food.')).toBeInTheDocument();
    expect(screen.queryByText(/Understood as/)).not.toBeInTheDocument();
  });
});

describe('AISearchResults decline', () => {
  it('shows the decline text and suggestions when unmatched', () => {
    render(
      <AISearchResults
        result={makeResult({
          matched: false,
          source: 'none',
          interpretation: null,
          textAnswer: "I can't answer that yet. Try one of these:",
          suggestions: ['How much did I spend on groceries last month?'],
        })}
        loading={false}
        {...noop}
      />,
    );
    expect(screen.getByText("I can't answer that yet. Try one of these:")).toBeInTheDocument();
    expect(screen.getByText('How much did I spend on groceries last month?')).toBeInTheDocument();
  });
});

const intentMatch: IntentMatch = {
  intent: 'TRANSACTION_SEARCH' as IntentMatch['intent'],
  confidence: 0.5,
  entities: {
    categoryId: null,
    categoryName: null,
    accountId: null,
    accountName: null,
    datePeriod: null,
    comparisonPeriod: null,
    amountThreshold: 20,
    amountOperator: 'over',
    merchant: 'Starbucks',
    wantsPercentage: false,
  },
};

const proposalResult = makeResult({
  matched: false,
  source: 'modelRouter',
  cardData: null,
  textAnswer: null,
  interpretation: 'Transactions at Starbucks over $20',
  proposal: { interpretation: 'Transactions at Starbucks over $20', intentMatch },
});

function Harness() {
  const { result, loading, search, confirm, decline } = useAISearch();
  return (
    <div>
      <button onClick={() => search('starbucks')}>ask</button>
      {result && <AISearchResults result={result} loading={loading} onConfirm={confirm} onDecline={decline} />}
    </div>
  );
}

describe('AISearchResults proposal', () => {
  it('shows Did you mean and no card before confirming', () => {
    render(<AISearchResults result={proposalResult} loading={false} {...noop} />);
    expect(screen.getByText('Did you mean: Transactions at Starbucks over $20?')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Show answer' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'No' })).toBeInTheDocument();
    expect(screen.queryByTestId('ai-search-card')).not.toBeInTheDocument();
  });

  it('calls confirm with the intent match when Show answer is clicked', () => {
    const onConfirm = vi.fn();
    render(<AISearchResults result={proposalResult} loading={false} onConfirm={onConfirm} onDecline={() => {}} />);
    fireEvent.click(screen.getByRole('button', { name: 'Show answer' }));
    expect(onConfirm).toHaveBeenCalledWith(intentMatch);
  });

  it('shows the decline text when No is clicked', async () => {
    mockSearch.mockResolvedValue(proposalResult);
    render(<Harness />);
    fireEvent.click(screen.getByText('ask'));
    fireEvent.click(await screen.findByRole('button', { name: 'No' }));
    await waitFor(() =>
      expect(screen.getByText("I can't answer that yet. Try one of these:")).toBeInTheDocument(),
    );
    expect(screen.queryByText(/Did you mean/)).not.toBeInTheDocument();
    expect(mockConfirm).not.toHaveBeenCalled();
  });
});

describe('AISearchResults beta', () => {
  it('shows the Beta badge and warning for an answer', () => {
    render(<AISearchResults result={makeResult({})} loading={false} {...noop} />);
    expect(screen.getByText('Beta')).toBeInTheDocument();
    expect(screen.getByText('AI search is in beta. Answers can be wrong or fail.')).toBeInTheDocument();
  });

  it('shows the Beta badge and warning for a proposal', () => {
    render(<AISearchResults result={proposalResult} loading={false} {...noop} />);
    expect(screen.getByText('Beta')).toBeInTheDocument();
    expect(screen.getByText('AI search is in beta. Answers can be wrong or fail.')).toBeInTheDocument();
  });

  it('shows the Beta badge and warning for the decline message', () => {
    render(
      <AISearchResults
        result={makeResult({
          matched: false,
          source: 'none',
          interpretation: null,
          textAnswer: "I can't answer that yet. Try one of these:",
        })}
        loading={false}
        {...noop}
      />,
    );
    expect(screen.getByText('Beta')).toBeInTheDocument();
    expect(screen.getByText('AI search is in beta. Answers can be wrong or fail.')).toBeInTheDocument();
  });
});
