import { sel } from '../helpers/selectors';
import {
  waitForAppReady,
  navigateTo,
  fillInput,
  selectOption,
  clickButton,
  confirmAction,
  countElements,
} from '../helpers/app';

describe('Categories', () => {
  before(async () => {
    await waitForAppReady();
    await navigateTo('categories');
  });

  it('shows seed categories on fresh launch', async () => {
    const count = await countElements(sel.catRow);
    expect(count).toBeGreaterThanOrEqual(15);
  });

  it('creates expense category', async () => {
    const initialCount = await countElements(sel.catRow);

    await clickButton(sel.createCategoryBtn);
    await browser.pause(300);
    await fillInput(sel.catNameInput, 'E2E Test Category');
    await selectOption(sel.catTypeSelect, 'Expense');
    await clickButton(sel.catSubmitBtn);
    await browser.pause(500);

    const updatedCount = await countElements(sel.catRow);
    expect(updatedCount).toBe(initialCount + 1);
  });

  it('creates income category', async () => {
    const initialCount = await countElements(sel.catRow);

    await clickButton(sel.createCategoryBtn);
    await browser.pause(300);
    await fillInput(sel.catNameInput, 'E2E Income');
    await selectOption(sel.catTypeSelect, 'Income');
    await clickButton(sel.catSubmitBtn);
    await browser.pause(500);

    const updatedCount = await countElements(sel.catRow);
    expect(updatedCount).toBe(initialCount + 1);
  });

  it('edits category name', async () => {
    // CategoryRow has Edit and Delete buttons directly in the row.
    // Click the Edit button within the row that contains "E2E Test Category".
    const rows = await $$(sel.catRow);
    for (const row of rows) {
      if ((await row.getText()).includes('E2E Test Category')) {
        const editBtn = await row.$('button*=Edit');
        await editBtn.waitForClickable({ timeout: 5000 });
        await editBtn.click();
        break;
      }
    }
    await browser.pause(300);

    // The CategoryForm modal should now be open
    await fillInput(sel.catNameInput, 'Renamed E2E Category');
    await clickButton(sel.catSubmitBtn);
    await browser.pause(500);

    const updatedRows = await $$(sel.catRow);
    let found = false;
    for (const row of updatedRows) {
      if ((await row.getText()).includes('Renamed E2E Category')) { found = true; break; }
    }
    expect(found).toBe(true);
  });

  it('deletes category', async () => {
    const initialCount = await countElements(sel.catRow);

    // Click the Delete button within the "E2E Income" row
    const rows = await $$(sel.catRow);
    for (const row of rows) {
      if ((await row.getText()).includes('E2E Income')) {
        const deleteBtn = await row.$('button*=Delete');
        await deleteBtn.waitForClickable({ timeout: 5000 });
        await deleteBtn.click();
        break;
      }
    }
    await browser.pause(300);

    // Confirm the deletion via ConfirmDialog
    await confirmAction();
    await browser.pause(500);

    const updatedCount = await countElements(sel.catRow);
    expect(updatedCount).toBeLessThan(initialCount);
  });
});
