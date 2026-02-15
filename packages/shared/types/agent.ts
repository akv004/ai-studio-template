// ============================================
// SHARED TYPES - AGENT, SESSION, MESSAGE
// Matches Rust backend structs (camelCase serialization)
// ============================================

export type ToolsMode = 'sandboxed' | 'restricted' | 'full';

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
