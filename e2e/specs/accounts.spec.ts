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

    // Try the Add Account button first, fall back to create-account-btn
    const addBtn = await $('button*=Add Account');
    if (await addBtn.isExisting()) {
      await addBtn.click();
    } else {
      // Look for a "+" or create button
      const plusBtn = await $('button*=+');
      if (await plusBtn.isExisting()) {
        await plusBtn.click();
      }
    }
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
    // Click the E2E Test Account row to select it
    const rows = await $$(byTestIdPrefix('account-row-'));
    for (const row of rows) {
      const text = await row.getText();
      if (text.includes('E2E Test Account')) {
        await row.click();
        break;
      }
    }
    await browser.pause(500);

    // Look for an edit button/icon to open the edit form
    const editBtn = await $('button*=Edit');
    if (await editBtn.isExisting()) {
      await editBtn.click();
      await browser.pause(500);
    } else {
      // Try a pencil/edit icon button
      const editIcon = await $('[data-testid="edit-account-btn"]');
      if (await editIcon.isExisting()) {
        await editIcon.click();
        await browser.pause(500);
      }
    }

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

    // Click the renamed account row
    const rows = await $$(byTestIdPrefix('account-row-'));
    for (const row of rows) {
      const text = await row.getText();
      if (text.includes('Renamed E2E Account')) {
        await row.click();
        break;
      }
    }
    await browser.pause(500);

    // Look for edit button first if needed
    const editBtn = await $('button*=Edit');
    if (await editBtn.isExisting()) {
      await editBtn.click();
      await browser.pause(500);
    }

    await clickButton(sel.deleteBtn);
    await confirmAction();
    await browser.pause(500);

    const updatedCount = await countElements(byTestIdPrefix('account-row-'));
    expect(updatedCount).toBe(initialCount - 1);
  });

  it('views account detail by clicking row', async () => {
    const firstAccount = await $(byTestIdPrefix('account-row-'));
    await firstAccount.click();
    await browser.pause(500);
    const mainContent = await $(sel.mainContent);
    expect(await mainContent.isDisplayed()).toBe(true);
  });
});
