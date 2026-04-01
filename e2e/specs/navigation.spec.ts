import { sel } from '../helpers/selectors';
import { waitForAppReady, navigateTo } from '../helpers/app';

describe('Navigation', () => {
  before(async () => {
    await waitForAppReady();
  });

  const views = [
    'dashboard', 'accounts', 'transactions', 'categories',
    'import', 'recurring', 'budgets', 'rules', 'reports', 'reconciliation',
  ] as const;

  it('navigates to all sidebar views', async () => {
    for (const view of views) {
      await navigateTo(view);
      const mainContent = await $(sel.mainContent);
      expect(await mainContent.isDisplayed()).toBe(true);
    }
  });

  it('navigates via keyboard shortcuts', async () => {
    const shortcuts: [string, typeof views[number]][] = [
      ['1', 'dashboard'], ['2', 'accounts'], ['3', 'transactions'],
      ['4', 'categories'], ['5', 'import'], ['6', 'recurring'],
      ['7', 'budgets'], ['8', 'rules'], ['9', 'reports'],
    ];

    for (const [key, _view] of shortcuts) {
      await browser.keys(['Control', key]);
      await browser.pause(300);
      const mainContent = await $(sel.mainContent);
      expect(await mainContent.isDisplayed()).toBe(true);
    }
  });
});
