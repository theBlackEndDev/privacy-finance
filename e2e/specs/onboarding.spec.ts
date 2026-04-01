import { sel } from '../helpers/selectors';
import { fillInput, clickButton } from '../helpers/app';

describe('Onboarding', () => {
  // NOTE: This spec tests the onboarding flow explicitly.
  // Other specs rely on waitForAppReady() which auto-completes onboarding.
  // This spec must NOT call waitForAppReady() — it tests the raw wizard.

  it('shows onboarding wizard on blank database', async () => {
    // Wait for either sidebar or wizard
    await browser.waitUntil(async () => {
      const wizard = await $(sel.onboardingWizard);
      return wizard.isDisplayed().catch(() => false);
    }, { timeout: 15000 });

    const wizard = await $(sel.onboardingWizard);
    expect(await wizard.isDisplayed()).toBe(true);
  });

  it('completes onboarding by creating an account', async () => {
    // Step 0: Welcome — click "Get started"
    await clickButton(sel.onboardingNext);
    await browser.pause(500);

    // Step 1: Create Account — use wizard-specific testids
    await fillInput('[data-testid="onboarding-account-name"]', 'Checking');
    await fillInput('[data-testid="onboarding-balance"]', '1000');

    // Click "Create account" (same testid as "Get started")
    await clickButton(sel.onboardingNext);
    await browser.pause(1000);

    // Step 2: Categories — click "Start using PrivacyFinance"
    await clickButton(sel.onboardingFinish);
    await browser.pause(500);
  });

  it('shows dashboard after onboarding completes', async () => {
    const sidebar = await $(sel.sidebar);
    await sidebar.waitForDisplayed({ timeout: 10000 });
    expect(await sidebar.isDisplayed()).toBe(true);

    const wizard = await $(sel.onboardingWizard);
    expect(await wizard.isExisting()).toBe(false);
  });
});
