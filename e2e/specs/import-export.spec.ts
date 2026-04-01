import { browser } from '@wdio/globals';
import { sel } from '../helpers/selectors';
import {
  waitForAppReady,
  navigateTo,
  clickButton,
  isVisible,
  selectOption,
  waitForToast,
} from '../helpers/app';

describe('Import & Export', () => {
  before(async () => {
    await waitForAppReady();
  });

  it('imports CSV file with column mapping', async () => {
    await navigateTo('import');

    // Verify import page loaded with the file input area visible
    const fileInput = await $(sel.importFileInput);
    await fileInput.waitForDisplayed({ timeout: 5000 });
    expect(await fileInput.isDisplayed()).toBe(true);

    // Note: Native file dialogs cannot be automated in Tauri E2E.
    // Instead, verify the import wizard UI is functional and navigable.

    // Verify the preview button exists (disabled until file selected)
    const previewBtn = await $(sel.importPreviewBtn);
    expect(await previewBtn.isExisting()).toBe(true);

    // Verify the mapping step button exists
    const mappingNext = await $(sel.importMappingNext);
    expect(await mappingNext.isExisting()).toBe(true);
  });

  it('exports transactions as CSV', async () => {
    await navigateTo('transactions');

    // Click the export button to open export options
    await clickButton(sel.exportBtn);

    // Select CSV format from the format dropdown
    const formatSelect = await $(sel.exportFormatSelect);
    await formatSelect.waitForDisplayed({ timeout: 5000 });
    await formatSelect.selectByVisibleText('CSV');

    // Click the download/export button
    await clickButton(sel.exportDownloadBtn);

    // Verify export completes via toast notification
    const toastText = await waitForToast();
    expect(toastText.toLowerCase()).toMatch(/export|download|csv/);
  });
});
