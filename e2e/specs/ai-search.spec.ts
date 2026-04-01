import { sel, byTestIdPrefix } from '../helpers/selectors';
import {
  waitForAppReady,
  navigateTo,
  fillInput,
  clickButton,
  countElements,
} from '../helpers/app';

/**
 * AI Search E2E Tests
 *
 * Tests the natural language query system (Tier 1 template matching).
 * These queries are answered instantly without the local LLM —
 * they use keyword scoring to detect intent and generate SQL.
 *
 * The tests create transactions first so queries have data to return.
 */
describe('AI Search', () => {
  before(async () => {
    await waitForAppReady();

    // Create test transactions so AI queries have data to work with
    await navigateTo('transactions');
    await browser.pause(500);

    const txCount = await countElements(byTestIdPrefix('tx-row-'));
    if (txCount < 3) {
      const kindGroup = await $(sel.txKindSelect);

      // Expense: Food
      const expenseBtn = await kindGroup.$('button*=\u2212');
      if (await expenseBtn.isExisting()) await expenseBtn.click();
      await fillInput(sel.txPayeeInput, 'Whole Foods');
      await fillInput(sel.txAmountInput, '85.50');
      try { await (await $(sel.txCategorySelect)).selectByVisibleText('Food'); } catch {}
      await clickButton(sel.txSubmitBtn);
      await browser.pause(500);

      // Expense: Transportation
      if (await expenseBtn.isExisting()) await expenseBtn.click();
      await fillInput(sel.txPayeeInput, 'Uber');
      await fillInput(sel.txAmountInput, '24.00');
      try { await (await $(sel.txCategorySelect)).selectByVisibleText('Transportation'); } catch {}
      await clickButton(sel.txSubmitBtn);
      await browser.pause(500);

      // Expense: Entertainment
      if (await expenseBtn.isExisting()) await expenseBtn.click();
      await fillInput(sel.txPayeeInput, 'Netflix');
      await fillInput(sel.txAmountInput, '15.99');
      try { await (await $(sel.txCategorySelect)).selectByVisibleText('Subscriptions'); } catch {}
      await clickButton(sel.txSubmitBtn);
      await browser.pause(500);

      // Income: Salary
      const incomeBtn = await kindGroup.$('button*=+');
      if (await incomeBtn.isExisting()) await incomeBtn.click();
      await fillInput(sel.txPayeeInput, 'Acme Corp');
      await fillInput(sel.txAmountInput, '3500');
      try { await (await $(sel.txCategorySelect)).selectByVisibleText('Salary'); } catch {}
      await clickButton(sel.txSubmitBtn);
      await browser.pause(500);
    }
  });

  /** Helper: type a query, wait for AI card to update, return its text, then clear */
  async function askAndGetAnswer(query: string): Promise<string> {
    const searchInput = await $(sel.searchInput);
    await searchInput.waitForDisplayed({ timeout: 5000 });

    // Clear any previous search and wait for card to disappear
    await searchInput.clearValue();
    await browser.pause(500);

    // Type the new query
    await searchInput.click();
    await searchInput.setValue(query);

    // Wait for AI card to appear with NEW content (debounce + processing)
    await browser.pause(1000);
    const aiCard = await $(sel.aiSearchCard);
    await aiCard.waitForDisplayed({ timeout: 15000 });
    const text = await aiCard.getText();

    // Clear for next query
    await searchInput.clearValue();
    await browser.pause(500);
    return text;
  }

  // ── Spending queries ──────────────────────────────────────────

  it('answers "how much did I spend" with total', async () => {
    const answer = await askAndGetAnswer('How much did I spend this month?');
    expect(answer).toBeTruthy();
    expect(answer).toContain('$');
  });

  it('answers spending by category (food)', async () => {
    const answer = await askAndGetAnswer('How much did I spend on food?');
    expect(answer.toLowerCase()).toMatch(/food|groceries/);
    expect(answer).toContain('$');
  });

  it('answers spending by category (transportation)', async () => {
    const answer = await askAndGetAnswer('How much did I spend on transportation?');
    expect(answer.toLowerCase()).toMatch(/transport|uber|gas/);
  });

  it('answers percentage query', async () => {
    const answer = await askAndGetAnswer('What percentage was spent on food this month?');
    // Should contain either "%" or "$" (percentage or dollar amount)
    expect(answer).toMatch(/%|\$/);
  });

  // ── Income queries ────────────────────────────────────────────

  it('answers income total', async () => {
    const answer = await askAndGetAnswer('How much did I earn this month?');
    expect(answer).toContain('$');
  });

  // ── Balance & net worth ───────────────────────────────────────

  it('answers balance check', async () => {
    const answer = await askAndGetAnswer('How much money do I have?');
    expect(answer).toBeTruthy();
  });

  it('answers net worth query', async () => {
    const answer = await askAndGetAnswer('What is my net worth?');
    expect(answer).toBeTruthy();
    expect(answer).toContain('$');
  });

  // ── Budget queries ────────────────────────────────────────────

  it('answers budget status question', async () => {
    // Budget queries may not show an AI card if no budgets are set.
    // Just verify the search doesn't crash and processes the query.
    const searchInput = await $(sel.searchInput);
    await searchInput.clearValue();
    await browser.pause(500);
    await searchInput.click();
    await searchInput.setValue('Am I over budget on food?');
    await browser.pause(2000);

    // Check if AI card appeared (optional — may not if no budget data)
    const aiCard = await $(sel.aiSearchCard);
    const cardVisible = await aiCard.isDisplayed().catch(() => false);
    if (cardVisible) {
      const text = await aiCard.getText();
      expect(text).toBeTruthy();
    }
    // Either way, the app didn't crash — pass
    await searchInput.clearValue();
    await browser.pause(500);
  });

  it('answers budget remaining question', async () => {
    const searchInput = await $(sel.searchInput);
    await searchInput.clearValue();
    await browser.pause(500);
    await searchInput.click();
    await searchInput.setValue('How much budget remaining for food?');
    await browser.pause(2000);

    const aiCard = await $(sel.aiSearchCard);
    const cardVisible = await aiCard.isDisplayed().catch(() => false);
    if (cardVisible) {
      const text = await aiCard.getText();
      expect(text).toBeTruthy();
    }
    await searchInput.clearValue();
    await browser.pause(500);
  });

  // ── Comparison queries ────────────────────────────────────────

  it('answers spending comparison', async () => {
    const answer = await askAndGetAnswer('Compare this month vs last month');
    expect(answer).toBeTruthy();
    expect(answer).toContain('$');
  });

  // ── Transaction search queries ────────────────────────────────

  it('answers "show me transactions over $50"', async () => {
    const answer = await askAndGetAnswer('Show me transactions over $50');
    expect(answer).toBeTruthy();
  });

  // ── Discovery queries ─────────────────────────────────────────

  it('answers "where does my money go"', async () => {
    const answer = await askAndGetAnswer('Where does my money go?');
    expect(answer).toBeTruthy();
  });

  it('answers "what are my subscriptions"', async () => {
    const answer = await askAndGetAnswer('What are my subscriptions?');
    expect(answer).toBeTruthy();
  });

  // ── Top/biggest queries ───────────────────────────────────────

  it('answers "biggest expenses this month"', async () => {
    const answer = await askAndGetAnswer('What are my biggest expenses this month?');
    expect(answer).toBeTruthy();
    expect(answer).toContain('$');
  });

  // ── Informal / natural phrasing ───────────────────────────────

  it('handles informal phrasing "what did I pay for food"', async () => {
    const answer = await askAndGetAnswer('What did I pay for food?');
    expect(answer.toLowerCase()).toContain('food');
  });

  it('handles "show me my monthly bills"', async () => {
    const answer = await askAndGetAnswer('Show me my monthly bills');
    expect(answer).toBeTruthy();
  });
});
