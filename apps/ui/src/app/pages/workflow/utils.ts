let nodeIdCounter = 0;

export function generateNodeId(type: string): string {
    return `${type}_${++nodeIdCounter}_${Date.now().toString(36)}`;
}

export function formatRuntimeError(error: unknown): string {
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
