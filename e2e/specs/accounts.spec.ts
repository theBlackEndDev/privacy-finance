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

  it('shows seed accounts on fresh launch', async () => {
    const count = await countElements(byTestIdPrefix('account-row-'));
    expect(count).toBeGreaterThanOrEqual(3); // Primary Checking, High-Yield Savings, Visa Rewards
  });

  it('creates a new savings account', async () => {
    const initialCount = await countElements(byTestIdPrefix('account-row-'));

    // Open create form — look for any add/create button
    const addBtn = await $('button*=Add Account');
    if (await addBtn.isExisting()) {
      await addBtn.click();
    } else {
      await clickButton(sel.createAccountBtn);
    }

    await fillInput(sel.accountNameInput, 'E2E Test Account');
    await selectOption(sel.accountTypeSelect, 'Savings');
    await fillInput(sel.accountBalanceInput, '5000');
    await clickButton(sel.createAccountBtn);

    await waitForToast();
    const updatedCount = await countElements(byTestIdPrefix('account-row-'));
    expect(updatedCount).toBe(initialCount + 1);
  });

  it('edits account name', async () => {
    // Find and click the E2E Test Account
    const rows = await $$(byTestIdPrefix('account-row-'));
    for (const row of rows) {
      const text = await row.getText();
      if (text.includes('E2E Test Account')) {
        await row.click();
        break;
      }
    }
    await browser.pause(500);

    await fillInput(sel.accountNameInput, 'Renamed E2E Account');
    await clickButton(sel.createAccountBtn);
    await waitForToast();
  });

  it('deletes account', async () => {
    const initialCount = await countElements(byTestIdPrefix('account-row-'));

    // Find and click the renamed account
    const rows = await $$(byTestIdPrefix('account-row-'));
    for (const row of rows) {
      const text = await row.getText();
      if (text.includes('Renamed E2E Account')) {
        await row.click();
        break;
      }
    }
    await browser.pause(500);

    await clickButton(sel.deleteBtn);
    await confirmAction();
    await waitForToast();

    const updatedCount = await countElements(byTestIdPrefix('account-row-'));
    expect(updatedCount).toBe(initialCount - 1);
  });

  it('views account detail page', async () => {
    const firstAccount = await $(byTestIdPrefix('account-row-'));
    await firstAccount.click();
    await browser.pause(500);
    const mainContent = await $(sel.mainContent);
    expect(await mainContent.isDisplayed()).toBe(true);
  });
});
