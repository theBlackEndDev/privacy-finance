import { useState } from 'react';
import { invoke } from '../../api/invoke';
import styles from './AskBar.module.css';

interface InferenceResult {
  matched: boolean;
  templateName: string | null;
  answer: string | null;
  rawValue: number | null;
  querySql: string | null;
  suggestions: string[];
}

export function AskBar() {
  const [question, setQuestion] = useState('');
  const [result, setResult] = useState<InferenceResult | null>(null);
  const [loading, setLoading] = useState(false);

  async function handleAsk(e: React.FormEvent) {
    e.preventDefault();
    if (!question.trim()) return;

    setLoading(true);
    setResult(null);
    try {
      const res = await invoke<InferenceResult>('ask_question', { question: question.trim() });
      setResult(res);
    } catch (err) {
      setResult({
        matched: false,
        templateName: null,
        answer: null,
        rawValue: null,
        querySql: null,
        suggestions: [`Error: ${String(err)}`],
      });
    } finally {
      setLoading(false);
    }
  }

  function handleSuggestion(text: string) {
    const clean = text.replace(/\[.*?\]/g, '').replace(/\?$/, '').trim();
    setQuestion(clean);
  }

  return (
    <div className={styles.container}>
      <form onSubmit={handleAsk} className={styles.form}>
        <span className={styles.icon}>💬</span>
        <input
          className={styles.input}
          value={question}
          onChange={(e) => setQuestion(e.target.value)}
          placeholder="Ask about your finances... e.g. &quot;How much did I spend on food?&quot;"
          disabled={loading}
        />
        <button type="submit" className={styles.button} disabled={loading || !question.trim()}>
          {loading ? '...' : 'Ask'}
        </button>
      </form>

      {result && (
        <div className={styles.result}>
          {result.matched && result.answer ? (
            <div className={styles.answer}>
              <span className={styles.answerIcon}>✨</span>
              <span>{result.answer}</span>
            </div>
          ) : (
            <div className={styles.noMatch}>
              <div className={styles.noMatchText}>
                I couldn't match that query. Try one of these:
              </div>
              <div className={styles.suggestions}>
                {result.suggestions.map((s, i) => (
                  <button
                    key={i}
                    className={styles.suggestion}
                    onClick={() => handleSuggestion(s)}
                  >
                    {s}
                  </button>
                ))}
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
