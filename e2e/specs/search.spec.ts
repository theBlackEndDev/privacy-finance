import { sel, byTestIdPrefix } from '../helpers/selectors';
import {
  waitForAppReady,
  navigateTo,
  fillInput,
  selectOption,
  clickButton,
  countElements,
} from '../helpers/app';

describe('Search', () => {
  before(async () => {
    await waitForAppReady();

    // Ensure at least one transaction exists for search to find.
    // The transactions spec may or may not have run yet (depends on run order).
    await navigateTo('transactions');
    await browser.pause(500);

    const txCount = await countElements(byTestIdPrefix('tx-row-'));
    if (txCount === 0) {
      // Create a transaction so search has something to find
      await fillInput(sel.txPayeeInput, 'Grocery Mart');
      await fillInput(sel.txAmountInput, '55.00');

      // Try selecting a category (may fail if no Food category, that's fine)
      const catSelect = await $(sel.txCategorySelect);
      if (await catSelect.isExisting()) {
        try {
          await catSelect.selectByVisibleText('Food');
        } catch {
          // Category may not exist, proceed without it
        }
      }

      await clickButton(sel.txSubmitBtn);
      await browser.pause(500);
    }
  });

  it('keyword search finds transaction by payee', async () => {
    // The GlobalSearch input is always visible in the header (it's not behind a button click).
    // Just focus and type directly into search-input.
    const searchInput = await $(sel.searchInput);
    await searchInput.waitForDisplayed({ timeout: 5000 });
    await searchInput.click();
    await searchInput.setValue('Grocery');

    // Wait for debounce (250ms) plus rendering
    await browser.pause(800);

    const results = await $(sel.searchResults);
    await results.waitForDisplayed({ timeout: 10000 });

    // Verify results contain matching text
    const resultsText = await results.getText();
    expect(resultsText.toLowerCase()).toContain('grocery');

    // Clear search to reset state
    await searchInput.clearValue();
    await browser.pause(300);
  });

  it('AI search shows answer card for spending question', async () => {
    const searchInput = await $(sel.searchInput);
    await searchInput.waitForDisplayed({ timeout: 5000 });
    await searchInput.click();
    await searchInput.setValue('How much did I spend on food this month?');

    const aiCard = await $(sel.aiSearchCard);
    await aiCard.waitForDisplayed({ timeout: 30000 });

    const cardText = await aiCard.getText();
    expect(cardText).toBeTruthy();

    // Clear search
    await searchInput.clearValue();
    await browser.pause(300);
  });
});
