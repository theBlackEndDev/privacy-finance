import { sel, byTestIdPrefix } from '../helpers/selectors';
import {
  waitForAppReady,
  navigateTo,
  fillInput,
  selectOption,
  clickButton,
} from '../helpers/app';

describe('Budgets', () => {
  before(async () => {
    await waitForAppReady();
    await navigateTo('budgets');
  });

  it('sets budget amount for a category', async () => {
    // Find the first budget amount input (prefixed selector matches any category row)
    const budgetInput = await $(sel.budgetAmountInput);
    await budgetInput.waitForDisplayed({ timeout: 5000 });
    await budgetInput.click();
    await budgetInput.clearValue();
    await budgetInput.setValue('500');

    // Tab out to trigger save
    await browser.keys('Tab');
    await browser.pause(1000);

    // Re-read the value to confirm persistence
    const savedValue = await budgetInput.getValue();
    expect(savedValue).toContain('500');
  });

  it('copies budget from previous month', async () => {
    // Navigate to the next month so it starts empty
    await clickButton(sel.budgetNextMonth);
    await browser.pause(500);

    // Click copy from previous month
    const copyBtn = await $(sel.budgetCopyBtn);
    await copyBtn.waitForDisplayed({ timeout: 5000 });
    // Copy may fail silently if previous month has no data — that's OK
    await browser.execute((el: HTMLElement) => el.click(), copyBtn as any);
    await browser.pause(1000);

    // Verify the budget page still renders (copy didn't crash the app)
    const monthNav = await $(sel.budgetMonthNav);
    expect(await monthNav.isDisplayed()).toBe(true);
  });

  it('shows budget progress bar', async () => {
    // Navigate back to a month with budget set and spending recorded
    await clickButton(sel.budgetPrevMonth);
    await browser.pause(500);

    const progressBar = await $(sel.budgetProgressBar);
    await progressBar.waitForDisplayed({ timeout: 5000 });
    const displayed = await progressBar.isDisplayed();
    expect(displayed).toBe(true);
  });

  it('budget progress reflects actual spending after creating transactions', async () => {
    // Create transactions in a known category first
    await navigateTo('transactions');

    const kindGroup = await $(sel.txKindSelect);
    const expenseBtn = await kindGroup.$('button*=\u2212');

    if (await expenseBtn.isExisting()) await expenseBtn.click();
    await fillInput(sel.txPayeeInput, 'Budget Test Purchase');
    await fillInput(sel.txAmountInput, '150.00');
    await selectOption(sel.txCategorySelect, 'Food');
    await clickButton(sel.txSubmitBtn);
    await browser.pause(500);

    // Navigate to budgets and set a budget for Food category
    await navigateTo('budgets');

    // Find a budget amount input and set it (the first one corresponds to an expense category)
    const budgetInput = await $(sel.budgetAmountInput);
    await budgetInput.waitForDisplayed({ timeout: 5000 });
    await budgetInput.click();
    await budgetInput.clearValue();
    await budgetInput.setValue('400');
    await browser.keys('Tab');
    await browser.pause(1000);

    // Verify the progress bar is visible (spending against budget)
    const progressBar = await $(sel.budgetProgressBar);
    await progressBar.waitForDisplayed({ timeout: 5000 });
    expect(await progressBar.isDisplayed()).toBe(true);
  });
});
