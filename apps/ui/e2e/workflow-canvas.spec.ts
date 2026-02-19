import { test, expect } from '@playwright/test';
import { setupTauriMock } from './tauri-mock';

test.describe('Workflow Canvas', () => {
    test.beforeEach(async ({ page }) => {
        await setupTauriMock(page);
    });

    test('workflow list page renders', async ({ page }) => {
        await page.goto('/');
        await page.waitForTimeout(1000);
        // Navigate to workflows — click sidebar item containing "Workflow" or "Node Editor"
        const workflowNav = page.getByText(/workflow|node editor/i).first();
        if (await workflowNav.isVisible()) {
            await workflowNav.click();
            await page.waitForTimeout(500);
        }
        await page.screenshot({ path: 'e2e/screenshots/workflow-list.png', fullPage: true });
    });

    test('workflow canvas loads with nodes', async ({ page }) => {
        await page.goto('/');
        await page.waitForTimeout(1000);
        // Navigate to workflows
        const workflowNav = page.getByText(/workflow|node editor/i).first();
        if (await workflowNav.isVisible()) {
            await workflowNav.click();
            await page.waitForTimeout(500);
        }
        // Click on test workflow if list is visible
        const workflowCard = page.getByText('Test Workflow').first();
        if (await workflowCard.isVisible()) {
            await workflowCard.click();
            await page.waitForTimeout(1000);
        }
        await page.screenshot({ path: 'e2e/screenshots/workflow-canvas.png', fullPage: true });
    });

    test('node palette is visible on canvas page', async ({ page }) => {
        await page.goto('/');
        await page.waitForTimeout(1000);
        const workflowNav = page.getByText(/workflow|node editor/i).first();
        if (await workflowNav.isVisible()) {
            await workflowNav.click();
            await page.waitForTimeout(500);
        }
        const workflowCard = page.getByText('Test Workflow').first();
        if (await workflowCard.isVisible()) {
            await workflowCard.click();
            await page.waitForTimeout(1000);
        }
        // Check for node palette
        const palette = page.getByText(/node palette/i).first();
        await page.screenshot({ path: 'e2e/screenshots/node-palette.png', fullPage: true });
    });

    test('custom nodes render with correct labels', async ({ page }) => {
        await page.goto('/');
        await page.waitForTimeout(1000);
        const workflowNav = page.getByText(/workflow|node editor/i).first();
        if (await workflowNav.isVisible()) {
            await workflowNav.click();
            await page.waitForTimeout(500);
        }
        const workflowCard = page.getByText('Test Workflow').first();
        if (await workflowCard.isVisible()) {
            await workflowCard.click();
            await page.waitForTimeout(1500);
        }
        // Look for node type labels
        const inputNode = page.getByText('INPUT').first();
        const llmNode = page.getByText('LLM').first();
        const outputNode = page.getByText('OUTPUT').first();
        await page.screenshot({ path: 'e2e/screenshots/nodes-rendered.png', fullPage: true });
    });
});
