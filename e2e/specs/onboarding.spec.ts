import { sel } from '../helpers/selectors';

describe('Onboarding', () => {
  it('shows onboarding wizard on blank database', async () => {
    await browser.waitUntil(async () => {
      const wizard = await $(sel.onboardingWizard);
      return wizard.isDisplayed().catch(() => false);
    }, { timeout: 15000 });

    const wizard = await $(sel.onboardingWizard);
    expect(await wizard.isDisplayed()).toBe(true);
  });

  it('advances through welcome to create account step', async () => {
    // Click "Get started"
    await (await $(sel.onboardingNext)).waitForClickable({ timeout: 5000 });
    await (await $(sel.onboardingNext)).click();

    // Verify account name input appears
    const nameInput = await $('[data-testid="onboarding-account-name"]');
    await nameInput.waitForDisplayed({ timeout: 5000 });
    expect(await nameInput.isDisplayed()).toBe(true);
  });

  it('creates account and finishes onboarding', async () => {
    // Fill account details
    const nameInput = await $('[data-testid="onboarding-account-name"]');
    await nameInput.setValue('Checking');

    const balanceInput = await $('[data-testid="onboarding-balance"]');
    await balanceInput.setValue('1000');

    // Click "Create account"
    await browser.pause(300);
    await (await $(sel.onboardingNext)).waitForClickable({ timeout: 5000 });
    await (await $(sel.onboardingNext)).click();
    await browser.pause(1000);

    // Step 2: Click finish
    await (await $(sel.onboardingFinish)).waitForClickable({ timeout: 10000 });
    await (await $(sel.onboardingFinish)).click();
    await browser.pause(500);
  });

  it('shows sidebar after onboarding completes', async () => {
    const sidebar = await $(sel.sidebar);
    await sidebar.waitForDisplayed({ timeout: 10000 });
    expect(await sidebar.isDisplayed()).toBe(true);

    const wizard = await $(sel.onboardingWizard);
    expect(await wizard.isExisting()).toBe(false);
  });
});
