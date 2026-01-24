// ============================================
// SHARED TYPES - RUNS / TIMELINE
// Execution pipeline structures
// ============================================

/**
 * Run phase status
 */
export type PhaseStatus = 'pending' | 'running' | 'completed' | 'failed' | 'skipped';

/**
 * Log level
 */
export type LogLevel = 'debug' | 'info' | 'warning' | 'error';

/**
 * Run phase definition
 */
export interface RunPhase {
    id: string;
    name: string;
    status: PhaseStatus;
    startedAt?: string;
    completedAt?: string;
    durationMs?: number;
    logs: LogEntry[];
}

/**
 * Log entry
 */
export interface LogEntry {
    timestamp: string;
    level: LogLevel;
    message: string;
    source?: string;
}

/**
 * Complete run/execution
 */
export interface Run {
    id: string;
    name: string;
    projectId: string;
    status: PhaseStatus;
    phases: RunPhase[];
    startedAt: string;
    completedAt?: string;
    totalDurationMs?: number;
}
