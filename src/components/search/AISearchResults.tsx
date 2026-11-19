import { AISearchResult } from '../../types';
import { TotalCard } from './cards/TotalCard';
import { ComparisonCard } from './cards/ComparisonCard';
import { BudgetStatusCard } from './cards/BudgetStatusCard';
import { TransactionListCard } from './cards/TransactionListCard';
import { BreakdownCard } from './cards/BreakdownCard';
import { AccountSummaryCard } from './cards/AccountSummaryCard';
import { TableCard } from './cards/TableCard';
import styles from './AISearchResults.module.css';

interface Props {
  result: AISearchResult;
  loading: boolean;
}

export function AISearchResults({ result, loading }: Props) {
  if (loading) {
    return <div className={styles.container}><div className={styles.loading}>Searching...</div></div>;
  }

  if (!result.matched) {
    return (
      <div className={styles.container}>
        <div className={styles.noMatch}>
          Couldn't find an answer for that.
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
      {result.textAnswer && (
        <div className={styles.textAnswer}>{result.textAnswer}</div>
      )}
      {result.cardData && <div data-testid="ai-search-card">{renderCard(result.cardData)}</div>}
      {result.sql && (
        <details className={styles.sql}>
          <summary>SQL</summary>
          <pre>{result.sql}</pre>
        </details>
      )}
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
    case 'table': return <TableCard data={cardData} />;
    default: return null;
  }
}
