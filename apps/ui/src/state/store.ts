import { create } from 'zustand';
import type {
    Agent, Session, Run, Message,
    Event as StudioEvent, SessionStats,
    CreateAgentRequest, UpdateAgentRequest, SendMessageResponse,
    McpServer, CreateMcpServerRequest, UpdateMcpServerRequest,
    CreateRunRequest,
    ApprovalRule, CreateApprovalRuleRequest, UpdateApprovalRuleRequest,
    Workflow, WorkflowSummary, CreateWorkflowRequest, UpdateWorkflowRequest,
    ValidationResult, WorkflowRunResult, NodeExecutionState, NodeExecutionStatus,
    BudgetStatus,
    Plugin, ScanResult,
} from '@ai-studio/shared';

// ============================================
// APP STATE STORE
// Real Tauri IPC — no more mocks
// ============================================

export type ModuleId =
    | 'agents'
    | 'sessions'
    | 'runs'
    | 'inspector'
    | 'workflows'
    | 'settings';

export interface Toast {
    id: string;
    message: string;
    type: 'success' | 'error' | 'info';
}

let toastCounter = 0;

export type { Agent, Session, Run, Message, Workflow, WorkflowSummary };
export type { StudioEvent, SessionStats };

// Lazy-load Tauri invoke to work in both desktop and browser dev
async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
    const { invoke: tauriInvoke } = await import('@tauri-apps/api/core');
    return tauriInvoke<T>(cmd, args);
}

function formatInvokeError(error: unknown): string {
    if (typeof error === 'string') return error;
    if (error instanceof Error) return error.message;
    if (error && typeof error === 'object') {
        const maybeError = error as { message?: unknown; error?: unknown; detail?: unknown };
        if (typeof maybeError.message === 'string') return maybeError.message;
        if (typeof maybeError.error === 'string') return maybeError.error;
        if (typeof maybeError.detail === 'string') return maybeError.detail;
        try {
            return JSON.stringify(error);
        } catch {
            // fall through
        }
    }
    return String(error);
}

interface AppState {
    // Navigation
    activeModule: ModuleId;
    setActiveModule: (module: ModuleId) => void;

    // Command Palette
    isCommandPaletteOpen: boolean;
    openCommandPalette: () => void;
    closeCommandPalette: () => void;
    toggleCommandPalette: () => void;

    // Agents
    agents: Agent[];
    agentsLoading: boolean;
    fetchAgents: () => Promise<void>;
    createAgent: (req: CreateAgentRequest) => Promise<Agent>;
    updateAgent: (id: string, updates: UpdateAgentRequest) => Promise<Agent>;
    deleteAgent: (id: string) => Promise<void>;

    // Sessions
    sessions: Session[];
    sessionsLoading: boolean;
    fetchSessions: () => Promise<void>;
    createSession: (agentId: string, title?: string) => Promise<Session>;
    branchSession: (sessionId: string, seq: number) => Promise<Session>;
    deleteSession: (id: string) => Promise<void>;

    // Messages (for active session)
    messages: Message[];
    messagesLoading: boolean;
    fetchMessages: (sessionId: string) => Promise<void>;
    sendMessage: (sessionId: string, content: string) => Promise<SendMessageResponse>;
    sending: boolean;

    // Events (for inspector)
    events: StudioEvent[];
    eventsLoading: boolean;
    sessionStats: SessionStats | null;
    fetchEvents: (sessionId: string) => Promise<void>;
    fetchSessionStats: (sessionId: string) => Promise<void>;
    pushEvent: (event: StudioEvent) => void;

    // Runs
    runs: Run[];
    runsLoading: boolean;
    fetchRuns: () => Promise<void>;
    createRun: (req: CreateRunRequest) => Promise<Run>;
    cancelRun: (id: string) => Promise<void>;

    // Database
    wipeDatabase: () => Promise<void>;

