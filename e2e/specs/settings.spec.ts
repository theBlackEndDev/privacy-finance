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

    // Change currency via JS to avoid WebKit selectByVisibleText unicode issues
    const currencySelect = await $(sel.settingsCurrency);
    await currencySelect.waitForDisplayed({ timeout: 5000 });
    await browser.execute((sel: string) => {
      const select = document.querySelector(sel) as HTMLSelectElement;
      if (select) {
        select.value = '\u20AC';
        select.dispatchEvent(new Event('change', { bubbles: true }));
      }
    }, sel.settingsCurrency);
    await browser.pause(300);

    await clickButton('[data-testid="settings-save-btn"]');
    await browser.pause(500);

    // Verify saved by reopening
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
    await browser.execute((sel: string) => {
      const select = document.querySelector(sel) as HTMLSelectElement;
      if (select) {
        select.value = '$';
        select.dispatchEvent(new Event('change', { bubbles: true }));
      }
    }, sel.settingsCurrency);
    await browser.pause(300);

    await clickButton('[data-testid="settings-save-btn"]');
    await browser.pause(500);
  });
});
