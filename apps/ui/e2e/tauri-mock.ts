/**
 * Tauri IPC Mock for Playwright E2E tests.
 * Injects window.__TAURI_INTERNALS__ with mock invoke() that returns test data.
 * This allows the React app to run in plain browser as if Tauri is present.
 */

import type { Page } from '@playwright/test';

// Test data fixtures
const TEST_AGENT = {
    id: 'agent-test-1',
    name: 'Test Agent',
    description: 'E2E test agent',
    system_prompt: 'You are a test assistant.',
    provider: 'local',
    model: 'qwen3-vl',
    tools: [],
    created_at: '2026-02-19T00:00:00Z',
    updated_at: '2026-02-19T00:00:00Z',
};

const TEST_WORKFLOW = {
    id: 'wf-test-1',
    name: 'Test Workflow',
    description: 'E2E test workflow',
    agent_id: 'agent-test-1',
    graph_json: JSON.stringify({
        nodes: [
            { id: 'input_1', type: 'input', position: { x: 100, y: 200 }, data: { name: 'input', defaultValue: 'hello' } },
            { id: 'llm_1', type: 'llm', position: { x: 400, y: 200 }, data: { provider: 'local', model: 'qwen3-vl', prompt: '{{input}}' } },
            { id: 'output_1', type: 'output', position: { x: 700, y: 200 }, data: { name: 'result' } },
        ],
        edges: [
            { id: 'e1', source: 'input_1', target: 'llm_1', sourceHandle: 'value', targetHandle: 'prompt' },
            { id: 'e2', source: 'llm_1', target: 'output_1', sourceHandle: 'response', targetHandle: 'value' },
        ],
        viewport: { x: 0, y: 0, zoom: 1 },
    }),
    created_at: '2026-02-19T00:00:00Z',
    updated_at: '2026-02-19T00:00:00Z',
};

const TEST_SETTINGS: Record<string, string> = {
    'provider.local.base_url': 'http://localhost:11434/v1',
    'provider.local.api_key': '',
    'provider.google.api_key': 'test-key',
    'budget.monthly_limit': '100',
};

// Mock IPC handler — maps Tauri command names to test responses
const MOCK_HANDLERS: Record<string, (args?: any) => any> = {
    // Agents
    list_agents: () => [TEST_AGENT],
    create_agent: ({ agent }: any) => ({ ...TEST_AGENT, ...agent, id: `agent-${Date.now()}` }),
    update_agent: ({ id, updates }: any) => ({ ...TEST_AGENT, id, ...updates }),
    delete_agent: () => undefined,

    // Sessions
    list_sessions: () => [],
    create_session: ({ agentId }: any) => ({
        id: `sess-${Date.now()}`, agent_id: agentId, title: 'Test Session',
        created_at: new Date().toISOString(), updated_at: new Date().toISOString(),
    }),
    delete_session: () => undefined,
    get_session_messages: () => [],
    get_session_events: () => [],
    get_session_stats: () => ({ total_events: 0, total_tokens: 0, total_cost: 0, duration_ms: 0 }),

    // Workflows
    list_workflows: () => [{ id: TEST_WORKFLOW.id, name: TEST_WORKFLOW.name, description: TEST_WORKFLOW.description, created_at: TEST_WORKFLOW.created_at, updated_at: TEST_WORKFLOW.updated_at }],
    get_workflow: () => TEST_WORKFLOW,
    create_workflow: ({ workflow }: any) => ({
        ...TEST_WORKFLOW, ...workflow, id: `wf-${Date.now()}`,
        graph_json: workflow.graph_json || TEST_WORKFLOW.graph_json,
    }),
    update_workflow: ({ id, updates }: any) => ({ ...TEST_WORKFLOW, id, ...updates }),
    delete_workflow: () => undefined,
    duplicate_workflow: () => ({ ...TEST_WORKFLOW, id: `wf-dup-${Date.now()}`, name: 'Test Workflow (copy)' }),
    validate_workflow: () => ({ valid: true, errors: [], warnings: [] }),
    run_workflow: () => ({
        session_id: `sess-run-${Date.now()}`,
        outputs: { result: 'Test LLM response from workflow' },
        total_tokens: 150,
        total_cost: 0.001,
        duration_ms: 2500,
    }),

    // Settings
    get_all_settings: () => TEST_SETTINGS,
    set_setting: () => undefined,

    // MCP
    list_mcp_servers: () => [],
    add_mcp_server: () => ({ id: `mcp-${Date.now()}`, name: 'test', transport: 'stdio' }),
    remove_mcp_server: () => undefined,

    // Approval Rules
    list_approval_rules: () => [],
    create_approval_rule: () => ({ id: `rule-${Date.now()}` }),

    // Runs
    list_runs: () => [],
    create_run: () => ({ id: `run-${Date.now()}`, status: 'completed' }),

    // Budget
    get_budget_status: () => ({ monthly_limit: 100, current_cost: 5.23, percentage: 5.23 }),
    set_budget: () => undefined,

    // Plugins
    list_plugins: () => [],
    scan_plugins: () => ({ found: 0, new: 0 }),
    connect_enabled_plugins: () => ({ connected: 0, failed: 0 }),

    // Database
    wipe_database: () => undefined,

    // Tauri plugin commands (event system, etc.)
    'plugin:event|listen': () => undefined,
    'plugin:event|unlisten': () => undefined,
};

/**
 * Inject Tauri mock into page before app loads.
 * Must be called via page.addInitScript() BEFORE navigating.
 */
export function getTauriMockScript(): string {
    return `
        window.__TAURI_INTERNALS__ = {
            invoke: async (cmd, args) => {
                const handlers = ${JSON.stringify(Object.keys(MOCK_HANDLERS))};
                console.log('[tauri-mock] invoke:', cmd, args);

                // Return mock data based on command
                const mockData = (${JSON.stringify(
                    Object.fromEntries(
                        Object.entries(MOCK_HANDLERS).map(([k, v]) => [k, '__FUNCTION__'])
                    )
                )});

                // We can't serialize functions, so we'll use a switch
                return window.__TAURI_MOCK_INVOKE__(cmd, args);
            },
            convertFileSrc: (path) => path,
        };

        // Event listeners (for workflow events, etc.)
        window.__TAURI_LISTENERS__ = {};
        window.__TAURI_INTERNALS__.transformCallback = (fn) => {
            const id = 'cb_' + Math.random().toString(36).slice(2);
            window[id] = fn;
            return id;
        };
    `;
}

/**
 * Set up Tauri mock on a Playwright page.
 * Registers mock invoke handler and injects the mock script.
 */
export async function setupTauriMock(page: Page) {
    // Expose the mock invoke function to the browser
    await page.exposeFunction('__TAURI_MOCK_INVOKE__', async (cmd: string, args: any) => {
        const handler = MOCK_HANDLERS[cmd];
        if (handler) {
            try {
                return handler(args);
            } catch (e) {
                console.error(`[tauri-mock] Error in handler for '${cmd}':`, e);
                throw e;
            }
        }
        console.warn(`[tauri-mock] No handler for command: ${cmd}`);
        return null;
    });

    // Inject the mock before page loads
    await page.addInitScript(getTauriMockScript());

    // Also mock @tauri-apps/api/event for listen()
    await page.addInitScript(`
        window.__TAURI_EVENT_LISTENERS__ = new Map();
        // The app dynamically imports @tauri-apps/api/event — we can't easily mock that.
        // But listen() calls fail gracefully in the app with try/catch, so it's OK.
    `);
}
