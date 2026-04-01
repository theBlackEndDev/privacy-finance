import { sel } from '../helpers/selectors';
import { waitForAppReady, fillInput, selectOption, clickButton } from '../helpers/app';

describe('Onboarding', () => {
  before(async () => {
    await waitForAppReady();
  });

  it('shows onboarding wizard on blank database', async () => {
    const wizard = await $(sel.onboardingWizard);
    await wizard.waitForDisplayed({ timeout: 10000 });
    expect(await wizard.isDisplayed()).toBe(true);
  });

  it('completes onboarding by creating an account', async () => {
    // Step 0: Welcome — click "Get started"
    const getStartedBtn = await $(sel.onboardingNext);
    await getStartedBtn.waitForClickable({ timeout: 5000 });
    await getStartedBtn.click();
    await browser.pause(500);

    // Step 1: Create Account — fill name, type, balance
    await fillInput(sel.accountNameInput, 'Checking');
    await selectOption(sel.accountTypeSelect, 'Checking');
    await fillInput(sel.accountBalanceInput, '1000');

    // Click "Create account" (same data-testid as "Get started": onboarding-next)
    const createBtn = await $(sel.onboardingNext);
    await createBtn.waitForClickable({ timeout: 5000 });
    await createBtn.click();
    await browser.pause(500);

    // Step 2: Categories display — click "Start using PrivacyFinance"
    const finishBtn = await $(sel.onboardingFinish);
    await finishBtn.waitForClickable({ timeout: 5000 });
    await finishBtn.click();
    await browser.pause(500);
  });

  it('shows dashboard after onboarding completes', async () => {
    const dashboard = await $(sel.dashboardSummary);
    await dashboard.waitForDisplayed({ timeout: 10000 });
    expect(await dashboard.isDisplayed()).toBe(true);

    // Wizard should be gone
    const wizard = await $(sel.onboardingWizard);
    expect(await wizard.isExisting()).toBe(false);
  });
});
