// ============================================
// SHARED TYPES - AGENT
// AI Agent data structures
// ============================================

/**
 * Agent operational status
 */
export type AgentStatus = 'running' | 'idle' | 'error' | 'offline' | 'starting';

/**
 * Agent capability type
 */
export type AgentCapability = 'vision' | 'audio' | 'text' | 'code' | 'multimodal';

/**
 * AI Agent definition
 */
export interface Agent {
    id: string;
    name: string;
    status: AgentStatus;
    model: string;
    capabilities: AgentCapability[];
    lastActive: string;
    memoryUsageMB?: number;
    gpuUsagePercent?: number;
}

/**
 * Agent message in chat timeline
 */
export interface AgentMessage {
    id: string;
    agentId: string;
    role: 'user' | 'agent' | 'system';
    content: string;
    timestamp: string;
    metadata?: Record<string, unknown>;
}

/**
 * Agent chat session
 */
export interface AgentSession {
    id: string;
    agentId: string;
    startedAt: string;
    messages: AgentMessage[];
}
