const QUESTION_WORDS = [
  'how', 'what', 'where', 'when', 'why', 'which', 'who',
  'am', 'is', 'are', 'do', 'did', 'does', 'can',
  'show', 'list', 'tell', 'compare', 'find',
];

export function isQuestion(query: string): boolean {
  const trimmed = query.trim().toLowerCase();
  if (!trimmed) return false;
  if (trimmed.includes('?')) return true;
  const firstWord = trimmed.split(/\s+/)[0];
  return QUESTION_WORDS.includes(firstWord);
}
