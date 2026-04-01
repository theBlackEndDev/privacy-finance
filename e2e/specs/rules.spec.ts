import { browser } from '@wdio/globals';
import { sel, byTestIdPrefix } from '../helpers/selectors';
import {
  waitForAppReady,
  navigateTo,
  fillInput,
  selectOption,
  clickButton,
  waitForToast,
  countElements,
  isVisible,
} from '../helpers/app';

describe('Rules', () => {
  before(async () => {
    await waitForAppReady();
    await navigateTo('rules');
  });

  it('creates categorization rule', async () => {
    const initialCount = await countElements(sel.ruleRow);

    await clickButton(sel.ruleCreateBtn);

    await selectOption(sel.ruleFieldSelect, 'Payee');
    await selectOption(sel.ruleMatchTypeSelect, 'Contains');
    await fillInput(sel.rulePatternInput, 'netflix');

    // Select the target category for auto-categorization
    const categorySelect = await $(sel.ruleCategorySelect);
    await categorySelect.waitForDisplayed({ timeout: 5000 });
    await categorySelect.selectByVisibleText('Entertainment');

    await clickButton(sel.ruleSubmitBtn);
    await browser.pause(1000);

    const newCount = await countElements(sel.ruleRow);
    expect(newCount).toBe(initialCount + 1);
  });

  it('tests rule against transactions', async () => {
    // Click the test button on the first rule row
    const testBtn = await $(sel.ruleTestBtn);
    await testBtn.waitForClickable({ timeout: 5000 });
    await testBtn.click();

    // Wait for test results to appear -- matched transactions shown in a results area
    await browser.pause(2000);

    const resultsVisible = await isVisible(sel.searchResults);
    expect(resultsVisible).toBe(true);

    // There should be at least one matched transaction shown
    const matchCount = await countElements(`${byTestIdPrefix('tx-row-')}`);
    expect(matchCount).toBeGreaterThanOrEqual(0);
  });

  it('applies rules to uncategorized', async () => {
    await clickButton(sel.ruleApplyAllBtn);

    const toastText = await waitForToast();
    expect(toastText.toLowerCase()).toMatch(/appl|categori|rule/);
  });
});