    // System Info (from Tauri)
    systemInfo: { platform: string; version: string } | null;
    setSystemInfo: (info: { platform: string; version: string }) => void;

    // Settings (provider keys, preferences)
    settings: Record<string, string>;
    settingsLoading: boolean;
    fetchSettings: () => Promise<void>;
    saveSetting: (key: string, value: string) => Promise<void>;

    // MCP Servers
    mcpServers: McpServer[];
    mcpServersLoading: boolean;
    fetchMcpServers: () => Promise<void>;
    addMcpServer: (req: CreateMcpServerRequest) => Promise<McpServer>;
    updateMcpServer: (id: string, req: UpdateMcpServerRequest) => Promise<void>;
    removeMcpServer: (id: string) => Promise<void>;

    // Approval Rules
    approvalRules: ApprovalRule[];
    approvalRulesLoading: boolean;
    fetchApprovalRules: () => Promise<void>;
    createApprovalRule: (req: CreateApprovalRuleRequest) => Promise<ApprovalRule>;
    updateApprovalRule: (id: string, updates: UpdateApprovalRuleRequest) => Promise<void>;
    deleteApprovalRule: (id: string) => Promise<void>;

    // Workflows (Node Editor)
    workflows: WorkflowSummary[];
    workflowsLoading: boolean;
    selectedWorkflow: Workflow | null;
    fetchWorkflows: () => Promise<void>;
    fetchWorkflow: (id: string) => Promise<Workflow>;
    createWorkflow: (req: CreateWorkflowRequest) => Promise<Workflow>;
    updateWorkflow: (id: string, updates: UpdateWorkflowRequest) => Promise<Workflow>;
    deleteWorkflow: (id: string) => Promise<void>;
    duplicateWorkflow: (id: string) => Promise<Workflow>;
    setSelectedWorkflow: (workflow: Workflow | null) => void;
    validateWorkflow: (id: string) => Promise<ValidationResult>;

    // Workflow Execution (Phase 3B)
    workflowRunning: boolean;
    workflowRunSessionId: string | null;
    workflowNodeStates: Record<string, NodeExecutionState>;
    runWorkflow: (workflowId: string, inputs: Record<string, unknown>) => Promise<WorkflowRunResult>;
    setNodeState: (nodeId: string, status: NodeExecutionStatus, extra?: Partial<NodeExecutionState>) => void;
    resetNodeStates: () => void;

    // Budget
    budgetStatus: BudgetStatus | null;
    budgetLoading: boolean;
    fetchBudgetStatus: () => Promise<void>;
    setBudget: (monthlyLimit: number | null, exhaustedBehavior: string) => Promise<void>;

    // Inspector navigation (set by Sessions "Inspect" button)
    inspectorSessionId: string | null;
    openInspector: (sessionId: string) => void;
    clearInspectorSession: () => void;

    // Plugins
    plugins: Plugin[];
    pluginsLoading: boolean;
    fetchPlugins: () => Promise<void>;
    scanPlugins: () => Promise<ScanResult>;
    enablePlugin: (id: string) => Promise<void>;
    disablePlugin: (id: string) => Promise<void>;
    removePlugin: (id: string) => Promise<void>;

    // Error tracking
    error: string | null;
    clearError: () => void;

    // Toast notifications
    toasts: Toast[];
    addToast: (message: string, type: Toast['type']) => void;
    removeToast: (id: string) => void;
}

