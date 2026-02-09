// ============================================
// SHARED TYPES - RUNS
// Matches Rust backend struct (camelCase serialization)
// ============================================

export type RunStatus = 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';

/**
 * Run — a headless batch execution of an agent
 */
export interface Run {
    id: string;
    agentId: string;
    sessionId: string | null;
    name: string;
    input: string;
    status: RunStatus;
    output: string | null;
    error: string | null;
    totalEvents: number;
    totalTokens: number;
    totalCostUsd: number;
    durationMs: number | null;
    createdAt: string;
    startedAt: string | null;
    completedAt: string | null;
    agentName: string | null;
}

/**
 * Request to create and execute a new run
 */
export interface CreateRunRequest {
    agentId: string;
    input: string;
    name?: string;
}
