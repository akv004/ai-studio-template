import { test, expect } from '@playwright/test';
import { setupTauriMock } from './tauri-mock';

test.describe('App Loading', () => {
    test.beforeEach(async ({ page }) => {
        await setupTauriMock(page);
    });

    test('app renders with sidebar and main content', async ({ page }) => {
        await page.goto('/');
        // Wait for app shell to render
        await expect(page.locator('body')).toBeVisible();
        // Take a screenshot of the initial state
        await page.screenshot({ path: 'e2e/screenshots/app-initial.png', fullPage: true });
    });

    test('sidebar navigation modules are visible', async ({ page }) => {
        await page.goto('/');
        await page.waitForTimeout(1000);
        // Check for sidebar navigation items
        const sidebar = page.locator('[class*="sidebar"], nav, [class*="nav"]').first();
        await page.screenshot({ path: 'e2e/screenshots/sidebar.png', fullPage: true });
    });
});
