import { sel } from '../helpers/selectors';
import { waitForAppReady } from '../helpers/app';

describe('Onboarding', () => {
  before(async () => {
    await waitForAppReady();
  });

  // The app seeds default accounts/categories on fresh DB, so onboarding
  // wizard only shows if the user has never created an account. With seed
  // data, it goes straight to dashboard. This test verifies that flow.
  it('shows dashboard with seed data on fresh launch', async () => {
    const dashboard = await $(sel.dashboardSummary);
    await dashboard.waitForDisplayed({ timeout: 10000 });
    expect(await dashboard.isDisplayed()).toBe(true);
  });

  it('sidebar is visible with navigation items', async () => {
    const sidebar = await $(sel.sidebar);
    expect(await sidebar.isDisplayed()).toBe(true);

    const navDashboard = await $(sel.navDashboard);
    expect(await navDashboard.isDisplayed()).toBe(true);
  });
});
