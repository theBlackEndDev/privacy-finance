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

describe('Accounts', () => {
  before(async () => {
    await waitForAppReady();
    await navigateTo('accounts');
  });

  it('shows the Checking account from onboarding', async () => {
    const count = await countElements(byTestIdPrefix('account-row-'));
    expect(count).toBeGreaterThanOrEqual(1);

    // Verify Checking account exists
    const rows = await $$(byTestIdPrefix('account-row-'));
    let found = false;
    for (const row of rows) {
      const text = await row.getText();
      if (text.includes('Checking')) { found = true; break; }
    }
    expect(found).toBe(true);
  });

  it('creates a new savings account', async () => {
    const initialCount = await countElements(byTestIdPrefix('account-row-'));

    // Click the "+ Add Account" button in the header
    const addBtn = await $('button*=Add Account');
    await addBtn.waitForClickable({ timeout: 5000 });
    await addBtn.click();
    await browser.pause(500);

    await fillInput(sel.accountNameInput, 'E2E Test Account');
    await selectOption(sel.accountTypeSelect, 'Savings');
    await fillInput(sel.accountBalanceInput, '5000');
    await clickButton(sel.createAccountBtn);
    await browser.pause(500);

    const updatedCount = await countElements(byTestIdPrefix('account-row-'));
    expect(updatedCount).toBe(initialCount + 1);
  });

  it('edits account name', async () => {
    // Find the E2E Test Account row and click the Edit button WITHIN the row
    // (clicking the row itself navigates to account detail)
    const rows = await $$(byTestIdPrefix('account-row-'));
    for (const row of rows) {
      const text = await row.getText();
      if (text.includes('E2E Test Account')) {
        // Click the Edit button inside this row (stops propagation, won't navigate)
        const editBtn = await row.$('button*=Edit');
        await editBtn.waitForClickable({ timeout: 5000 });
        await editBtn.click();
        break;
      }
    }
    await browser.pause(500);

    // The AccountForm modal should now be open
    await fillInput(sel.accountNameInput, 'Renamed E2E Account');
    await clickButton(sel.createAccountBtn);
    await browser.pause(500);

    // Verify rename
    const updatedRows = await $$(byTestIdPrefix('account-row-'));
    let found = false;
    for (const row of updatedRows) {
      const text = await row.getText();
      if (text.includes('Renamed E2E Account')) { found = true; break; }
    }
    expect(found).toBe(true);
  });

  it('deletes account', async () => {
    const initialCount = await countElements(byTestIdPrefix('account-row-'));

    // Find the renamed account row and click the Delete button WITHIN the row
    const rows = await $$(byTestIdPrefix('account-row-'));
    for (const row of rows) {
      const text = await row.getText();
      if (text.includes('Renamed E2E Account')) {
        const deleteBtn = await row.$('button*=Delete');
        await deleteBtn.waitForClickable({ timeout: 5000 });
        await deleteBtn.click();
        break;
      }
    }
    await browser.pause(300);

    // Confirm the deletion in the ConfirmDialog
    await confirmAction();
    await browser.pause(500);

    const updatedCount = await countElements(byTestIdPrefix('account-row-'));
    expect(updatedCount).toBe(initialCount - 1);
  });

  it('updates account list balance after a transaction', async () => {
    async function readAccountBalanceCents(accountName: string): Promise<number> {
      const rows = await $$(byTestIdPrefix('account-row-'));
      for (const row of rows) {
        const text = await row.getText();
        if (text.includes(accountName)) {
          const match = text.match(/-?\$[\d,]+\.\d{2}/);
          if (!match) throw new Error(`Could not find balance in "${text}"`);
          const cents = Math.round(parseFloat(match[0].replace(/[^0-9.-]/g, '')) * 100);
          if (Number.isNaN(cents)) throw new Error(`Could not parse balance from "${match[0]}"`);
          return cents;
        }
      }
      throw new Error(`No account row for ${accountName}`);
    }

    const beforeCents = await readAccountBalanceCents('Checking');

    await navigateTo('transactions');
    const kindGroup = await $(sel.txKindSelect);
    await kindGroup.waitForDisplayed({ timeout: 5000 });
    const expenseBtn = await kindGroup.$('button*=\u2212');
    if (await expenseBtn.isExisting()) {
      await expenseBtn.click();
    }
    await fillInput(sel.txPayeeInput, 'Account Balance Check');
    await fillInput(sel.txAmountInput, '25.00');
    await selectOption(sel.txCategorySelect, 'Food');
    await clickButton(sel.txSubmitBtn);
    await browser.pause(500);

    await navigateTo('accounts');
    await browser.waitUntil(async () => {
      return (await readAccountBalanceCents('Checking')) !== beforeCents;
    }, { timeout: 5000, timeoutMsg: 'Accounts list balance did not update after transaction' });
    const afterCents = await readAccountBalanceCents('Checking');
    expect(afterCents - beforeCents).toBe(-2500);

    // Clean up the transaction so subsequent specs start with a clean ledger
    await navigateTo('transactions');
    const firstRow = await $(byTestIdPrefix('tx-row-'));
    await firstRow.waitForClickable({ timeout: 5000 });
    await firstRow.click();
    const editModal = await $(sel.txEditModal);
    await editModal.waitForDisplayed({ timeout: 5000 });
    await clickButton(sel.txEditDelete);
    await confirmAction();
    await browser.pause(500);

    await navigateTo('accounts');
    await browser.waitUntil(async () => {
      return (await readAccountBalanceCents('Checking')) === beforeCents;
    }, { timeout: 5000, timeoutMsg: 'Accounts list balance did not restore after deleting transaction' });
  });

  it('views account detail by clicking row', async () => {
    // After delete, there should still be the Checking account
    const rows = await $$(byTestIdPrefix('account-row-'));
    expect(rows.length).toBeGreaterThanOrEqual(1);

    // Click the row itself (not an action button) to navigate to detail
    await rows[0].click();
    await browser.pause(500);
    const mainContent = await $(sel.mainContent);
    expect(await mainContent.isDisplayed()).toBe(true);
  });
});
