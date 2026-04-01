import { sel } from '../helpers/selectors';
import { waitForAppReady, fillInput, clickButton } from '../helpers/app';

describe('Onboarding', () => {
  before(async () => {
    await waitForAppReady();
  });

  it('shows onboarding wizard on fresh app', async () => {
    const wizard = await $(sel.onboardingWizard);
    await wizard.waitForDisplayed({ timeout: 10000 });
    expect(await wizard.isDisplayed()).toBe(true);
  });

  it('completes onboarding creating first account', async () => {
    // Step 1: Welcome — click Get Started / Next
    await clickButton(sel.onboardingNext);

    // Step 2: Create account
    await fillInput(sel.accountNameInput, 'Checking');
    await fillInput(sel.accountBalanceInput, '1000');
    await clickButton(sel.onboardingNext);

    // Step 3: Categories — finish
    const finishBtn = await $(sel.onboardingFinish);
    await finishBtn.waitForDisplayed({ timeout: 5000 });
    await clickButton(sel.onboardingFinish);
  });

  it('shows dashboard after onboarding', async () => {
    const dashboard = await $(sel.dashboardSummary);
    await dashboard.waitForDisplayed({ timeout: 10000 });
    expect(await dashboard.isDisplayed()).toBe(true);

    // Wizard should be gone
    const wizard = await $(sel.onboardingWizard);
    expect(await wizard.isExisting()).toBe(false);
  });
});
