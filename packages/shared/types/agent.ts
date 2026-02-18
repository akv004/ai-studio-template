// ============================================
// SHARED TYPES - AGENT, SESSION, MESSAGE
// Matches Rust backend structs (camelCase serialization)
// ============================================

export type ToolsMode = 'sandboxed' | 'restricted' | 'full';

export type RoutingMode = 'single' | 'hybrid_auto' | 'hybrid_manual';

export type RoutingCondition =
    | 'vision_required'
    | 'simple_query'
    | 'code_task'
    | 'large_context'
    | 'budget_low'
    | 'always';

export interface RoutingRule {
    condition: RoutingCondition;
    provider: string;
    model: string;
    priority: number;
}

export type ModelCapability = 'reasoning' | 'code' | 'vision' | 'speed' | 'large_context' | 'tool_use' | 'balanced' | 'multilingual' | 'free' | 'cheap';

export type CostTier = 'free' | 'cheap' | 'moderate' | 'expensive';

export interface RoutingDecision {
    provider: string;
    model: string;
    reason: string;
    estimatedSavings: number;
    alternativesConsidered: { model: string; estimatedCost: number }[];
}

export type BudgetExhaustedBehavior = 'local_only' | 'cheapest_cloud' | 'ask' | 'none';

export interface BudgetStatus {
    monthlyLimit: number | null;
    used: number;
    remaining: number;
    percentage: number;
    exhaustedBehavior: BudgetExhaustedBehavior;
    breakdown: { provider: string; cost: number }[];
}

/**
 * AI Agent definition
 */
export interface Agent {
    id: string;
    name: string;
    description: string;
    provider: string;
    model: string;
    systemPrompt: string;
    temperature: number;
    maxTokens: number;
    tools: string[];
    toolsMode: ToolsMode;
    mcpServers: string[];
    approvalRules: Record<string, unknown>[];
    routingMode: RoutingMode;
    routingRules: RoutingRule[];
    createdAt: string;
    updatedAt: string;
    isArchived: boolean;
}

export interface CreateAgentRequest {
    name: string;
    provider: string;
    model: string;
    description?: string;
    systemPrompt?: string;
    temperature?: number;
    maxTokens?: number;
    tools?: string[];
    toolsMode?: ToolsMode;
    mcpServers?: string[];
    routingMode?: RoutingMode;
    routingRules?: RoutingRule[];
}

export interface UpdateAgentRequest {
    name?: string;
    description?: string;
    provider?: string;
    model?: string;
    systemPrompt?: string;
    temperature?: number;
    maxTokens?: number;
    tools?: string[];
    toolsMode?: ToolsMode;
    mcpServers?: string[];
    approvalRules?: Record<string, unknown>[];
    routingMode?: RoutingMode;
    routingRules?: RoutingRule[];
}

/**
 * Session — an interactive conversation with an agent
 */
export interface Session {
    id: string;
    agentId: string;
    title: string;
    status: string;
    messageCount: number;
    eventCount: number;
    totalInputTokens: number;
    totalOutputTokens: number;
    totalCostUsd: number;
    createdAt: string;
    updatedAt: string;
    endedAt: string | null;
    agentName: string | null;
    agentModel: string | null;
    parentSessionId: string | null;
    branchFromSeq: number | null;
}

/**
 * Message in a session
 */
export interface Message {
    id: string;
    sessionId: string;
    seq: number;
    role: 'user' | 'assistant' | 'system';
    content: string;
    model: string | null;
    provider: string | null;
    inputTokens: number | null;
    outputTokens: number | null;
    costUsd: number | null;
    durationMs: number | null;
    createdAt: string;
}

/**
 * Event recorded by the inspector
 */
export interface Event {
    eventId: string;
    type: string;
    ts: string;
    sessionId: string;
    source: string;
    seq: number;
    payload: Record<string, unknown>;
    costUsd: number | null;
}

/**
 * Aggregated stats for a session
 */
export interface SessionStats {
    totalEvents: number;
    totalMessages: number;
    totalInputTokens: number;
    totalOutputTokens: number;
    totalCostUsd: number;
    modelsUsed: string[];
    totalRoutingDecisions: number;
    totalEstimatedSavings: number;
    modelUsage: { model: string; calls: number; cost: number }[];
}

