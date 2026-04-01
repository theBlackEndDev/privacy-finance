import { sel, byTestId } from '../helpers/selectors';
import {
  waitForAppReady,
  navigateTo,
  clickButton,
  isVisible,
} from '../helpers/app';

describe('Reports', () => {
  before(async () => {
    await waitForAppReady();
    await navigateTo('reports');
  });

  it('views income/expense report via tab', async () => {
    const tab = await $('[data-testid="report-tab-income-expense"]');
    await tab.waitForClickable({ timeout: 5000 });
    await tab.click();
    await browser.pause(1000);

    const chart = await $(sel.reportChart);
    await chart.waitForDisplayed({ timeout: 10000 });
    expect(await chart.isDisplayed()).toBe(true);
  });

  it('views category spending report via tab', async () => {
    const tab = await $('[data-testid="report-tab-categories"]');
    await tab.waitForClickable({ timeout: 5000 });
    await tab.click();
    await browser.pause(1000);

    const chart = await $(sel.reportChart);
    await chart.waitForDisplayed({ timeout: 10000 });
    expect(await chart.isDisplayed()).toBe(true);
  });

  it('views net worth report via tab', async () => {
    const tab = await $('[data-testid="report-tab-net-worth"]');
    await tab.waitForClickable({ timeout: 5000 });
    await tab.click();
    await browser.pause(1000);

    const chart = await $(sel.reportChart);
    await chart.waitForDisplayed({ timeout: 10000 });
    expect(await chart.isDisplayed()).toBe(true);
  });
});
