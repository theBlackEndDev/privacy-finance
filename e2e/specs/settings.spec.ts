import { sel } from '../helpers/selectors';
import {
  waitForAppReady,
  navigateTo,
  openSettings,
  closeModal,
} from '../helpers/app';

describe('Settings', () => {
  before(async () => {
    await waitForAppReady();
  });

  it('changes currency symbol', async () => {
    await openSettings();

    // Change currency to Euro using exact option text
    const currencySelect = await $(sel.settingsCurrency);
    await currencySelect.waitForDisplayed({ timeout: 5000 });
    await currencySelect.selectByVisibleText('\u20AC \u2014 Euro');
    await browser.pause(500);

    await closeModal();

    // Navigate to dashboard and verify the Euro symbol appears
    await navigateTo('dashboard');
    await browser.pause(1000);

    const summaryCard = await $(sel.dashboardSummary);
    await summaryCard.waitForDisplayed({ timeout: 5000 });
    const summaryText = await summaryCard.getText();
    expect(summaryText).toContain('\u20AC');
  });

  it('toggles theme to light', async () => {
    await openSettings();

    const themeContainer = await $(sel.settingsTheme);
    await themeContainer.waitForDisplayed({ timeout: 5000 });

    // Read current body class
    const bodyBefore = await $('body');
    const classBefore = await bodyBefore.getAttribute('class');

    // Click the Light theme button
    const lightBtn = await themeContainer.$('button*=Light');
    if (await lightBtn.isExisting()) {
      await lightBtn.click();
      await browser.pause(500);
    }

    const classAfter = await bodyBefore.getAttribute('class');
    // Theme class should have changed
    expect(classAfter).not.toBe(classBefore);

    await closeModal();
  });

  it('changes currency back to USD', async () => {
    await openSettings();

    const currencySelect = await $(sel.settingsCurrency);
    await currencySelect.waitForDisplayed({ timeout: 5000 });
    await currencySelect.selectByVisibleText('$ \u2014 US Dollar');
    await browser.pause(500);

    await closeModal();
  });
});
