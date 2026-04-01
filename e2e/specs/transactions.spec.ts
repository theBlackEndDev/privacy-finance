import { sel, byTestIdPrefix } from '../helpers/selectors';
import {
  waitForAppReady,
  navigateTo,
  fillInput,
  selectOption,
  clickButton,
  confirmAction,
  countElements,
} from '../helpers/app';

describe('Transactions', () => {
  before(async () => {
    await waitForAppReady();
    await navigateTo('transactions');
  });

  it('loads transactions page', async () => {
    const mainContent = await $(sel.mainContent);
    expect(await mainContent.isDisplayed()).toBe(true);
  });

  it('creates expense transaction via quick entry', async () => {
    // Select expense kind: click the button with text content minus sign
    const kindGroup = await $(sel.txKindSelect);
    await kindGroup.waitForDisplayed({ timeout: 5000 });
    const expenseBtn = await kindGroup.$('button*=\u2212');
    if (await expenseBtn.isExisting()) {
      await expenseBtn.click();
    }

    await fillInput(sel.txPayeeInput, 'E2E Test Store');
    await fillInput(sel.txAmountInput, '42.50');
    await selectOption(sel.txCategorySelect, 'Food');
    await clickButton(sel.txSubmitBtn);
    await browser.pause(500);

    const count = await countElements(byTestIdPrefix('tx-row-'));
    expect(count).toBeGreaterThanOrEqual(1);
  });

  it('creates income transaction', async () => {
    // Select income kind
    const kindGroup = await $(sel.txKindSelect);
    const incomeBtn = await kindGroup.$('button*=+');
    if (await incomeBtn.isExisting()) {
      await incomeBtn.click();
    }

    await fillInput(sel.txPayeeInput, 'E2E Employer');
    await fillInput(sel.txAmountInput, '1000');
    await selectOption(sel.txCategorySelect, 'Salary');
    await clickButton(sel.txSubmitBtn);
    await browser.pause(500);

    const count = await countElements(byTestIdPrefix('tx-row-'));
    expect(count).toBeGreaterThanOrEqual(2);
  });

  it('edits transaction amount and payee', async () => {
    // Click first transaction row to open edit modal
    const firstRow = await $(byTestIdPrefix('tx-row-'));
    await firstRow.waitForClickable({ timeout: 5000 });
    await firstRow.click();

    const editModal = await $(sel.txEditModal);
    await editModal.waitForDisplayed({ timeout: 5000 });

    await fillInput(sel.txEditAmount, '99.99');
    await fillInput(sel.txEditPayee, 'Updated Payee');
    await clickButton(sel.txEditSave);
    await browser.pause(500);
  });

  it('deletes transaction', async () => {
    const initialCount = await countElements(byTestIdPrefix('tx-row-'));

    const firstRow = await $(byTestIdPrefix('tx-row-'));
    await firstRow.waitForClickable({ timeout: 5000 });
    await firstRow.click();

    const editModal = await $(sel.txEditModal);
    await editModal.waitForDisplayed({ timeout: 5000 });

    await clickButton(sel.txEditDelete);
    await confirmAction();
    await browser.pause(500);

    const updatedCount = await countElements(byTestIdPrefix('tx-row-'));
    expect(updatedCount).toBe(initialCount - 1);
  });

  it('creates more transactions for other specs', async () => {
    // Create a few transactions that subsequent specs (search, tags, etc.) can use
    const kindGroup = await $(sel.txKindSelect);

    // Expense 1
    const expenseBtn = await kindGroup.$('button*=\u2212');
    if (await expenseBtn.isExisting()) await expenseBtn.click();
    await fillInput(sel.txPayeeInput, 'Grocery Mart');
    await fillInput(sel.txAmountInput, '55.00');
    await selectOption(sel.txCategorySelect, 'Food');
    await clickButton(sel.txSubmitBtn);
    await browser.pause(500);

    // Expense 2
    if (await expenseBtn.isExisting()) await expenseBtn.click();
    await fillInput(sel.txPayeeInput, 'Gas Station');
    await fillInput(sel.txAmountInput, '40.00');
    await selectOption(sel.txCategorySelect, 'Transportation');
    await clickButton(sel.txSubmitBtn);
    await browser.pause(500);

    // Income
    const incomeBtn = await kindGroup.$('button*=+');
    if (await incomeBtn.isExisting()) await incomeBtn.click();
    await fillInput(sel.txPayeeInput, 'Monthly Salary');
    await fillInput(sel.txAmountInput, '3000');
    await selectOption(sel.txCategorySelect, 'Salary');
    await clickButton(sel.txSubmitBtn);
    await browser.pause(500);

    const count = await countElements(byTestIdPrefix('tx-row-'));
    expect(count).toBeGreaterThanOrEqual(3);
  });

  it('filters by category', async () => {
    await selectOption(sel.txFilterCategory, 'Food');
    await browser.pause(500);

    const rows = await $$(byTestIdPrefix('tx-row-'));
    expect(rows.length).toBeGreaterThanOrEqual(1);

    // Reset filter
    await selectOption(sel.txFilterCategory, 'All');
    await browser.pause(500);
  });

  it('filters by date range', async () => {
    // Use browser.execute to set date input values directly via JS,
    // which avoids locale-dependent date input format issues on Linux.
    const today = new Date().toISOString().split('T')[0]; // YYYY-MM-DD
    const sixtyDaysAgo = new Date(Date.now() - 60 * 86400000).toISOString().split('T')[0];

    const dateFrom = await $(sel.txFilterDateFrom);
    await dateFrom.waitForDisplayed({ timeout: 5000 });

    // Set date values via JavaScript to avoid platform-specific input format issues
    await browser.execute((fromSel: string, toSel: string, fromVal: string, toVal: string) => {
      const fromInput = document.querySelector(fromSel) as HTMLInputElement;
      const toInput = document.querySelector(toSel) as HTMLInputElement;
      if (fromInput) {
        const nativeInputValueSetter = Object.getOwnPropertyDescriptor(
          window.HTMLInputElement.prototype, 'value'
        )?.set;
        nativeInputValueSetter?.call(fromInput, fromVal);
        fromInput.dispatchEvent(new Event('input', { bubbles: true }));
        fromInput.dispatchEvent(new Event('change', { bubbles: true }));
      }
      if (toInput) {
        const nativeInputValueSetter = Object.getOwnPropertyDescriptor(
          window.HTMLInputElement.prototype, 'value'
        )?.set;
        nativeInputValueSetter?.call(toInput, toVal);
        toInput.dispatchEvent(new Event('input', { bubbles: true }));
        toInput.dispatchEvent(new Event('change', { bubbles: true }));
      }
    }, sel.txFilterDateFrom, sel.txFilterDateTo, sixtyDaysAgo, today);

    await browser.pause(500);

    const rows = await $$(byTestIdPrefix('tx-row-'));
    expect(rows.length).toBeGreaterThan(0);
  });

  it('opens transfer modal', async () => {
    // Transfer opens via a button on the transactions page
    const transferBtn = await $('button*=Transfer');
    if (await transferBtn.isExisting()) {
      await transferBtn.click();
      await browser.pause(500);

      // Verify modal opened with transfer fields
      const transferAmount = await $(sel.transferAmount);
      expect(await transferAmount.isDisplayed()).toBe(true);

      // Close the modal without submitting (only one account exists)
      const closeBtn = await $(sel.modalClose);
      if (await closeBtn.isDisplayed()) {
        await closeBtn.click();
        await browser.pause(300);
      }
    }
  });
});
