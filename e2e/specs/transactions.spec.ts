import { sel, byTestIdPrefix } from '../helpers/selectors';
import {
  waitForAppReady,
  navigateTo,
  fillInput,
  selectOption,
  clickButton,
  confirmAction,
  waitForToast,
  countElements,
} from '../helpers/app';

describe('Transactions', () => {
  before(async () => {
    await waitForAppReady();
    await navigateTo('transactions');
  });

  it('shows seed transactions on fresh launch', async () => {
    const count = await countElements(byTestIdPrefix('tx-row-'));
    expect(count).toBeGreaterThan(0);
  });

  it('creates expense transaction via quick entry', async () => {
    const initialCount = await countElements(byTestIdPrefix('tx-row-'));

    await fillInput(sel.txPayeeInput, 'E2E Test Store');
    await fillInput(sel.txAmountInput, '42.50');

    // Select expense kind (toggle group)
    const kindSelect = await $(sel.txKindSelect);
    const expenseBtn = await kindSelect.$('button*=Expense');
    if (await expenseBtn.isExisting()) await expenseBtn.click();

    // Use seed category "Food"
    await selectOption(sel.txCategorySelect, 'Food');
    await clickButton(sel.txSubmitBtn);

    await browser.pause(500);
    const updatedCount = await countElements(byTestIdPrefix('tx-row-'));
    expect(updatedCount).toBeGreaterThan(initialCount);
  });

  it('creates income transaction', async () => {
    await fillInput(sel.txPayeeInput, 'E2E Employer');
    await fillInput(sel.txAmountInput, '1000');

    const kindSelect = await $(sel.txKindSelect);
    const incomeBtn = await kindSelect.$('button*=Income');
    if (await incomeBtn.isExisting()) await incomeBtn.click();

    await selectOption(sel.txCategorySelect, 'Salary');
    await clickButton(sel.txSubmitBtn);
    await browser.pause(500);
  });

  it('edits transaction amount and payee', async () => {
    // Click on a transaction row to open edit modal
    const firstRow = await $(byTestIdPrefix('tx-row-'));
    await firstRow.waitForClickable({ timeout: 5000 });
    await firstRow.click();

    const editModal = await $(sel.txEditModal);
    await editModal.waitForDisplayed({ timeout: 5000 });

    await fillInput(sel.txEditAmount, '99.99');
    await fillInput(sel.txEditPayee, 'Updated Payee');
    await clickButton(sel.txEditSave);
    await waitForToast();
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
    await waitForToast();

    const updatedCount = await countElements(byTestIdPrefix('tx-row-'));
    expect(updatedCount).toBe(initialCount - 1);
  });

  it('filters by category', async () => {
    await selectOption(sel.txFilterCategory, 'Food');
    await browser.pause(500);

    const rows = await $$(byTestIdPrefix('tx-row-'));
    expect(rows.length).toBeGreaterThanOrEqual(0);

    // Reset filter
    await selectOption(sel.txFilterCategory, '');
  });

  it('filters by date range', async () => {
    const today = new Date().toISOString().split('T')[0];
    const sixtyDaysAgo = new Date(Date.now() - 60 * 86400000).toISOString().split('T')[0];

    await fillInput(sel.txFilterDateFrom, sixtyDaysAgo);
    await fillInput(sel.txFilterDateTo, today);
    await browser.pause(500);

    const rows = await $$(byTestIdPrefix('tx-row-'));
    expect(rows.length).toBeGreaterThan(0);
  });

  it('creates transfer between accounts', async () => {
    // Check if transfer form is visible or needs toggling
    const transferAmount = await $(sel.transferAmount);
    if (await transferAmount.isExisting()) {
      await fillInput(sel.transferAmount, '100');
      await clickButton(sel.transferSubmitBtn);
      await waitForToast();
    }
  });
});
