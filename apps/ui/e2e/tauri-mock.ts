/**
 * Tauri IPC Mock for Playwright E2E tests.
 * Injects window.__TAURI_INTERNALS__ with mock invoke() that returns test data.
 * This allows the React app to run in plain browser as if Tauri is present.
 */

import type { Page } from '@playwright/test';

// Test data fixtures — must match camelCase Agent interface from @ai-studio/shared
const TEST_AGENT = {
    id: 'agent-test-1',
    name: 'Test Agent',
    description: 'E2E test agent',
    systemPrompt: 'You are a test assistant.',
    provider: 'local',
    model: 'qwen3-vl',
    temperature: 0.7,
    maxTokens: 4096,
    tools: [],
    toolsMode: 'sandboxed',
    mcpServers: [],
    approvalRules: [],
    routingMode: 'single',
    routingRules: [],
    isArchived: false,
    createdAt: '2026-02-19T00:00:00Z',
    updatedAt: '2026-02-19T00:00:00Z',
};

const TEST_WORKFLOW = {
    id: 'wf-test-1',
    name: 'Test Workflow',
    description: 'E2E test workflow',
    agentId: 'agent-test-1',
    isArchived: false,
    variablesJson: '{}',
    graphJson: JSON.stringify({
        nodes: [
            { id: 'input_1', type: 'input', position: { x: 100, y: 200 }, data: { name: 'input', defaultValue: 'hello' } },
            { id: 'llm_1', type: 'llm', position: { x: 400, y: 200 }, data: { provider: 'local', model: 'qwen3-vl', prompt: '{{input}}' } },
            { id: 'output_1', type: 'output', position: { x: 700, y: 200 }, data: { name: 'result' } },
            { id: 'kb_1', type: 'knowledge_base', position: { x: 400, y: 400 }, data: { docsFolder: '~/docs', embeddingProvider: 'azure_openai', embeddingModel: 'text-embedding-3-small', chunkStrategy: 'recursive', label: 'My Docs' } },
        ],
        edges: [
            { id: 'e1', source: 'input_1', target: 'llm_1', sourceHandle: 'value', targetHandle: 'prompt' },
            { id: 'e2', source: 'llm_1', target: 'output_1', sourceHandle: 'response', targetHandle: 'value' },
            { id: 'e3', source: 'input_1', target: 'kb_1', sourceHandle: 'value', targetHandle: 'query' },
            { id: 'e4', source: 'kb_1', target: 'llm_1', sourceHandle: 'context', targetHandle: 'context' },
        ],
        viewport: { x: 0, y: 0, zoom: 1 },
    }),
    createdAt: '2026-02-19T00:00:00Z',
    updatedAt: '2026-02-19T00:00:00Z',
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
        id: `sess-${Date.now()}`, agentId, title: 'Test Session', status: 'active',
        messageCount: 0, eventCount: 0, totalInputTokens: 0, totalOutputTokens: 0, totalCostUsd: 0,
        createdAt: new Date().toISOString(), updatedAt: new Date().toISOString(),
        endedAt: null, agentName: 'Test Agent', agentModel: 'qwen3-vl',
        parentSessionId: null, branchFromSeq: null,
    }),
    delete_session: () => undefined,
    get_session_messages: () => [],
    get_session_events: () => [],
    get_session_stats: () => ({ totalEvents: 0, totalMessages: 0, totalInputTokens: 0, totalOutputTokens: 0, totalCostUsd: 0, modelsUsed: [], totalRoutingDecisions: 0, totalEstimatedSavings: 0, modelUsage: [] }),

    // Workflows
    list_workflows: () => [{ id: TEST_WORKFLOW.id, name: TEST_WORKFLOW.name, description: TEST_WORKFLOW.description, agentId: TEST_WORKFLOW.agentId, nodeCount: 4, isArchived: false, createdAt: TEST_WORKFLOW.createdAt, updatedAt: TEST_WORKFLOW.updatedAt }],
    get_workflow: () => TEST_WORKFLOW,
    create_workflow: ({ workflow }: any) => ({
        ...TEST_WORKFLOW, ...workflow, id: `wf-${Date.now()}`,
        graphJson: workflow.graphJson || TEST_WORKFLOW.graphJson,
    }),
    update_workflow: ({ id, updates }: any) => ({ ...TEST_WORKFLOW, id, ...updates }),
    delete_workflow: () => undefined,
    duplicate_workflow: () => ({ ...TEST_WORKFLOW, id: `wf-dup-${Date.now()}`, name: 'Test Workflow (copy)' }),
    validate_workflow: () => ({ valid: true, errors: [], warnings: [] }),
    run_workflow: () => ({
        sessionId: `sess-run-${Date.now()}`,
        status: 'completed',
        outputs: { result: 'Test LLM response from workflow' },
        totalTokens: 150,
        totalCostUsd: 0.001,
        durationMs: 2500,
        nodeCount: 3,
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
    get_budget_status: () => ({ monthlyLimit: 100, used: 5.23, remaining: 94.77, percentage: 5.23, exhaustedBehavior: 'ask', breakdown: [] }),
    set_budget: () => undefined,

    // Plugins
    list_plugins: () => [],
    scan_plugins: () => ({ found: 0, new: 0 }),
    connect_enabled_plugins: () => ({ connected: 0, failed: 0 }),

    // Templates
    list_templates: () => [
        { id: 'knowledge-qa', name: 'Knowledge Q&A', description: 'RAG demo', nodeCount: 4, source: 'bundled' },
        { id: 'codebase-explorer', name: 'Codebase Explorer', description: 'Index source code', nodeCount: 4, source: 'bundled' },
    ],

    // Knowledge Base (RAG)
    index_folder: () => ({ fileCount: 3, chunkCount: 15, dimensions: 1536, embeddingModel: 'text-embedding-3-small', lastIndexed: new Date().toISOString(), indexSizeBytes: 12345 }),
    search_index: () => [
        { text: 'Auth uses JWT tokens with 15min expiry', score: 0.92, source: 'auth-service.md', lineStart: 23, lineEnd: 45, chunkId: 0 },
        { text: 'Refresh tokens stored in Redis', score: 0.87, source: 'auth-service.md', lineStart: 46, lineEnd: 62, chunkId: 1 },
    ],
    get_index_stats: () => ({ fileCount: 3, chunkCount: 15, dimensions: 1536, embeddingModel: 'text-embedding-3-small', lastIndexed: '2026-02-22T12:00:00Z', indexSizeBytes: 12345 }),
    delete_index: () => undefined,

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
