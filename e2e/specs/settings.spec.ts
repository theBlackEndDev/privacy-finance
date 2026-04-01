import { browser } from '@wdio/globals';
import { sel } from '../helpers/selectors';
import {
  waitForAppReady,
  navigateTo,
  openSettings,
  closeModal,
  clickButton,
  selectOption,
} from '../helpers/app';

describe('Settings', () => {
  before(async () => {
    await waitForAppReady();
  });

  it('changes currency symbol', async () => {
    await openSettings();

    // Change currency to Euro
    const currencySelect = await $(sel.settingsCurrency);
    await currencySelect.waitForDisplayed({ timeout: 5000 });
    await currencySelect.selectByVisibleText('EUR (€)');
    await browser.pause(500);

    await closeModal();

    // Navigate to dashboard and verify the Euro symbol appears
    await navigateTo('dashboard');
    await browser.pause(1000);

    const summaryCard = await $(sel.dashboardSummary);
    await summaryCard.waitForDisplayed({ timeout: 5000 });
    const summaryText = await summaryCard.getText();
    expect(summaryText).toContain('€');
  });

  it('toggles theme', async () => {
    await openSettings();

    const themeToggle = await $(sel.settingsTheme);
    await themeToggle.waitForDisplayed({ timeout: 5000 });

    // Read current theme from body class
    const bodyBefore = await $('body');
    const classBefore = await bodyBefore.getAttribute('class');

    await themeToggle.click();
    await browser.pause(500);

    const classAfter = await bodyBefore.getAttribute('class');

    // Theme class should have changed (dark <-> light)
    expect(classAfter).not.toBe(classBefore);

    await closeModal();
  });
});
