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

describe('Accounts', () => {
  before(async () => {
    await waitForAppReady();
    await navigateTo('accounts');
  });

  it('creates a new savings account', async () => {
    const initialCount = await countElements(byTestIdPrefix('account-row-'));

    const addBtn = await $(sel.createAccountBtn);
    if (!(await addBtn.isExisting())) {
      const altBtn = await $('button*=Add Account');
      await altBtn.click();
    } else {
      await addBtn.waitForClickable({ timeout: 5000 });
      await addBtn.click();
    }

    await fillInput(sel.accountNameInput, 'Test Savings');
    await selectOption(sel.accountTypeSelect, 'Savings');
    await fillInput(sel.accountBalanceInput, '5000');
    await clickButton(sel.createAccountBtn);

    await waitForToast();
    const updatedCount = await countElements(byTestIdPrefix('account-row-'));
    expect(updatedCount).toBe(initialCount + 1);
  });

  it('edits account name', async () => {
    const count = await countElements(byTestIdPrefix('account-row-'));
    expect(count).toBeGreaterThan(0);

    const lastRow = await $(byTestIdPrefix('account-row-'));
    await lastRow.waitForClickable({ timeout: 5000 });
    await lastRow.click();
    await browser.pause(300);

    await fillInput(sel.accountNameInput, 'Renamed Savings');
    await clickButton(sel.createAccountBtn);
    await waitForToast();
  });

  it('deletes account', async () => {
    const initialCount = await countElements(byTestIdPrefix('account-row-'));

    const row = await $(byTestIdPrefix('account-row-'));
    await row.waitForClickable({ timeout: 5000 });
    await row.click();
    await browser.pause(300);

    await clickButton(sel.deleteBtn);
    await confirmAction();

    await waitForToast();
    const updatedCount = await countElements(byTestIdPrefix('account-row-'));
    expect(updatedCount).toBe(initialCount - 1);
  });

  it('views account detail page', async () => {
    const count = await countElements('[data-testid^="account-row-"]');
    if (count > 0) {
      const firstAccount = await $('[data-testid^="account-row-"]');
      await firstAccount.click();
      await browser.pause(500);
      const mainContent = await $(sel.mainContent);
      expect(await mainContent.isDisplayed()).toBe(true);
    }
  });
});