/**
 * Send message request/response
 */
export interface SendMessageRequest {
    sessionId: string;
    content: string;
}

export interface SendMessageResponse {
    userMessage: Message;
    assistantMessage: Message;
}

// ============================================
// MCP SERVER TYPES
// ============================================

export type McpTransport = 'stdio' | 'sse' | 'streamable-http';

/**
 * MCP Server configuration — stored in SQLite, managed via Settings
 */
export interface McpServer {
    id: string;
    name: string;
    transport: McpTransport;
    command: string | null;
    args: string[];
    url: string | null;
    env: Record<string, string>;
    enabled: boolean;
    createdAt: string;
    updatedAt: string;
}

export interface CreateMcpServerRequest {
    name: string;
    transport?: McpTransport;
    command?: string;
    args?: string[];
    url?: string;
    env?: Record<string, string>;
}

export interface UpdateMcpServerRequest {
    name?: string;
    transport?: McpTransport;
    command?: string;
    args?: string[];
    url?: string;
    env?: Record<string, string>;
    enabled?: boolean;
}

// ============================================
// APPROVAL RULE TYPES
// ============================================

export type ApprovalAction = 'allow' | 'deny' | 'ask';

export interface ApprovalRule {
    id: string;
    name: string;
    toolPattern: string;
    action: ApprovalAction;
    priority: number;
    enabled: boolean;
    createdAt: string;
}

export interface CreateApprovalRuleRequest {
    name: string;
    toolPattern: string;
    action: ApprovalAction;
    priority?: number;
}

export interface UpdateApprovalRuleRequest {
    name?: string;
    toolPattern?: string;
    action?: ApprovalAction;
    priority?: number;
    enabled?: boolean;
}

// ============================================
// WORKFLOW TYPES (Node Editor)
// ============================================

export interface Workflow {
    id: string;
    name: string;
    description: string;
    graphJson: string;
    variablesJson: string;
    agentId: string | null;
    isArchived: boolean;
    createdAt: string;
    updatedAt: string;
}

export interface WorkflowSummary {
    id: string;
    name: string;
    description: string;
    agentId: string | null;
    nodeCount: number;
    isArchived: boolean;
    createdAt: string;
    updatedAt: string;
}

export interface CreateWorkflowRequest {
    name: string;
    description?: string;
    graphJson?: string;
    variablesJson?: string;
    agentId?: string;
}

export interface UpdateWorkflowRequest {
    name?: string;
    description?: string;
    graphJson?: string;
    variablesJson?: string;
    agentId?: string | null;
}

// ============================================
// WORKFLOW EXECUTION TYPES (Phase 3B)
// ============================================

export type NodeExecutionStatus = 'idle' | 'running' | 'completed' | 'error' | 'waiting' | 'skipped';

export interface NodeExecutionState {
    nodeId: string;
    status: NodeExecutionStatus;
    output?: string;
    error?: string;
    tokens?: number;
    costUsd?: number;
    durationMs?: number;
}

export interface ValidationResult {
    valid: boolean;
    errors: string[];
    warnings: string[];
}

export interface WorkflowRunResult {
    sessionId: string;
    status: string;
    outputs: Record<string, unknown>;
    totalTokens: number;
    totalCostUsd: number;
    durationMs: number;
    nodeCount: number;
    error?: string;
}

// ============================================
// PLUGIN TYPES
// ============================================

export type PluginPermission = 'network' | 'filesystem' | 'shell' | 'env';
export type PluginRuntime = 'python' | 'node' | 'binary';

export interface Plugin {
    id: string;
    name: string;
    version: string;
    description: string;
    author: string;
    homepage: string;
    license: string;
    runtime: PluginRuntime;
    entryPoint: string;
    transport: 'stdio';
    permissions: PluginPermission[];
    providesTools: boolean;
    providesNodeTypes: string[];
    directory: string;
    enabled: boolean;
    installedAt: string;
    updatedAt: string;
}

export interface ScanResult {
    installed: number;
    updated: number;
    errors: string[];
}

export interface PluginConnectResult {
    tools: string[];
}

export interface PluginStartupResult {
    connected: number;
    failed: number;
    errors: string[];
}
