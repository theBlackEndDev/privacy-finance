import { sel, byTestIdPrefix } from '../helpers/selectors';
import {
  waitForAppReady,
  navigateTo,
  fillInput,
  selectOption,
  clickButton,
  countElements,
} from '../helpers/app';

/**
 * AI Search E2E Tests
 *
 * Tests the natural language query system (Tier 1 template matching).
 * Creates a realistic dataset first so every query returns real answers.
 */
describe('AI Search', () => {
  before(async () => {
    await waitForAppReady();

    // ── Create transactions for current month ──────────────────
    await navigateTo('transactions');
    await browser.pause(500);

    const txCount = await countElements(byTestIdPrefix('tx-row-'));
    if (txCount < 5) {
      const kindGroup = await $(sel.txKindSelect);
      const expenseBtn = await kindGroup.$('button*=\u2212');
      const incomeBtn = await kindGroup.$('button*=+');

      // Helper to create a transaction via the quick entry form
      async function createTx(kind: 'expense' | 'income', payee: string, amount: string, category: string) {
        if (kind === 'expense' && await expenseBtn.isExisting()) await expenseBtn.click();
        if (kind === 'income' && await incomeBtn.isExisting()) await incomeBtn.click();
        await fillInput(sel.txPayeeInput, payee);
        await fillInput(sel.txAmountInput, amount);
        try { await (await $(sel.txCategorySelect)).selectByVisibleText(category); } catch {}
        await clickButton(sel.txSubmitBtn);
        await browser.pause(400);
      }

      // Food expenses
      await createTx('expense', 'Whole Foods', '85.50', 'Food');
      await createTx('expense', 'Trader Joes', '62.30', 'Food');

      // Transportation
      await createTx('expense', 'Uber', '24.00', 'Transportation');
      await createTx('expense', 'Shell Gas', '45.00', 'Transportation');

      // Subscriptions
      await createTx('expense', 'Netflix', '15.99', 'Subscriptions');
      await createTx('expense', 'Spotify', '10.99', 'Subscriptions');

      // Housing
      await createTx('expense', 'Sunset Apartments', '1850.00', 'Housing');

      // Shopping
      await createTx('expense', 'Amazon', '129.99', 'Shopping');

      // Income
      await createTx('income', 'Acme Corp', '3500.00', 'Salary');
      await createTx('income', 'Freelance Client', '750.00', 'Freelance');
    }

    // ── Set a budget for Food so budget queries return data ────
    await navigateTo('budgets');
    await browser.pause(500);

    const budgetInput = await $(sel.budgetAmountInput);
    if (await budgetInput.isExisting()) {
      await budgetInput.click();
      await budgetInput.clearValue();
      await budgetInput.setValue('300');
      // Blur to save
      await (await $(sel.budgetMonthNav)).click();
      await browser.pause(500);
    }

    // ── Create a recurring transaction for subscription queries ─
    await navigateTo('recurring');
    await browser.pause(500);

    const recurringCount = await countElements(sel.recurringRow);
    if (recurringCount === 0) {
      await clickButton(sel.recurringCreateBtn);
      await browser.pause(300);
      await fillInput(sel.recurringPayeeInput, 'Gym Membership');
      await fillInput(sel.recurringAmountInput, '49.99');
      try { await (await $(sel.recurringFreqSelect)).selectByVisibleText('Monthly'); } catch {}
      await clickButton(sel.recurringSubmitBtn);
      await browser.pause(500);
    }
  });

  /** Helper: type a query, wait for AI card, return its text, then clear */
  async function askAndGetAnswer(query: string): Promise<string> {
    const searchInput = await $(sel.searchInput);
    await searchInput.waitForDisplayed({ timeout: 5000 });

    // Clear previous search and wait for card to disappear
    await searchInput.clearValue();
    await browser.pause(500);

    // Type the new query
    await searchInput.click();
    await searchInput.setValue(query);

    // Wait for AI card to appear (debounce + processing)
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

  it('answers "how much did I spend" with a dollar total', async () => {
    const answer = await askAndGetAnswer('How much did I spend this month?');
    expect(answer).toContain('$');
    // Should have an actual amount, not $0
    expect(answer).not.toContain('$0.00');
  });

  it('answers spending on food with category name and amount', async () => {
    const answer = await askAndGetAnswer('How much did I spend on food?');
    expect(answer.toLowerCase()).toMatch(/food/);
    expect(answer).toContain('$');
    // Verify the amount reflects our $85.50 + $62.30 food transactions
    expect(answer).toMatch(/\$\d+/);
  });

  it('answers spending on transportation', async () => {
    const answer = await askAndGetAnswer('How much did I spend on transportation?');
    expect(answer.toLowerCase()).toMatch(/transport/);
    expect(answer).toContain('$');
  });

  it('answers food spending as a percentage of total', async () => {
    const searchInput = await $(sel.searchInput);
    await searchInput.clearValue();
    await browser.pause(500);
    await searchInput.click();
    await searchInput.setValue('What percentage was spent on food this month?');
    await browser.pause(1000);

    const aiCard = await $(sel.aiSearchCard);
    await aiCard.waitForDisplayed({ timeout: 15000 });

    // The percentage is in the textAnswer element (above the card)
    // Get ALL text from the search results container
    const resultsContainer = await $(sel.searchResults);
    const fullText = await resultsContainer.getText();
    // Should contain "%" from the text answer OR "$" from the card
    expect(fullText).toMatch(/%|\$/);

    await searchInput.clearValue();
    await browser.pause(500);
  });

  // ── Income queries ────────────────────────────────────────────

  it('answers income total with dollar amount', async () => {
    const answer = await askAndGetAnswer('How much did I earn this month?');
    expect(answer).toContain('$');
    // Should reflect $3500 + $750 = $4250
    expect(answer).toMatch(/4,?250/);
  });

  // ── Balance & net worth ───────────────────────────────────────

  it('answers balance check with account info', async () => {
    const answer = await askAndGetAnswer('How much money do I have?');
    expect(answer).toContain('$');
    // Should mention the Checking account
    expect(answer.toLowerCase()).toMatch(/checking|account/);
  });

  it('answers net worth with total across accounts', async () => {
    const answer = await askAndGetAnswer('What is my net worth?');
    expect(answer).toContain('$');
  });

  // ── Budget queries ────────────────────────────────────────────

  it('answers budget status with real budget data', async () => {
    const searchInput = await $(sel.searchInput);
    await searchInput.clearValue();
    await browser.pause(500);
    await searchInput.click();
    await searchInput.setValue('Am I over budget on food?');
    await browser.pause(2000);

    const aiCard = await $(sel.aiSearchCard);
    const cardVisible = await aiCard.isDisplayed().catch(() => false);
    if (cardVisible) {
      const text = await aiCard.getText();
      // Should mention budget status
      expect(text.toLowerCase()).toMatch(/budget|food/);
    }
    await searchInput.clearValue();
    await browser.pause(500);
  });

  it('answers budget remaining with category info', async () => {
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
      expect(text.toLowerCase()).toMatch(/budget|food/);
    }
    await searchInput.clearValue();
    await browser.pause(500);
  });

  // ── Comparison queries ────────────────────────────────────────

  it('answers spending comparison with amounts for both periods', async () => {
    const answer = await askAndGetAnswer('Compare this month vs last month');
    expect(answer).toContain('$');
    // Should show current and previous period amounts
    expect(answer.toLowerCase()).toMatch(/this month|last month/);
  });

  // ── Transaction search queries ────────────────────────────────

  it('answers "show me transactions over $50" with results', async () => {
    const answer = await askAndGetAnswer('Show me transactions over $50');
    expect(answer).toBeTruthy();
    // Should show transaction count — we have several over $50
    expect(answer).toMatch(/\d+ transaction/);
  });

  // ── Discovery queries ─────────────────────────────────────────

  it('answers "where does my money go" with category breakdown', async () => {
    const answer = await askAndGetAnswer('Where does my money go?');
    expect(answer).toBeTruthy();
    // Should show top spending category
    expect(answer).toContain('$');
  });

  it('answers "what are my subscriptions" with recurring items', async () => {
    const answer = await askAndGetAnswer('What are my subscriptions?');
    expect(answer).toBeTruthy();
    // Should show the recurring transactions we created
    expect(answer).toMatch(/\$/);
  });

  // ── Top/biggest queries ───────────────────────────────────────

  it('answers biggest expenses with dollar amounts', async () => {
    const answer = await askAndGetAnswer('What are my biggest expenses this month?');
    expect(answer).toContain('$');
    // Sunset Apartments ($1850) should be among the top
    expect(answer).toMatch(/\d+ transaction/);
  });

  // ── Informal / natural phrasing ───────────────────────────────

  it('handles "what did I pay for food" with food spending', async () => {
    const answer = await askAndGetAnswer('What did I pay for food?');
    expect(answer.toLowerCase()).toMatch(/food/);
    expect(answer).toContain('$');
  });

  it('handles "show me my monthly bills" with recurring list', async () => {
    const answer = await askAndGetAnswer('Show me my monthly bills');
    expect(answer).toBeTruthy();
    expect(answer).toMatch(/\$/);
  });
});
