import { sel, byTestIdPrefix } from '../helpers/selectors';
import {
  waitForAppReady,
  navigateTo,
  fillInput,
  clickButton,
  countElements,
} from '../helpers/app';

describe('Tags', () => {
  before(async () => {
    await waitForAppReady();

    // Ensure at least one transaction exists
    await navigateTo('transactions');
    await browser.pause(500);

    const txCount = await countElements(byTestIdPrefix('tx-row-'));
    if (txCount === 0) {
      await fillInput(sel.txPayeeInput, 'Tag Test Store');
      await fillInput(sel.txAmountInput, '25.00');
      await clickButton(sel.txSubmitBtn);
      await browser.pause(500);
    }
  });

  it('creates tag and assigns to transaction', async () => {
    await navigateTo('transactions');
    await browser.pause(500);

    // Click on the first transaction row to open its edit modal
    const firstTx = await $(sel.txRow);
    await firstTx.waitForDisplayed({ timeout: 10000 });
    await browser.execute((el: HTMLElement) => el.click(), firstTx as any);

    const editModal = await $(sel.txEditModal);
    await editModal.waitForDisplayed({ timeout: 5000 });

    // The TagPicker component has a "+ Add tag" trigger button that opens the dropdown.
    // The tag-input is only visible AFTER clicking that trigger.
    const tagPicker = await editModal.$('[data-testid="tag-picker"]');
    await tagPicker.waitForDisplayed({ timeout: 5000 });

    // Click the "+ Add tag" trigger button to open the dropdown
    const addTagTrigger = await tagPicker.$('button');
    await addTagTrigger.waitForClickable({ timeout: 5000 });
    await addTagTrigger.click();
    await browser.pause(300);

    // Now tag-input should be visible inside the dropdown
    const tagInput = await $('[data-testid="tag-input"]');
    await tagInput.waitForDisplayed({ timeout: 5000 });
    await tagInput.clearValue();
    await tagInput.setValue('vacation');
    await browser.pause(300);

    // Click the "+ Create" option to create the new tag
    const createBtn = await $('[data-testid="tag-create-btn"]');
    await createBtn.waitForDisplayed({ timeout: 5000 });
    await createBtn.click();
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
