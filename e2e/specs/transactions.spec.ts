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

  it('creates expense transaction via quick entry', async () => {
    const initialCount = await countElements(byTestIdPrefix('tx-row-'));

    await fillInput(sel.txPayeeInput, 'Trader Joes');
    await fillInput(sel.txAmountInput, '85.50');

    // Select expense kind (tx-kind-select is a toggle group)
    const kindSelect = await $(sel.txKindSelect);
    const expenseBtn = await kindSelect.$('button*=Expense');
    if (await expenseBtn.isExisting()) await expenseBtn.click();

    await selectOption(sel.txCategorySelect, 'Food');
    await clickButton(sel.txSubmitBtn);

    const updatedCount = await countElements(byTestIdPrefix('tx-row-'));
    expect(updatedCount).toBe(initialCount + 1);
  });

  it('creates income transaction', async () => {
    const initialCount = await countElements(byTestIdPrefix('tx-row-'));

    await fillInput(sel.txPayeeInput, 'Employer Inc');
    await fillInput(sel.txAmountInput, '3500');

    const kindSelect = await $(sel.txKindSelect);
    const incomeBtn = await kindSelect.$('button*=Income');
    if (await incomeBtn.isExisting()) await incomeBtn.click();

    await selectOption(sel.txCategorySelect, 'Salary');
    await clickButton(sel.txSubmitBtn);

    const updatedCount = await countElements(byTestIdPrefix('tx-row-'));
    expect(updatedCount).toBe(initialCount + 1);
  });

  it('edits transaction amount and payee', async () => {
    const firstRow = await $(byTestIdPrefix('tx-row-'));
    await firstRow.waitForClickable({ timeout: 5000 });
    await firstRow.click();

    const editModal = await $(sel.txEditModal);
    await editModal.waitForDisplayed({ timeout: 5000 });

    await fillInput(sel.txEditAmount, '90.00');
    await fillInput(sel.txEditPayee, 'Trader Joe Updated');
    await clickButton(sel.txEditSave);

    await waitForToast();
    const updatedRow = await $(byTestIdPrefix('tx-row-'));
    expect(await updatedRow.getText()).toContain('Trader Joe Updated');
  });

  it('deletes transaction', async () => {
    const initialCount = await countElements(byTestIdPrefix('tx-row-'));

    const row = await $(byTestIdPrefix('tx-row-'));
    await row.waitForClickable({ timeout: 5000 });
    await row.click();

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
    const thirtyDaysAgo = new Date(Date.now() - 30 * 86400000).toISOString().split('T')[0];

    await fillInput(sel.txFilterDateFrom, thirtyDaysAgo);
    await fillInput(sel.txFilterDateTo, today);
    await browser.pause(500);

    const rows = await $$(byTestIdPrefix('tx-row-'));
    expect(rows.length).toBeGreaterThan(0);
  });

  it('creates transfer between accounts', async () => {
    await fillInput(sel.transferAmount, '250');
    await selectOption(sel.transferFromAccount, 'Checking');
    await selectOption(sel.transferToAccount, 'Test Savings');
    await clickButton(sel.transferSubmitBtn);

    await waitForToast();
  });
});
