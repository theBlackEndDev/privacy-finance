import { browser } from '@wdio/globals';
import { sel, byTestIdPrefix } from '../helpers/selectors';
import { waitForAppReady, clickButton, fillInput, isVisible, countElements } from '../helpers/app';

describe('Search', () => {
  before(async () => {
    await waitForAppReady();
  });

  it('keyword search finds transaction by payee', async () => {
    await clickButton(sel.globalSearch);

    const searchInput = await $(sel.searchInput);
    await searchInput.waitForDisplayed({ timeout: 5000 });
    await searchInput.setValue('Whole Foods');

    const results = await $(sel.searchResults);
    await results.waitForDisplayed({ timeout: 10000 });

    const resultCount = await countElements(`${sel.searchResults} ${byTestIdPrefix('tx-row-')}`);
    expect(resultCount).toBeGreaterThan(0);

    const resultsText = await results.getText();
    expect(resultsText.toLowerCase()).toContain('whole foods');
  });

  it('AI search answers spending question', async () => {
    await clickButton(sel.globalSearch);

    const searchInput = await $(sel.searchInput);
    await searchInput.waitForDisplayed({ timeout: 5000 });
    await searchInput.setValue('How much did I spend on food this month?');

    const aiCard = await $(sel.aiSearchCard);
    await aiCard.waitForDisplayed({ timeout: 30000 });

    const cardText = await aiCard.getText();
    expect(cardText).toBeTruthy();
    // AI search card should contain a dollar amount in the spending summary
    expect(cardText).toMatch(/\$[\d,.]+/);
  });
});
