import { browser } from '@wdio/globals';
import { sel, byTestIdPrefix } from '../helpers/selectors';
import {
  waitForAppReady,
  navigateTo,
  fillInput,
  clickButton,
  isVisible,
  countElements,
  waitForToast,
} from '../helpers/app';

describe('Tags', () => {
  before(async () => {
    await waitForAppReady();
  });

  it('creates tag and assigns to transaction', async () => {
    await navigateTo('transactions');

    // Click on the first transaction row to open its edit modal
    const firstTx = await $(sel.txRow);
    await firstTx.waitForClickable({ timeout: 5000 });
    await firstTx.click();

    const editModal = await $(sel.txEditModal);
    await editModal.waitForDisplayed({ timeout: 5000 });

    // Open tag picker within the edit modal
    await clickButton(sel.tagPicker);

    // Type a new tag name and create it
    await fillInput(sel.tagInput, 'vacation');
    await clickButton(sel.tagCreateBtn);
    await browser.pause(500);

    // Verify the tag badge now appears
    const tagBadge = await $(byTestIdPrefix('tag-badge-'));
    await tagBadge.waitForDisplayed({ timeout: 5000 });
    const badgeText = await tagBadge.getText();
    expect(badgeText.toLowerCase()).toContain('vacation');
  });

  it('removes tag from transaction', async () => {
    // The edit modal should still be open with the tag badge visible
    const removeBtn = await $(byTestIdPrefix('tag-remove-'));
    await removeBtn.waitForClickable({ timeout: 5000 });
    await removeBtn.click();
    await browser.pause(500);

    // Verify the tag badge is no longer visible
    const badgeCount = await countElements(byTestIdPrefix('tag-badge-'));
    expect(badgeCount).toBe(0);
  });
});
