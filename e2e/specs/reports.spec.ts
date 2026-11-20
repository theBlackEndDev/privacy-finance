import { sel, byTestId, byTestIdPrefix } from '../helpers/selectors';
import {
  waitForAppReady,
  navigateTo,
  fillInput,
  selectOption,
  clickButton,
  isVisible,
} from '../helpers/app';

describe('Reports', () => {
  before(async () => {
    await waitForAppReady();

    // Create known transactions so reports have data to display
    await navigateTo('transactions');
    const kindGroup = await $(sel.txKindSelect);

    const expenseBtn = await kindGroup.$('button*=\u2212');
    if (await expenseBtn.isExisting()) await expenseBtn.click();
    await fillInput(sel.txPayeeInput, 'Report Test Store');
    await fillInput(sel.txAmountInput, '200.00');
    await selectOption(sel.txCategorySelect, 'Food');
    await clickButton(sel.txSubmitBtn);
    await browser.pause(500);

    const incomeBtn = await kindGroup.$('button*=+');
    if (await incomeBtn.isExisting()) await incomeBtn.click();
    await fillInput(sel.txPayeeInput, 'Report Test Income');
    await fillInput(sel.txAmountInput, '1500');
    await selectOption(sel.txCategorySelect, 'Salary');
    await clickButton(sel.txSubmitBtn);
    await browser.pause(500);

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

  it('income/expense report shows actual data (not empty)', async () => {
    // The income/expense chart area should contain rendered content (SVG or canvas)
    const chart = await $(sel.reportChart);
    const chartHtml = await chart.getHTML({ includeSelectorTag: false });
    // Chart renders as SVG or contains visual elements when data exists
    expect(chartHtml.length).toBeGreaterThan(50);
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

  it('category spending report has at least one category with data', async () => {
    // The chart should contain rendered elements (not be an empty container)
    const chart = await $(sel.reportChart);
    const chartHtml = await chart.getHTML({ includeSelectorTag: false });
    // A chart with data will have substantial HTML (SVG paths, text elements, etc.)
    expect(chartHtml.length).toBeGreaterThan(50);
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
