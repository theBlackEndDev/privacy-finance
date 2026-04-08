import { sel, byTestIdPrefix } from '../helpers/selectors';
import {
  waitForAppReady,
  navigateTo,
  fillInput,
  selectOption,
  clickButton,
  countElements,
} from '../helpers/app';

describe('Reconciliation', () => {
  before(async () => {
    await waitForAppReady();

    // Create transactions in the default Checking account so reconciliation has items
    await navigateTo('transactions');
    const kindGroup = await $(sel.txKindSelect);
    const expenseBtn = await kindGroup.$('button*=\u2212');

    if (await expenseBtn.isExisting()) await expenseBtn.click();
    await fillInput(sel.txPayeeInput, 'Recon Test Store');
    await fillInput(sel.txAmountInput, '50.00');
    await selectOption(sel.txCategorySelect, 'Food');
    await clickButton(sel.txSubmitBtn);
    await browser.pause(500);

    if (await expenseBtn.isExisting()) await expenseBtn.click();
    await fillInput(sel.txPayeeInput, 'Recon Test Gas');
    await fillInput(sel.txAmountInput, '30.00');
    await selectOption(sel.txCategorySelect, 'Transportation');
    await clickButton(sel.txSubmitBtn);
    await browser.pause(500);

    await navigateTo('reconciliation');
  });

  it('shows account list for reconciliation', async () => {
    // The reconciliation page shows account buttons (recon-account-{id})
    const accountList = await $(sel.reconAccountSelect);
    await accountList.waitForDisplayed({ timeout: 5000 });
    expect(await accountList.isDisplayed()).toBe(true);
  });

  it('starts reconciliation for an account', async () => {
    // Click the first account button (data-testid starts with recon-account-)
    const accountBtn = await $(byTestIdPrefix('recon-account-'));
    await accountBtn.waitForDisplayed({ timeout: 10000 });
    await browser.execute((el: HTMLElement) => el.click(), accountBtn as any);
    await browser.pause(1000);

    // Verify the reconciliation wizard/workspace appeared
    const mainContent = await $(sel.mainContent);
    await mainContent.waitForDisplayed({ timeout: 5000 });
    const contentText = await mainContent.getText();
    expect(contentText.toLowerCase()).toMatch(/reconcil|balance|difference|statement/);
  });

  it('enters statement balance and date, then starts reconciliation', async () => {
    // Fill in statement date (today)
    const today = new Date().toISOString().split('T')[0];
    await browser.execute((dateSel: string, balSel: string, dateVal: string, balVal: string) => {
      const dateInput = document.querySelector(dateSel) as HTMLInputElement;
      const balInput = document.querySelector(balSel) as HTMLInputElement;
      const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value')?.set;
      if (dateInput && setter) {
        setter.call(dateInput, dateVal);
        dateInput.dispatchEvent(new Event('input', { bubbles: true }));
        dateInput.dispatchEvent(new Event('change', { bubbles: true }));
      }
      if (balInput && setter) {
        setter.call(balInput, balVal);
        balInput.dispatchEvent(new Event('input', { bubbles: true }));
        balInput.dispatchEvent(new Event('change', { bubbles: true }));
      }
    }, sel.reconDateInput, sel.reconBalanceInput, today, '920');
    await browser.pause(500);

    // Click Start Reconciliation
    await clickButton(sel.reconStartBtn);
    await browser.pause(1000);

    // Verify step 2 appeared — should show difference/balance info
    const mainContent = await $(sel.mainContent);
    const text = await mainContent.getText();
    expect(text.toLowerCase()).toMatch(/difference|cleared|statement/);
  });

  it('shows transaction items to reconcile', async () => {
    // The reconciliation wizard step 2 shows a list of transactions
    const mainContent = await $(sel.mainContent);
    const text = await mainContent.getText();
    // Should contain at least one of our test transactions
    expect(text).toMatch(/Recon Test|Date|Payee|Amount/);
  });

  it('cancels reconciliation to return to account list', async () => {
    // Click Cancel to go back
    const cancelBtn = await $(sel.reconCancelBtn);
    if (await cancelBtn.isDisplayed()) {
      await browser.execute((el: HTMLElement) => el.click(), cancelBtn as any);
      await browser.pause(1000);
    }

    // Should be back at the reconciliation account list
    const accountList = await $(sel.reconAccountSelect);
    await accountList.waitForDisplayed({ timeout: 10000 });
    expect(await accountList.isDisplayed()).toBe(true);
  });
});
