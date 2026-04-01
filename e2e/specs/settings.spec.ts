import { sel } from '../helpers/selectors';
import {
  waitForAppReady,
  navigateTo,
  openSettings,
  closeModal,
  clickButton,
} from '../helpers/app';

describe('Settings', () => {
  before(async () => {
    await waitForAppReady();
  });

  it('changes currency symbol', async () => {
    await openSettings();

    const currencySelect = await $(sel.settingsCurrency);
    await currencySelect.waitForDisplayed({ timeout: 5000 });
    await currencySelect.selectByVisibleText('\u20AC \u2014 Euro');
    await browser.pause(300);

    // Click Save button (has data-testid)
    await clickButton('[data-testid="settings-save-btn"]');
    await browser.pause(500);

    // Verify the setting was saved by reopening settings
    await openSettings();
    const currencyAfter = await $(sel.settingsCurrency);
    const selectedValue = await currencyAfter.getValue();
    expect(selectedValue).toBe('\u20AC');
    await closeModal();
  });

  it('toggles theme to light', async () => {
    await openSettings();

    const themeContainer = await $(sel.settingsTheme);
    await themeContainer.waitForDisplayed({ timeout: 5000 });

    // Click the Light theme button via JS (find by text content)
    await browser.execute(() => {
      const buttons = document.querySelectorAll('[data-testid="settings-theme"] button');
      for (const btn of buttons) {
        if (btn.textContent?.includes('Light')) {
          (btn as HTMLElement).click();
          break;
        }
      }
    });
    await browser.pause(500);

    const htmlEl = await $('html');
    const themeAfter = await htmlEl.getAttribute('data-theme');
    expect(themeAfter).toBe('light');

    await closeModal();
  });

  it('changes currency back to USD', async () => {
    await openSettings();

    const currencySelect = await $(sel.settingsCurrency);
    await currencySelect.waitForDisplayed({ timeout: 5000 });
    await currencySelect.selectByVisibleText('$ \u2014 US Dollar');
    await browser.pause(300);

    await clickButton('[data-testid="settings-save-btn"]');
    await browser.pause(500);
  });
});
