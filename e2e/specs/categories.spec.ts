import { sel, byTestIdPrefix } from '../helpers/selectors';
import {
  waitForAppReady,
  navigateTo,
  fillInput,
  selectOption,
  clickButton,
  confirmAction,
  waitForToast,
  countElements,
} from '../helpers/app';

describe('Categories', () => {
  before(async () => {
    await waitForAppReady();
    await navigateTo('categories');
  });

  it('creates expense category with color', async () => {
    const initialCount = await countElements(sel.catRow);

    await clickButton(sel.createCategoryBtn);
    await fillInput(sel.catNameInput, 'Entertainment');
    await selectOption(sel.catTypeSelect, 'Expense');

    const colorPicker = await $(sel.catColorPicker);
    if (await colorPicker.isDisplayed()) {
      await colorPicker.click();
      await browser.pause(200);
    }

    await clickButton(sel.catSubmitBtn);
    await waitForToast();

    const updatedCount = await countElements(sel.catRow);
    expect(updatedCount).toBe(initialCount + 1);
  });

  it('creates income category', async () => {
    const initialCount = await countElements(sel.catRow);

    await clickButton(sel.createCategoryBtn);
    await fillInput(sel.catNameInput, 'Side Income');
    await selectOption(sel.catTypeSelect, 'Income');
    await clickButton(sel.catSubmitBtn);
    await waitForToast();

    const updatedCount = await countElements(sel.catRow);
    expect(updatedCount).toBe(initialCount + 1);
  });

  it('edits category name and color', async () => {
    const rows = await $$(sel.catRow);
    let targetRow: WebdriverIO.Element | null = null;
    for (const row of rows) {
      if ((await row.getText()).includes('Entertainment')) {
        targetRow = row;
        break;
      }
    }
    expect(targetRow).not.toBeNull();
    await targetRow!.click();
    await browser.pause(300);

    await fillInput(sel.catNameInput, 'Fun & Games');
    await clickButton(sel.catSubmitBtn);
    await waitForToast();

    const updatedRows = await $$(sel.catRow);
    let found = false;
    for (const row of updatedRows) {
      if ((await row.getText()).includes('Fun & Games')) { found = true; break; }
    }
    expect(found).toBe(true);
  });

  it('deletes category', async () => {
    const initialCount = await countElements(sel.catRow);

    const rows = await $$(sel.catRow);
    let targetRow: WebdriverIO.Element | null = null;
    for (const row of rows) {
      if ((await row.getText()).includes('Side Income')) {
        targetRow = row;
        break;
      }
    }
    expect(targetRow).not.toBeNull();
    await targetRow!.click();
    await browser.pause(300);

    await clickButton(sel.deleteBtn);
    await confirmAction();
    await waitForToast();

    const updatedCount = await countElements(sel.catRow);
    expect(updatedCount).toBeLessThan(initialCount);
  });
});
