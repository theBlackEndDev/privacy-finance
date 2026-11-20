import { sel, byTestIdPrefix } from '../helpers/selectors';
import {
  waitForAppReady,
  navigateTo,
  fillInput,
  selectOption,
  clickButton,
  closeModal,
  countElements,
} from '../helpers/app';

describe('Transfers', () => {
  before(async () => {
    await waitForAppReady();

    // Ensure at least 2 accounts exist
    await navigateTo('accounts');
    const initialCount = await countElements(byTestIdPrefix('account-row-'));

    if (initialCount < 2) {
      const addBtn = await $('button*=Add Account');
      await addBtn.waitForClickable({ timeout: 5000 });
      await addBtn.click();
      await browser.pause(500);

      await fillInput(sel.accountNameInput, 'Transfer Target');
      await selectOption(sel.accountTypeSelect, 'Savings');
      await fillInput(sel.accountBalanceInput, '3000');
      await clickButton(sel.createAccountBtn);
      await browser.pause(500);
    }
  });

  it('opens transfer form from transactions page', async () => {
    await navigateTo('transactions');

    const transferBtn = await $('button*=Transfer');
    await transferBtn.waitForClickable({ timeout: 5000 });
    await transferBtn.click();
    await browser.pause(500);

    const fromSelect = await $(sel.transferFromAccount);
    expect(await fromSelect.isDisplayed()).toBe(true);

    const toSelect = await $(sel.transferToAccount);
    expect(await toSelect.isDisplayed()).toBe(true);

    const amountInput = await $(sel.transferAmount);
    expect(await amountInput.isDisplayed()).toBe(true);
  });

  it('submits a $250 transfer between accounts', async () => {
    // Select from and to accounts (pick whatever is available)
    const fromSelect = await $(sel.transferFromAccount);
    await fromSelect.waitForDisplayed({ timeout: 5000 });

    // Select the first available from-account option (skip empty/placeholder)
    await browser.execute((fromSel: string, toSel: string) => {
      const from = document.querySelector(fromSel) as HTMLSelectElement;
      const to = document.querySelector(toSel) as HTMLSelectElement;
      if (from && from.options.length > 1) {
        from.value = from.options[1].value;
        from.dispatchEvent(new Event('change', { bubbles: true }));
      }
      if (to && to.options.length > 2) {
        to.value = to.options[2].value;
        to.dispatchEvent(new Event('change', { bubbles: true }));
      } else if (to && to.options.length > 1) {
        to.value = to.options[1].value;
        to.dispatchEvent(new Event('change', { bubbles: true }));
      }
    }, sel.transferFromAccount, sel.transferToAccount);
    await browser.pause(300);

    await fillInput(sel.transferAmount, '250');
    await clickButton(sel.transferSubmitBtn);
    await browser.pause(1000);

    // After successful transfer, verify main content is still displayed
    const mainContent = await $(sel.mainContent);
    expect(await mainContent.isDisplayed()).toBe(true);
  });

  it('transfer appears in transaction list', async () => {
    // After transfer, we should be on the transactions page
    // The transfer creates paired transactions — at least one should appear
    const rows = await $$(byTestIdPrefix('tx-row-'));
    expect(rows.length).toBeGreaterThanOrEqual(1);

    // Look for a transfer-related entry
    let foundTransfer = false;
    for (const row of rows) {
      const text = await row.getText();
      if (text.toLowerCase().includes('transfer') || text.includes('250')) {
        foundTransfer = true;
        break;
      }
    }
    expect(foundTransfer).toBe(true);
  });

  it('accounts page reflects transfer', async () => {
    await navigateTo('accounts');
    await browser.pause(500);

    // Both accounts should still exist and show balances
    const rows = await $$(byTestIdPrefix('account-row-'));
    expect(rows.length).toBeGreaterThanOrEqual(2);

    // Each account row should show a dollar amount
    for (const row of rows) {
      const text = await row.getText();
      expect(text).toMatch(/\$/);
    }
  });
});
