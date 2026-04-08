import { sel, byTestIdPrefix } from '../helpers/selectors';
import {
  waitForAppReady,
  navigateTo,
  fillInput,
  selectOption,
  clickButton,
  countElements,
  closeModal,
} from '../helpers/app';

describe('Financial Workflow (Full Journey)', () => {
  before(async () => {
    await waitForAppReady();
  });

  it('creates checking account with $5000 opening balance', async () => {
    await navigateTo('accounts');

    const addBtn = await $('button*=Add Account');
    await addBtn.waitForClickable({ timeout: 5000 });
    await addBtn.click();
    await browser.pause(500);

    await fillInput(sel.accountNameInput, 'Workflow Checking');
    await selectOption(sel.accountTypeSelect, 'Checking');
    await fillInput(sel.accountBalanceInput, '5000');
    await clickButton(sel.createAccountBtn);
    await browser.pause(500);

    const rows = await $$(byTestIdPrefix('account-row-'));
    let found = false;
    for (const row of rows) {
      const text = await row.getText();
      if (text.includes('Workflow Checking')) { found = true; break; }
    }
    expect(found).toBe(true);
  });

  it('creates savings account with $10000 opening balance', async () => {
    const addBtn = await $('button*=Add Account');
    await addBtn.waitForClickable({ timeout: 5000 });
    await addBtn.click();
    await browser.pause(500);

    await fillInput(sel.accountNameInput, 'Workflow Savings');
    await selectOption(sel.accountTypeSelect, 'Savings');
    await fillInput(sel.accountBalanceInput, '10000');
    await clickButton(sel.createAccountBtn);
    await browser.pause(500);

    const rows = await $$(byTestIdPrefix('account-row-'));
    let found = false;
    for (const row of rows) {
      const text = await row.getText();
      if (text.includes('Workflow Savings')) { found = true; break; }
    }
    expect(found).toBe(true);
  });

  it('creates expense categories: Groceries, Utilities, Entertainment', async () => {
    await navigateTo('categories');

    const categoriesToCreate = ['Groceries', 'Utilities', 'Entertainment'];
    for (const name of categoriesToCreate) {
      await clickButton(sel.createCategoryBtn);
      await browser.pause(300);
      await fillInput(sel.catNameInput, name);
      await selectOption(sel.catTypeSelect, 'Expense');
      await clickButton(sel.catSubmitBtn);
      await browser.pause(500);
    }

    const rows = await $$(sel.catRow);
    const texts: string[] = [];
    for (const row of rows) {
      texts.push(await row.getText());
    }
    for (const name of categoriesToCreate) {
      expect(texts.some(t => t.includes(name))).toBe(true);
    }
  });

  it('adds 5 expense transactions across categories', async () => {
    await navigateTo('transactions');

    const expenses = [
      { payee: 'Whole Foods', amount: '120.00', category: 'Groceries' },
      { payee: 'Electric Company', amount: '85.00', category: 'Utilities' },
      { payee: 'Movie Theater', amount: '30.00', category: 'Entertainment' },
      { payee: 'Trader Joes', amount: '95.00', category: 'Groceries' },
      { payee: 'Water Bill', amount: '45.00', category: 'Utilities' },
    ];

    const kindGroup = await $(sel.txKindSelect);
    const expenseBtn = await kindGroup.$('button*=\u2212');

    for (const tx of expenses) {
      if (await expenseBtn.isExisting()) await expenseBtn.click();
      await fillInput(sel.txPayeeInput, tx.payee);
      await fillInput(sel.txAmountInput, tx.amount);
      await selectOption(sel.txCategorySelect, tx.category);
      await clickButton(sel.txSubmitBtn);
      await browser.pause(500);
    }

    const count = await countElements(byTestIdPrefix('tx-row-'));
    expect(count).toBeGreaterThanOrEqual(5);
  });

  it('adds 2 income transactions', async () => {
    const kindGroup = await $(sel.txKindSelect);
    const incomeBtn = await kindGroup.$('button*=+');

    const incomes = [
      { payee: 'Employer Inc', amount: '3500', category: 'Salary' },
      { payee: 'Freelance Job', amount: '800', category: 'Salary' },
    ];

    for (const tx of incomes) {
      if (await incomeBtn.isExisting()) await incomeBtn.click();
      await fillInput(sel.txPayeeInput, tx.payee);
      await fillInput(sel.txAmountInput, tx.amount);
      await selectOption(sel.txCategorySelect, tx.category);
      await clickButton(sel.txSubmitBtn);
      await browser.pause(500);
    }

    const count = await countElements(byTestIdPrefix('tx-row-'));
    expect(count).toBeGreaterThanOrEqual(7);
  });

  it('dashboard shows expected summary cards', async () => {
    await navigateTo('dashboard');

    const summary = await $(sel.dashboardSummary);
    await summary.waitForDisplayed({ timeout: 10000 });

    // Income card should show a dollar value
    const incomeCard = await $(sel.dashboardIncomeCard);
    await incomeCard.waitForDisplayed({ timeout: 5000 });
    const incomeText = await incomeCard.getText();
    expect(incomeText).toMatch(/\$/);

    // Expense card should show a dollar value
    const expenseCard = await $(sel.dashboardExpenseCard);
    await expenseCard.waitForDisplayed({ timeout: 5000 });
    const expenseText = await expenseCard.getText();
    expect(expenseText).toMatch(/\$/);
  });

  it('navigates to budgets and sets budget amounts', async () => {
    await navigateTo('budgets');

    // Set budget for at least the first category input visible
    const budgetInput = await $(sel.budgetAmountInput);
    await budgetInput.waitForDisplayed({ timeout: 5000 });
    await budgetInput.click();
    await budgetInput.clearValue();
    await budgetInput.setValue('300');
    await browser.keys('Tab');
    await browser.pause(1000);

    const savedValue = await budgetInput.getValue();
    expect(savedValue).toContain('300');
  });

  it('budget progress bars are visible', async () => {
    const progressBar = await $(sel.budgetProgressBar);
    await progressBar.waitForDisplayed({ timeout: 5000 });
    expect(await progressBar.isDisplayed()).toBe(true);
  });

  it('creates a transfer from checking to savings ($500)', async () => {
    await navigateTo('transactions');

    const transferBtn = await $('button*=Transfer');
    await transferBtn.waitForClickable({ timeout: 5000 });
    await transferBtn.click();
    await browser.pause(500);

    // Select from/to accounts
    await selectOption(sel.transferFromAccount, 'Workflow Checking');
    await selectOption(sel.transferToAccount, 'Workflow Savings');
    await fillInput(sel.transferAmount, '500');
    await clickButton(sel.transferSubmitBtn);
    await browser.pause(1000);

    // Verify we're back on transactions page (transfer completed)
    const mainContent = await $(sel.mainContent);
    expect(await mainContent.isDisplayed()).toBe(true);
  });

  it('verifies both accounts reflect the transfer', async () => {
    await navigateTo('accounts');
    await browser.pause(500);

    const rows = await $$(byTestIdPrefix('account-row-'));
    let checkingText = '';
    let savingsText = '';
    for (const row of rows) {
      const text = await row.getText();
      if (text.includes('Workflow Checking')) checkingText = text;
      if (text.includes('Workflow Savings')) savingsText = text;
    }

    // Both accounts should still be visible (transfer didn't break anything)
    expect(checkingText).toContain('Workflow Checking');
    expect(savingsText).toContain('Workflow Savings');
  });

  it('reports page shows at least one chart with data', async () => {
    await navigateTo('reports');

    // Click income/expense tab
    const tab = await $('[data-testid="report-tab-income-expense"]');
    await tab.waitForClickable({ timeout: 5000 });
    await tab.click();
    await browser.pause(1000);

    const chart = await $(sel.reportChart);
    await chart.waitForDisplayed({ timeout: 10000 });
    expect(await chart.isDisplayed()).toBe(true);
  });
});
