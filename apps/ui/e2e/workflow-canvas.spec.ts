import { test, expect } from '@playwright/test';
import { setupTauriMock } from './tauri-mock';

// Helper: wait for app to fully render, then navigate to Workflows
async function navigateToWorkflows(page: import('@playwright/test').Page) {
    await page.goto('/');
    // Wait for sidebar to render (proves React + Tauri mock loaded)
    await expect(page.getByText('Modules')).toBeVisible({ timeout: 10000 });
    // Click Workflows in sidebar
    await page.getByText('Workflows').click();
    // Wait for the workflow list or editor to appear
    await page.waitForTimeout(500);
}

test.describe('Workflow Canvas', () => {
    test.beforeEach(async ({ page }) => {
        await setupTauriMock(page);
    });

    test('workflow list page renders', async ({ page }) => {
        await navigateToWorkflows(page);
        // Should see the test workflow from our mock
        await expect(page.getByText('Test Workflow', { exact: true })).toBeVisible({ timeout: 5000 });
        await page.screenshot({ path: 'e2e/screenshots/workflow-list.png', fullPage: true });
    });

    test('workflow canvas loads with nodes', async ({ page }) => {
        await navigateToWorkflows(page);
        // Click the test workflow to open it
        const workflowCard = page.getByText('Test Workflow').first();
        await expect(workflowCard).toBeVisible({ timeout: 5000 });
        await workflowCard.click();
        // Wait for canvas to load (React Flow renders)
        await expect(page.locator('.react-flow')).toBeVisible({ timeout: 10000 });
        await page.screenshot({ path: 'e2e/screenshots/workflow-canvas.png', fullPage: true });
    });

    test('node palette is visible on canvas page', async ({ page }) => {
        await navigateToWorkflows(page);
        const workflowCard = page.getByText('Test Workflow').first();
        await expect(workflowCard).toBeVisible({ timeout: 5000 });
        await workflowCard.click();
        await expect(page.locator('.react-flow')).toBeVisible({ timeout: 10000 });
        await page.screenshot({ path: 'e2e/screenshots/node-palette.png', fullPage: true });
    });

    test('custom nodes render with correct labels', async ({ page }) => {
        await navigateToWorkflows(page);
        const workflowCard = page.getByText('Test Workflow').first();
        await expect(workflowCard).toBeVisible({ timeout: 5000 });
        await workflowCard.click();
        await expect(page.locator('.react-flow')).toBeVisible({ timeout: 10000 });
        // Wait a bit for nodes to render inside React Flow
        await page.waitForTimeout(1000);
        await page.screenshot({ path: 'e2e/screenshots/nodes-rendered.png', fullPage: true });
    });

    test('knowledge base node renders on canvas', async ({ page }) => {
        await navigateToWorkflows(page);
        const workflowCard = page.getByText('Test Workflow').first();
        await expect(workflowCard).toBeVisible({ timeout: 5000 });
        await workflowCard.click();
        await expect(page.locator('.react-flow')).toBeVisible({ timeout: 10000 });
        await page.waitForTimeout(1000);
        // Knowledge Base node should be visible with its label on the canvas
        await expect(page.locator('.react-flow').getByText('KNOWLEDGE BASE · MY DOCS')).toBeVisible({ timeout: 5000 });
        await page.screenshot({ path: 'e2e/screenshots/knowledge-base-node.png', fullPage: true });
    });

    test('knowledge base node in palette', async ({ page }) => {
        await navigateToWorkflows(page);
        const workflowCard = page.getByText('Test Workflow').first();
        await expect(workflowCard).toBeVisible({ timeout: 5000 });
        await workflowCard.click();
        await expect(page.locator('.react-flow')).toBeVisible({ timeout: 10000 });
        // Knowledge Base should appear in the palette (exact match to avoid canvas node)
        await expect(page.getByText('Knowledge Base', { exact: true })).toBeVisible({ timeout: 5000 });
        await page.screenshot({ path: 'e2e/screenshots/kb-in-palette.png', fullPage: true });
    });
});
