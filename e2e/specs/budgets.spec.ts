import { browser } from '@wdio/globals';
import { sel, byTestIdPrefix } from '../helpers/selectors';
import {
  waitForAppReady,
  navigateTo,
  fillInput,
  clickButton,
  isVisible,
  waitForToast,
  countElements,
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

    // Tab out or click elsewhere to trigger save
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
    await clickButton(sel.budgetCopyBtn);
    await browser.pause(1000);

    // Verify at least one budget amount input now has a value
    const budgetInput = await $(sel.budgetAmountInput);
    await budgetInput.waitForDisplayed({ timeout: 5000 });
    const copiedValue = await budgetInput.getValue();
    expect(Number(copiedValue)).toBeGreaterThan(0);
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
});
