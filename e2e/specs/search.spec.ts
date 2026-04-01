import { sel } from '../helpers/selectors';
import {
  waitForAppReady,
  clickButton,
  countElements,
} from '../helpers/app';

describe('Search', () => {
  before(async () => {
    await waitForAppReady();
  });

  it('keyword search finds transaction by payee', async () => {
    await clickButton(sel.globalSearch);

    const searchInput = await $(sel.searchInput);
    await searchInput.waitForDisplayed({ timeout: 5000 });
    await searchInput.setValue('Grocery Mart');

    // Wait for debounce (250ms) plus rendering
    await browser.pause(500);

    const results = await $(sel.searchResults);
    await results.waitForDisplayed({ timeout: 10000 });

    // Check that results contain matching items
    const resultItems = await results.$$('.resultItem');
    expect(resultItems.length).toBeGreaterThan(0);

    const resultsText = await results.getText();
    expect(resultsText.toLowerCase()).toContain('grocery');
  });

  it('AI search shows answer card for spending question', async () => {
    await clickButton(sel.globalSearch);

    const searchInput = await $(sel.searchInput);
    await searchInput.waitForDisplayed({ timeout: 5000 });
    await searchInput.setValue('How much did I spend on food this month?');

    const aiCard = await $(sel.aiSearchCard);
    await aiCard.waitForDisplayed({ timeout: 30000 });

    const cardText = await aiCard.getText();
    expect(cardText).toBeTruthy();
  });
});
