import { sel } from './selectors';
import { unlinkSync } from 'fs';
import { join } from 'path';
import { homedir } from 'os';

function getAppDataDir(): string {
  if (process.platform === 'darwin') {
    return join(homedir(), 'Library/Application Support/com.privacyfinance.app');
  }
  return join(homedir(), '.local/share/com.privacyfinance.app');
}

function tryUnlink(path: string) {
  try { unlinkSync(path); } catch { /* ENOENT is fine */ }
}

/**
 * Remove the app database (and WAL/SHM journals) to ensure fresh state.
 */
export function resetDatabase() {
  const dir = getAppDataDir();
  tryUnlink(join(dir, 'privacyfinance.db'));
  tryUnlink(join(dir, 'privacyfinance.db-wal'));
  tryUnlink(join(dir, 'privacyfinance.db-shm'));
}

/**
 * Wait for the app to be fully loaded.
 * If the onboarding wizard appears (blank DB), complete it automatically
 * so every spec starts from a ready state with sidebar visible.
 */
export async function waitForAppReady() {
  // Wait for either sidebar or onboarding wizard to appear
  await browser.waitUntil(async () => {
    const sidebar = await $(sel.sidebar);
    const wizard = await $(sel.onboardingWizard);
    return (await sidebar.isDisplayed().catch(() => false))
        || (await wizard.isDisplayed().catch(() => false));
  }, { timeout: 30000, timeoutMsg: 'Neither sidebar nor onboarding wizard appeared' });

  // If onboarding wizard is showing, complete or skip it
  const wizard = await $(sel.onboardingWizard);
  if (await wizard.isDisplayed().catch(() => false)) {
    try {
      await completeOnboarding();
    } catch {
      const skipBtn = await $(sel.onboardingSkip);
      if (await skipBtn.isDisplayed().catch(() => false)) {
        await skipBtn.click();
        await browser.pause(1000);
      }
    }
  }

  // Dismiss any non-onboarding modal that might be blocking (e.g. RecurringDueModal)
  const wizard2 = await $(sel.onboardingWizard);
  const wizardStillUp = await wizard2.isDisplayed().catch(() => false);
  if (!wizardStillUp) {
    const modalClose = await $(sel.modalClose);
    if (await modalClose.isDisplayed().catch(() => false)) {
      await browser.execute((el: HTMLElement) => el.click(), modalClose as any);
      await browser.pause(500);
    }
  }

  // Now sidebar should be visible and clickable
  const sidebar = await $(sel.sidebar);
  await sidebar.waitForDisplayed({ timeout: 30000 });
}

/**
 * Auto-complete the onboarding wizard to get to the main app.
 */
async function completeOnboarding() {
  // Step 0: Welcome → click "Get started"
  await (await $(sel.onboardingNext)).waitForClickable({ timeout: 5000 });
  await (await $(sel.onboardingNext)).click();

  // Step 1: Create Account — wait for the name input to appear
  const nameInput = await $('[data-testid="onboarding-account-name"]');
  await nameInput.waitForDisplayed({ timeout: 5000 });
  await nameInput.setValue('Checking');

  const balanceInput = await $('[data-testid="onboarding-balance"]');
  await balanceInput.setValue('1000');

  // Re-query the submit button (it's a new element in step 1)
  await browser.pause(300);
  await (await $(sel.onboardingNext)).waitForClickable({ timeout: 5000 });
  await (await $(sel.onboardingNext)).click();

  // Step 2: Categories → wait for finish button, then click
  await browser.pause(1000);
  await (await $(sel.onboardingFinish)).waitForClickable({ timeout: 10000 });
  await (await $(sel.onboardingFinish)).click();
  await browser.pause(500);
}

/**
 * Navigate to a view via sidebar click.
 */
export async function navigateTo(view: keyof typeof viewSelectors) {
  const selector = viewSelectors[view];
  const navItem = await $(selector);
  await navItem.waitForDisplayed({ timeout: 30000 });
  // Use JS click to bypass any overlay that might block native clicks
  await browser.execute((el: HTMLElement) => el.click(), navItem as any);
  await navItem.click();
  await browser.pause(500); // Wait for view transition
}

const viewSelectors = {
  dashboard: sel.navDashboard,
  accounts: sel.navAccounts,
  transactions: sel.navTransactions,
  categories: sel.navCategories,
  import: sel.navImport,
  recurring: sel.navRecurring,
  budgets: sel.navBudgets,
  rules: sel.navRules,
  reports: sel.navReports,
  reconciliation: sel.navReconciliation,
} as const;

/**
 * Open settings modal.
 */
export async function openSettings() {
  const btn = await $(sel.navSettings);
  await btn.waitForDisplayed({ timeout: 15000 });
  await browser.execute((el: HTMLElement) => el.click(), btn as any);
  const modal = await $(sel.settingsModal);
  await modal.waitForDisplayed({ timeout: 10000 });
}

/**
 * Close any open modal.
 */
export async function closeModal() {
  const closeBtn = await $(sel.modalClose);
  if (await closeBtn.isDisplayed()) {
    await closeBtn.click();
    await browser.pause(300);
  }
}

/**
 * Confirm a destructive action in the confirm dialog.
 */
export async function confirmAction() {
  const dialog = await $(sel.confirmDialog);
  await dialog.waitForDisplayed({ timeout: 5000 });
  const yesBtn = await $(sel.confirmYes);
  await yesBtn.click();
  await browser.pause(500);
}

/**
 * Fill an input field (clear first, then type).
 */
export async function fillInput(selector: string, value: string) {
  const input = await $(selector);
  await input.waitForDisplayed({ timeout: 5000 });
  await input.clearValue();
  await input.setValue(value);
}

/**
 * Select an option from a select/dropdown by visible text.
 */
export async function selectOption(selector: string, text: string) {
  const select = await $(selector);
  await select.waitForDisplayed({ timeout: 5000 });
  await select.selectByVisibleText(text);
}

/**
 * Click a button and wait briefly.
 */
export async function clickButton(selector: string) {
  const btn = await $(selector);
  await btn.waitForDisplayed({ timeout: 10000 });
  await browser.execute((el: HTMLElement) => el.click(), btn as any);
  await browser.pause(300);
}

/**
 * Check if an element exists and is displayed.
 */
export async function isVisible(selector: string): Promise<boolean> {
  const el = await $(selector);
  return el.isDisplayed();
}

/**
 * Wait for a toast notification to appear and return its text.
 */
export async function waitForToast(): Promise<string> {
  const toast = await $(sel.toast);
  await toast.waitForDisplayed({ timeout: 5000 });
  return toast.getText();
}

/**
 * Get the count of elements matching a selector.
 */
export async function countElements(selector: string): Promise<number> {
  const elements = await $$(selector);
  return elements.length;
}
