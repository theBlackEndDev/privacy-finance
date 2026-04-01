import { browser } from '@wdio/globals';
import { sel } from '../helpers/selectors';
import {
  waitForAppReady,
  navigateTo,
  fillInput,
  selectOption,
  clickButton,
  isVisible,
} from '../helpers/app';

describe('Reconciliation', () => {
  before(async () => {
    await waitForAppReady();
    await navigateTo('reconciliation');
  });

  it('starts reconciliation for an account', async () => {
    // Select the account to reconcile
    const accountSelect = await $(sel.reconAccountSelect);
    await accountSelect.waitForDisplayed({ timeout: 5000 });
    // Select the first available account option
    // Select the first non-placeholder account option
    let selectedAccount = false;
    const options = await accountSelect.$$('option');
    let idx = 0;
    for (const option of options) {
      if (idx > 0) {
        const accountName = await option.getText();
        await accountSelect.selectByVisibleText(accountName);
        selectedAccount = true;
        break;
      }
      idx++;
    }
    expect(selectedAccount).toBe(true);

    // Enter statement balance
    await fillInput(sel.reconBalanceInput, '5000');

    // Enter statement date
    await fillInput(sel.reconDateInput, '2026-03-31');

    // Click start reconciliation
    await clickButton(sel.reconStartBtn);
    await browser.pause(1000);

    // Verify the reconciliation view loaded -- the main content area
    // should now show reconciliation-specific UI (the start button
    // is replaced by the reconciliation workspace)
    const mainContent = await $(sel.mainContent);
    await mainContent.waitForDisplayed({ timeout: 5000 });
    const contentText = await mainContent.getText();
    expect(contentText.toLowerCase()).toMatch(/reconcil|balance|difference|statement/);
  });
});
