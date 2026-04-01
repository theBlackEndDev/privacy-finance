import { browser } from '@wdio/globals';
import { sel, byTestIdPrefix } from '../helpers/selectors';
import {
  waitForAppReady,
  navigateTo,
  fillInput,
  selectOption,
  clickButton,
  waitForToast,
  countElements,
  isVisible,
} from '../helpers/app';

describe('Recurring Transactions', () => {
  before(async () => {
    await waitForAppReady();
    await navigateTo('recurring');
  });

  it('creates monthly recurring transaction', async () => {
    const initialCount = await countElements(sel.recurringRow);

    await clickButton(sel.recurringCreateBtn);

    await fillInput(sel.recurringPayeeInput, 'Netflix');
    await fillInput(sel.recurringAmountInput, '15.99');
    await selectOption(sel.recurringFreqSelect, 'Monthly');

    await clickButton(sel.recurringSubmitBtn);
    await browser.pause(1000);

    const newCount = await countElements(sel.recurringRow);
    expect(newCount).toBe(initialCount + 1);

    // Verify the newly created row contains Netflix
    let foundNetflix = false;
    const rows = await $$(sel.recurringRow);
    for (const row of rows) {
      const text = await row.getText();
      if (text.includes('Netflix')) {
        foundNetflix = true;
        break;
      }
    }
    expect(foundNetflix).toBe(true);
  });

  it('generates due recurring transactions', async () => {
    const generateBtn = await $(sel.recurringGenerateBtn);
    await generateBtn.waitForClickable({ timeout: 5000 });
    await generateBtn.click();

    // Wait for generation to complete and toast confirmation
    const toastText = await waitForToast();
    expect(toastText.toLowerCase()).toMatch(/generat|creat|transaction/);
  });

  it('toggles recurring active status', async () => {
    // Find the first recurring toggle
    const toggle = await $(sel.recurringToggle);
    await toggle.waitForDisplayed({ timeout: 5000 });

    // Read the current state via aria-checked or class
    const wasChecked = await toggle.getAttribute('aria-checked');

    await toggle.click();
    await browser.pause(500);

    const isChecked = await toggle.getAttribute('aria-checked');
    expect(isChecked).not.toBe(wasChecked);
  });
});