export const useAppStore = create<AppState>((set, get) => ({
    // Navigation (clear error on page change)
    activeModule: 'agents',
    setActiveModule: (module) => set({ activeModule: module, error: null }),

    // Command Palette
    isCommandPaletteOpen: false,
    openCommandPalette: () => set({ isCommandPaletteOpen: true }),
    closeCommandPalette: () => set({ isCommandPaletteOpen: false }),
    toggleCommandPalette: () => set((s) => ({ isCommandPaletteOpen: !s.isCommandPaletteOpen })),

    // Agents
    agents: [],
    agentsLoading: false,
    fetchAgents: async () => {
        set({ agentsLoading: true, error: null });
        try {
            const agents = await invoke<Agent[]>('list_agents');
            set({ agents, agentsLoading: false });
        } catch (e) {
            set({ agentsLoading: false, error: `Failed to load agents: ${e}` });
        }
    },
    createAgent: async (req) => {
        try {
            const agent = await invoke<Agent>('create_agent', { agent: req });
            set((s) => ({ agents: [agent, ...s.agents] }));
            get().addToast('Agent created', 'success');
            return agent;
        } catch (e) {
            const msg = `Failed to create agent: ${e}`;
            set({ error: msg });
            get().addToast(msg, 'error');
            throw e;
        }
    },
    updateAgent: async (id, updates) => {
        try {
            const agent = await invoke<Agent>('update_agent', { id, updates });
            set((s) => ({ agents: s.agents.map((a) => a.id === id ? agent : a) }));
            get().addToast('Agent updated', 'success');
            return agent;
        } catch (e) {
            const msg = `Failed to update agent: ${e}`;
            set({ error: msg });
            get().addToast(msg, 'error');
            throw e;
        }
    },
    deleteAgent: async (id) => {
        try {
            await invoke<void>('delete_agent', { id });
            set((s) => ({ agents: s.agents.filter((a) => a.id !== id) }));
            get().addToast('Agent deleted', 'success');
        } catch (e) {
            const msg = `Failed to delete agent: ${e}`;
            set({ error: msg });
            get().addToast(msg, 'error');
        }
    },

    // Sessions
    sessions: [],
    sessionsLoading: false,
    fetchSessions: async () => {
        set({ sessionsLoading: true, error: null });
        try {
            const sessions = await invoke<Session[]>('list_sessions');
            set({ sessions, sessionsLoading: false });
        } catch (e) {
            set({ sessionsLoading: false, error: `Failed to load sessions: ${e}` });
        }
    },
    createSession: async (agentId, title) => {
        set({ error: null });
        try {
            const args: Record<string, unknown> = { agentId };
            if (title) args.title = title;
            const session = await invoke<Session>('create_session', args);
            set((s) => ({ sessions: [session, ...s.sessions] }));
            return session;
        } catch (e) {
            const msg = `Failed to create session: ${e}`;
            set({ error: msg });
            throw e;
        }
    },
    branchSession: async (sessionId, seq) => {
        try {
            const session = await invoke<Session>('branch_session', { sessionId, seq });
            set((s) => ({ sessions: [session, ...s.sessions] }));
            get().addToast('Branch created', 'success');
            return session;
        } catch (e) {
            const msg = `Failed to branch session: ${e}`;
            set({ error: msg });
            get().addToast(msg, 'error');
            throw e;
        }
    },
    deleteSession: async (id) => {
        try {
            await invoke<void>('delete_session', { id });
            set((s) => ({
                sessions: s.sessions.filter((sess) => sess.id !== id),
                messages: [],
            }));
            get().addToast('Session deleted', 'success');
        } catch (e) {
            const msg = `Failed to delete session: ${e}`;
            set({ error: msg });
            get().addToast(msg, 'error');
        }
    },

    // Messages
    messages: [],
    messagesLoading: false,
    sending: false,
    fetchMessages: async (sessionId) => {
        set({ messagesLoading: true });
        try {
            const messages = await invoke<Message[]>('get_session_messages', { sessionId });
            set({ messages, messagesLoading: false });
        } catch (e) {
            set({ messagesLoading: false, error: `Failed to load messages: ${e}` });
        }
    },
    sendMessage: async (sessionId, content) => {
        set({ sending: true, error: null });
        try {
            const resp = await invoke<SendMessageResponse>('send_message', {
                request: { sessionId, content },
            });
            set((s) => ({
                messages: [...s.messages, resp.userMessage, resp.assistantMessage],
                sending: false,
            }));
            // Refresh sessions to update message counts
            get().fetchSessions();
            return resp;
        } catch (e) {
            set({ sending: false, error: `Send failed: ${e}` });
            throw e;
        }
    },

    // Events (Inspector)
    events: [],
    eventsLoading: false,
    sessionStats: null,
    fetchEvents: async (sessionId) => {
        set({ eventsLoading: true });
        try {
            const events = await invoke<StudioEvent[]>('get_session_events', { sessionId });
            set({ events, eventsLoading: false });
        } catch (e) {
            set({ eventsLoading: false, error: `Failed to load events: ${e}` });
        }
    },
    fetchSessionStats: async (sessionId) => {
        try {
            const stats = await invoke<SessionStats>('get_session_stats', { sessionId });
            set({ sessionStats: stats });
        } catch (e) {
            set({ error: `Failed to load stats: ${e}` });
        }
    },
    pushEvent: (event) => {
        set((s) => {
            // Only append if event belongs to a session we're currently viewing
            // and isn't a duplicate
            if (s.events.length > 0 && s.events[0].sessionId !== event.sessionId) {
                return s;
            }
            if (s.events.some((e) => e.eventId === event.eventId)) {
                return s;
            }
            return { events: [...s.events, event] };
        });
    },

    // Runs
    runs: [],
    runsLoading: false,
    fetchRuns: async () => {
        set({ runsLoading: true, error: null });
        try {
            const runs = await invoke<Run[]>('list_runs');
            set({ runs, runsLoading: false });
        } catch (e) {
            set({ runsLoading: false, error: `Failed to load runs: ${e}` });
        }
    },

    createRun: async (req) => {
        set({ error: null });
        try {
            const run = await invoke<Run>('create_run', { request: req });
            set((s) => ({ runs: [run, ...s.runs] }));
            return run;
        } catch (e) {
            const msg = `Failed to create run: ${e}`;
            set({ error: msg });
            throw e;
        }
    },
    cancelRun: async (id) => {
        try {
            await invoke<void>('cancel_run', { id });
            set((s) => ({
                runs: s.runs.map((r) => r.id === id ? { ...r, status: 'cancelled' as const } : r),
            }));
            get().addToast('Run cancelled', 'info');
        } catch (e) {
            const msg = `Failed to cancel run: ${e}`;
            set({ error: msg });
            get().addToast(msg, 'error');
        }
    },

    // Database
    wipeDatabase: async () => {
        try {
            await invoke<void>('wipe_database');
            set({
                agents: [], sessions: [], runs: [], messages: [], events: [],
                mcpServers: [], workflows: [], settings: {}, sessionStats: null,
                selectedWorkflow: null,
                workflowRunning: false, workflowNodeStates: {}, workflowRunSessionId: null,
            });
            get().addToast('Database wiped', 'success');
        } catch (e) {
            const msg = `Failed to wipe database: ${e}`;
            set({ error: msg });
            get().addToast(msg, 'error');
            throw e;
        }
    },

    // Settings
    settings: {},
    settingsLoading: false,
    fetchSettings: async () => {
        set({ settingsLoading: true });
        try {
            const settings = await invoke<Record<string, string>>('get_all_settings');
            set({ settings, settingsLoading: false });
        } catch (e) {
            set({ settingsLoading: false, error: `Failed to load settings: ${e}` });
        }
    },
    saveSetting: async (key, value) => {
        try {
            await invoke<void>('set_setting', { key, value });
            set((s) => ({ settings: { ...s.settings, [key]: value } }));
        } catch (e) {
            const msg = `Failed to save setting: ${e}`;
            set({ error: msg });
            get().addToast(msg, 'error');
            throw e;
        }
    },

    // MCP Servers
    mcpServers: [],
    mcpServersLoading: false,
    fetchMcpServers: async () => {
        set({ mcpServersLoading: true });
        try {
            const servers = await invoke<McpServer[]>('list_mcp_servers');
            set({ mcpServers: servers, mcpServersLoading: false });
        } catch (e) {
            set({ mcpServersLoading: false, error: `Failed to load MCP servers: ${e}` });
        }
    },
    addMcpServer: async (req) => {
        try {
            const server = await invoke<McpServer>('add_mcp_server', { server: req });
            set((s) => ({ mcpServers: [...s.mcpServers, server] }));
            get().addToast('MCP server added', 'success');
            return server;
        } catch (e) {
            const msg = `Failed to add MCP server: ${e}`;
            set({ error: msg });
            get().addToast(msg, 'error');
            throw e;
        }
    },
    updateMcpServer: async (id, req) => {
        try {
            await invoke<void>('update_mcp_server', { id, update: req });
            get().fetchMcpServers();
        } catch (e) {
            const msg = `Failed to update MCP server: ${e}`;
            set({ error: msg });
            get().addToast(msg, 'error');
        }
    },
    removeMcpServer: async (id) => {
        try {
            await invoke<void>('remove_mcp_server', { id });
            set((s) => ({ mcpServers: s.mcpServers.filter((srv) => srv.id !== id) }));
            get().addToast('MCP server removed', 'success');
        } catch (e) {
            const msg = `Failed to remove MCP server: ${e}`;
            set({ error: msg });
            get().addToast(msg, 'error');
        }
    },

    // Approval Rules
    approvalRules: [],
    approvalRulesLoading: false,
    fetchApprovalRules: async () => {
        set({ approvalRulesLoading: true });
        try {
            const rules = await invoke<ApprovalRule[]>('list_approval_rules');
            set({ approvalRules: rules, approvalRulesLoading: false });
        } catch (e) {
            set({ approvalRulesLoading: false, error: `Failed to load approval rules: ${e}` });
        }
    },
    createApprovalRule: async (req) => {
        try {
            const rule = await invoke<ApprovalRule>('create_approval_rule', { rule: req });
            set((s) => ({ approvalRules: [...s.approvalRules, rule] }));
            get().addToast('Approval rule created', 'success');
            return rule;
        } catch (e) {
            const msg = `Failed to create approval rule: ${e}`;
            set({ error: msg });
            get().addToast(msg, 'error');
            throw e;
        }
    },
    updateApprovalRule: async (id, updates) => {
        try {
            await invoke<void>('update_approval_rule', { id, updates });
            get().fetchApprovalRules();
        } catch (e) {
            const msg = `Failed to update approval rule: ${e}`;
            set({ error: msg });
            get().addToast(msg, 'error');
        }
    },
    deleteApprovalRule: async (id) => {
        try {
            await invoke<void>('delete_approval_rule', { id });
            set((s) => ({ approvalRules: s.approvalRules.filter((r) => r.id !== id) }));
            get().addToast('Approval rule deleted', 'success');
        } catch (e) {
            const msg = `Failed to delete approval rule: ${e}`;
            set({ error: msg });
            get().addToast(msg, 'error');
        }
    },

    // Workflows (Node Editor)
    workflows: [],
    workflowsLoading: false,
    selectedWorkflow: null,
    fetchWorkflows: async () => {
        set({ workflowsLoading: true, error: null });
        try {
            const workflows = await invoke<WorkflowSummary[]>('list_workflows');
            set({ workflows, workflowsLoading: false });
        } catch (e) {
            set({ workflowsLoading: false, error: `Failed to load workflows: ${e}` });
        }
    },
    fetchWorkflow: async (id) => {
        try {
            const workflow = await invoke<Workflow>('get_workflow', { id });
            set({ selectedWorkflow: workflow });
            return workflow;
        } catch (e) {
            const msg = `Failed to load workflow: ${e}`;
            set({ error: msg });
            get().addToast(msg, 'error');
            throw e;
        }
    },
    createWorkflow: async (req) => {
        try {
            const workflow = await invoke<Workflow>('create_workflow', { workflow: req });
            set((s) => ({
                workflows: [{
                    id: workflow.id,
                    name: workflow.name,
                    description: workflow.description,
                    agentId: workflow.agentId,
                    nodeCount: 0,
                    isArchived: false,
                    createdAt: workflow.createdAt,
                    updatedAt: workflow.updatedAt,
                }, ...s.workflows],
            }));
            get().addToast('Workflow created', 'success');
            return workflow;
        } catch (e) {
            const msg = `Failed to create workflow: ${e}`;
            set({ error: msg });
            get().addToast(msg, 'error');
            throw e;
        }
    },
    updateWorkflow: async (id, updates) => {
        try {
            const workflow = await invoke<Workflow>('update_workflow', { id, updates });
            set((s) => ({
                selectedWorkflow: workflow,
                workflows: s.workflows.map((w) => w.id === id ? {
                    ...w,
                    name: workflow.name,
                    description: workflow.description,
                    agentId: workflow.agentId,
                    updatedAt: workflow.updatedAt,
                } : w),
            }));
            return workflow;
        } catch (e) {
            const msg = `Failed to update workflow: ${e}`;
            set({ error: msg });
            get().addToast(msg, 'error');
            throw e;
        }
    },
    deleteWorkflow: async (id) => {
        try {
            await invoke<void>('delete_workflow', { id });
            set((s) => ({
                workflows: s.workflows.filter((w) => w.id !== id),
                selectedWorkflow: s.selectedWorkflow?.id === id ? null : s.selectedWorkflow,
            }));
            get().addToast('Workflow deleted', 'success');
        } catch (e) {
            const msg = `Failed to delete workflow: ${e}`;
            set({ error: msg });
            get().addToast(msg, 'error');
        }
    },
    duplicateWorkflow: async (id) => {
        try {
            const workflow = await invoke<Workflow>('duplicate_workflow', { id });
            get().fetchWorkflows();
            get().addToast('Workflow duplicated', 'success');
            return workflow;
        } catch (e) {
            const msg = `Failed to duplicate workflow: ${e}`;
            set({ error: msg });
            get().addToast(msg, 'error');
            throw e;
        }
    },
    setSelectedWorkflow: (workflow) => set({ selectedWorkflow: workflow }),
    validateWorkflow: async (id) => {
        try {
            return await invoke<ValidationResult>('validate_workflow', { id });
        } catch (e) {
            const msg = `Validation failed: ${e}`;
            set({ error: msg });
            get().addToast(msg, 'error');
            throw e;
        }
    },

    // Workflow Execution (Phase 3B)
    workflowRunning: false,
    workflowRunSessionId: null,
    workflowNodeStates: {},
    runWorkflow: async (workflowId, inputs) => {
        set({ workflowRunning: true, workflowNodeStates: {}, workflowRunSessionId: null, error: null });
        try {
            const result = await invoke<WorkflowRunResult>('run_workflow', {
                request: { workflowId, inputs },
            });
            set({ workflowRunning: false, workflowRunSessionId: result.sessionId });
            if (result.status === 'completed') {
                get().addToast(`Workflow completed in ${(result.durationMs / 1000).toFixed(1)}s`, 'success');
            } else {
                get().addToast(
                    `Workflow failed (${result.sessionId}): ${result.error || 'unknown error'}`,
                    'error',
                );
                console.error('[workflow.run.failed]', {
                    workflowId,
                    sessionId: result.sessionId,
                    error: result.error || 'unknown error',
                });
            }
            return result;
        } catch (e) {
            set({ workflowRunning: false });
            const msg = `Workflow execution failed: ${formatInvokeError(e)}`;
            set({ error: msg });
            get().addToast(msg, 'error');
            console.error('[workflow.run.error]', { workflowId, inputs, error: formatInvokeError(e), raw: e });
            throw e;
        }
    },
    setNodeState: (nodeId, status, extra) => {
        set((s) => ({
            workflowNodeStates: {
                ...s.workflowNodeStates,
                [nodeId]: { nodeId, status, ...extra },
            },
        }));
    },
    resetNodeStates: () => set({ workflowNodeStates: {}, workflowRunSessionId: null }),

    // System Info
    systemInfo: null,
    setSystemInfo: (info) => set({ systemInfo: info }),

    // Budget
    budgetStatus: null,
    budgetLoading: false,
    fetchBudgetStatus: async () => {
        set({ budgetLoading: true });
        try {
            const status = await invoke<BudgetStatus>('get_budget_status');
            set({ budgetStatus: status, budgetLoading: false });
        } catch (e) {
            set({ budgetLoading: false, error: `Failed to load budget: ${e}` });
        }
    },
    setBudget: async (monthlyLimit, exhaustedBehavior) => {
        try {
            await invoke<void>('set_budget', {
                request: { monthlyLimit, exhaustedBehavior },
            });
            get().addToast('Budget updated', 'success');
            get().fetchBudgetStatus();
        } catch (e) {
            const msg = `Failed to save budget: ${e}`;
            set({ error: msg });
            get().addToast(msg, 'error');
        }
    },

    // Plugins
    plugins: [],
    pluginsLoading: false,
    fetchPlugins: async () => {
        set({ pluginsLoading: true });
        try {
            const plugins = await invoke<Plugin[]>('list_plugins');
            set({ plugins, pluginsLoading: false });
        } catch (e) {
            set({ pluginsLoading: false, error: formatInvokeError(e) });
        }
    },
    scanPlugins: async () => {
        try {
            const result = await invoke<ScanResult>('scan_plugins');
            const msg = `Scan complete: ${result.installed} installed, ${result.updated} updated`;
            get().addToast(msg, 'success');
            get().fetchPlugins();
            return result;
        } catch (e) {
            const msg = `Plugin scan failed: ${formatInvokeError(e)}`;
            set({ error: msg });
            get().addToast(msg, 'error');
            return { installed: 0, updated: 0, errors: [msg] };
        }
    },
    enablePlugin: async (id) => {
        try {
            await invoke<void>('enable_plugin', { id });
            get().addToast('Plugin enabled', 'success');
            get().fetchPlugins();
        } catch (e) {
            const msg = `Failed to enable plugin: ${formatInvokeError(e)}`;
            set({ error: msg });
            get().addToast(msg, 'error');
        }
    },
    disablePlugin: async (id) => {
        try {
            await invoke<void>('disable_plugin', { id });
            get().addToast('Plugin disabled', 'success');
            get().fetchPlugins();
        } catch (e) {
            const msg = `Failed to disable plugin: ${formatInvokeError(e)}`;
            set({ error: msg });
            get().addToast(msg, 'error');
        }
    },
    removePlugin: async (id) => {
        try {
            await invoke<void>('remove_plugin', { id });
            get().addToast('Plugin removed', 'success');
            get().fetchPlugins();
        } catch (e) {
            const msg = `Failed to remove plugin: ${formatInvokeError(e)}`;
            set({ error: msg });
            get().addToast(msg, 'error');
        }
    },

    // Inspector navigation
    inspectorSessionId: null,
    openInspector: (sessionId) => set({ inspectorSessionId: sessionId, activeModule: 'inspector' }),
    clearInspectorSession: () => set({ inspectorSessionId: null }),

    // Error
    error: null,
    clearError: () => set({ error: null }),

    // Toast notifications
    toasts: [],
    addToast: (message, type) => {
        const id = `toast-${++toastCounter}`;
        set((s) => ({ toasts: [...s.toasts, { id, message, type }] }));
        // Auto-dismiss after 5s
        setTimeout(() => get().removeToast(id), 5000);
    },
    removeToast: (id) => {
        set((s) => ({ toasts: s.toasts.filter((t) => t.id !== id) }));
    },
}));
