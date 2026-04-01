import { sel, byTestIdPrefix } from '../helpers/selectors';
import {
  waitForAppReady,
  navigateTo,
} from '../helpers/app';

describe('Reconciliation', () => {
  before(async () => {
    await waitForAppReady();
    await navigateTo('reconciliation');
  });

  it('shows account list for reconciliation', async () => {
    // The reconciliation page shows account buttons (recon-account-{id})
    const accountList = await $(sel.reconAccountSelect);
    await accountList.waitForDisplayed({ timeout: 5000 });
    expect(await accountList.isDisplayed()).toBe(true);
  });

  it('starts reconciliation for an account', async () => {
    // Click the first account button (data-testid starts with recon-account-)
    const accountBtn = await $(byTestIdPrefix('recon-account-'));
    await accountBtn.waitForDisplayed({ timeout: 10000 });
    await browser.execute((el: HTMLElement) => el.click(), accountBtn as any);
    await browser.pause(1000);

    // Verify the reconciliation wizard/workspace appeared
    const mainContent = await $(sel.mainContent);
    await mainContent.waitForDisplayed({ timeout: 5000 });
    const contentText = await mainContent.getText();
    expect(contentText.toLowerCase()).toMatch(/reconcil|balance|difference|statement/);
  });
});
