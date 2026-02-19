import { test, expect } from '@playwright/test';
import { setupTauriMock } from './tauri-mock';

test.describe('App Loading', () => {
    test.beforeEach(async ({ page }) => {
        await setupTauriMock(page);
    });

    test('app renders with sidebar and main content', async ({ page }) => {
        await page.goto('/');
        // Wait for React to mount and render the sidebar
        await expect(page.getByText('Modules')).toBeVisible({ timeout: 10000 });
        await page.screenshot({ path: 'e2e/screenshots/app-initial.png', fullPage: true });
    });

    test('sidebar navigation modules are visible', async ({ page }) => {
        await page.goto('/');
        // Wait for sidebar items to render
        const sidebar = page.locator('.app-sidebar');
        await expect(sidebar).toBeVisible({ timeout: 10000 });
        // Verify at least one nav item is visible
        await expect(page.locator('.sidebar-item').first()).toBeVisible();
        await page.screenshot({ path: 'e2e/screenshots/sidebar.png', fullPage: true });
    });
});
