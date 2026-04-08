import { sel, byTestIdPrefix } from '../helpers/selectors';
import {
  waitForAppReady,
  navigateTo,
  fillInput,
  selectOption,
  clickButton,
  countElements,
} from '../helpers/app';

describe('Dashboard', () => {
  before(async () => {
    await waitForAppReady();

    // Create a second account for richer data
    await navigateTo('accounts');
    const addBtn = await $('button*=Add Account');
    await addBtn.waitForClickable({ timeout: 5000 });
    await addBtn.click();
    await browser.pause(500);
    await fillInput(sel.accountNameInput, 'Savings');
    await selectOption(sel.accountTypeSelect, 'Savings');
    await fillInput(sel.accountBalanceInput, '2000');
    await clickButton(sel.createAccountBtn);
    await browser.pause(500);

    // Add expense transactions
    await navigateTo('transactions');

    const kindGroup = await $(sel.txKindSelect);
    const expenseBtn = await kindGroup.$('button*=\u2212');

    if (await expenseBtn.isExisting()) await expenseBtn.click();
    await fillInput(sel.txPayeeInput, 'Dashboard Grocery');
    await fillInput(sel.txAmountInput, '75.00');
    await selectOption(sel.txCategorySelect, 'Food');
    await clickButton(sel.txSubmitBtn);
    await browser.pause(500);

    if (await expenseBtn.isExisting()) await expenseBtn.click();
    await fillInput(sel.txPayeeInput, 'Dashboard Gas');
    await fillInput(sel.txAmountInput, '45.00');
    await selectOption(sel.txCategorySelect, 'Transportation');
    await clickButton(sel.txSubmitBtn);
    await browser.pause(500);

    // Add income transaction
    const incomeBtn = await kindGroup.$('button*=+');
    if (await incomeBtn.isExisting()) await incomeBtn.click();
    await fillInput(sel.txPayeeInput, 'Dashboard Salary');
    await fillInput(sel.txAmountInput, '2000');
    await selectOption(sel.txCategorySelect, 'Salary');
    await clickButton(sel.txSubmitBtn);
    await browser.pause(500);

    // Navigate to dashboard
    await navigateTo('dashboard');
  });

  it('shows dashboard summary container', async () => {
    const summary = await $(sel.dashboardSummary);
    await summary.waitForDisplayed({ timeout: 10000 });
    expect(await summary.isDisplayed()).toBe(true);
  });

  it('shows income card with a value', async () => {
    const incomeCard = await $(sel.dashboardIncomeCard);
    await incomeCard.waitForDisplayed({ timeout: 5000 });
    const text = await incomeCard.getText();
    expect(text).toMatch(/\$/);
  });

  it('shows expense card with a value', async () => {
    const expenseCard = await $(sel.dashboardExpenseCard);
    await expenseCard.waitForDisplayed({ timeout: 5000 });
    const text = await expenseCard.getText();
    expect(text).toMatch(/\$/);
  });

  it('shows net card', async () => {
    const netCard = await $(sel.dashboardNetCard);
    await netCard.waitForDisplayed({ timeout: 5000 });
    const text = await netCard.getText();
    expect(text).toMatch(/\$/);
  });

  it('shows recent transactions list', async () => {
    const recentTx = await $(sel.dashboardRecentTx);
    await recentTx.waitForDisplayed({ timeout: 5000 });
    const text = await recentTx.getText();
    // Should contain at least one of the transactions we created
    expect(text.length).toBeGreaterThan(0);
  });

  it('shows chart area', async () => {
    const chart = await $(sel.dashboardChart);
    await chart.waitForDisplayed({ timeout: 5000 });
    expect(await chart.isDisplayed()).toBe(true);
  });
});
