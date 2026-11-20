import { AISearchResult, IntentMatch } from '../../types';
import { TotalCard } from './cards/TotalCard';
import { ComparisonCard } from './cards/ComparisonCard';
import { BudgetStatusCard } from './cards/BudgetStatusCard';
import { TransactionListCard } from './cards/TransactionListCard';
import { BreakdownCard } from './cards/BreakdownCard';
import { AccountSummaryCard } from './cards/AccountSummaryCard';
import styles from './AISearchResults.module.css';

interface Props {
  result: AISearchResult;
  loading: boolean;
  onConfirm: (intentMatch: IntentMatch) => void;
  onDecline: () => void;
}

export function AISearchResults({ result, loading, onConfirm, onDecline }: Props) {
  if (loading) {
    return <div className={styles.container}><div className={styles.loading}>Searching...</div></div>;
  }

  const betaBanner = (
    <div className={styles.betaBanner} data-testid="ai-search-beta">
      <span className={styles.betaBadge}>Beta</span>
      <span className={styles.betaText}>AI search is in beta. Answers can be wrong or fail.</span>
    </div>
  );

  if (result.proposal) {
    const { proposal } = result;
    return (
      <div className={styles.container}>
        {betaBanner}
        <div className={styles.proposal} data-testid="ai-search-proposal">
          <div className={styles.proposalText}>Did you mean: {proposal.interpretation}?</div>
          <div className={styles.proposalActions}>
            <button type="button" className={styles.proposalConfirm} onClick={() => onConfirm(proposal.intentMatch)}>
              Show answer
            </button>
            <button type="button" className={styles.proposalDecline} onClick={onDecline}>
              No
            </button>
          </div>
        </div>
      </div>
    );
  }

  if (!result.matched) {
    return (
      <div className={styles.container}>
        {betaBanner}
        <div className={styles.noMatch}>
          {result.textAnswer ?? "Couldn't find an answer for that."}
        </div>
        {result.suggestions.length > 0 && (
          <div className={styles.suggestions}>
            <div className={styles.suggestLabel}>Try asking:</div>
            {result.suggestions.map((s, i) => (
              <div key={i} className={styles.suggestion}>{s}</div>
            ))}
          </div>
        )}
      </div>
    );
  }

  return (
    <div className={styles.container}>
      {betaBanner}
      {result.source !== 'none' && result.interpretation && (
        <div className={styles.interpretation} data-testid="ai-search-interpretation">
          Understood as: {result.interpretation}
        </div>
      )}
      {result.textAnswer && (
        <div className={styles.textAnswer}>{result.textAnswer}</div>
      )}
      {result.cardData && <div data-testid="ai-search-card">{renderCard(result.cardData)}</div>}
      {result.queryTimeMs > 0 && (
        <div className={styles.timing}>Answered in {result.queryTimeMs}ms</div>
      )}
    </div>
  );
}

function renderCard(cardData: AISearchResult['cardData']) {
  if (!cardData) return null;
  switch (cardData.type) {
    case 'total': return <TotalCard data={cardData} />;
    case 'comparison': return <ComparisonCard data={cardData} />;
    case 'budgetStatus': return <BudgetStatusCard data={cardData} />;
    case 'transactionList': return <TransactionListCard data={cardData} />;
    case 'breakdown': return <BreakdownCard data={cardData} />;
    case 'accountSummary': return <AccountSummaryCard data={cardData} />;
    default: return null;
  }
}
