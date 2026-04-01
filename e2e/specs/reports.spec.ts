import { browser } from '@wdio/globals';
import { sel } from '../helpers/selectors';
import {
  waitForAppReady,
  navigateTo,
  selectOption,
  isVisible,
} from '../helpers/app';

describe('Reports', () => {
  before(async () => {
    await waitForAppReady();
    await navigateTo('reports');
  });

  it('views income/expense report', async () => {
    await selectOption(sel.reportTypeSelect, 'Income vs Expense');
    await browser.pause(1000);

    const chart = await $(sel.reportIncomeExpense);
    await chart.waitForDisplayed({ timeout: 10000 });
    expect(await chart.isDisplayed()).toBe(true);
  });

  it('views category spending report', async () => {
    await selectOption(sel.reportTypeSelect, 'Category Spending');
    await browser.pause(1000);

    const breakdown = await $(sel.reportCategorySpending);
    await breakdown.waitForDisplayed({ timeout: 10000 });
    expect(await breakdown.isDisplayed()).toBe(true);
  });

  it('views net worth report', async () => {
    await selectOption(sel.reportTypeSelect, 'Net Worth');
    await browser.pause(1000);

    const chart = await $(sel.reportNetWorth);
    await chart.waitForDisplayed({ timeout: 10000 });
    expect(await chart.isDisplayed()).toBe(true);
  });
});
