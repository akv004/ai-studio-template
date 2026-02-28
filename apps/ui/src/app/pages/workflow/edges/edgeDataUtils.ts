/**
 * Edge data preview utilities for X-Ray mode.
 * Mirrors Rust resolve_source_handle logic for extracting handle-specific values.
 */

export function resolveHandleValue(nodeOutput: unknown, sourceHandle: string): unknown {
    if (nodeOutput == null) return undefined;

    // Default handle: return whole value
    if (sourceHandle === 'output') return nodeOutput;

    // Structured output: select specific handle field
    if (typeof nodeOutput === 'object' && !Array.isArray(nodeOutput)) {
        const obj = nodeOutput as Record<string, unknown>;
        if (sourceHandle in obj) return obj[sourceHandle];

        // Router backward compat: branch-* handles unwrap "value" field
        if (sourceHandle.startsWith('branch-') && 'value' in obj) {
            return obj.value;
        }
    }

    // Fallback: whole value
    return nodeOutput;
}

export function formatPreview(value: unknown, maxLen = 40): string {
    if (value == null) return '(empty)';

    if (typeof value === 'string') {
        if (value.length === 0) return '(empty)';
        return value.length > maxLen ? value.slice(0, maxLen) + '\u2026' : value;
    }

    if (typeof value === 'number' || typeof value === 'boolean') {
        return String(value);
    }

    if (Array.isArray(value)) {
        const len = value.length;
        if (len === 0) return '[ ]';
        const first = formatPreview(value[0], 20);
        return `[${len} item${len !== 1 ? 's' : ''}] ${first}`;
    }

    if (typeof value === 'object') {
        const keys = Object.keys(value as Record<string, unknown>);
        if (keys.length === 0) return '{ }';
        const firstKey = keys[0];
        return `{${keys.length} key${keys.length !== 1 ? 's' : ''}} ${firstKey}`;
    }

    return String(value);
}

export function formatFullPreview(value: unknown, maxLen = 500): string {
    if (value == null) return '(empty)';

    if (typeof value === 'string') {
        return value.length > maxLen ? value.slice(0, maxLen) + '\u2026' : value;
    }

    try {
        const json = JSON.stringify(value, null, 2);
        return json.length > maxLen ? json.slice(0, maxLen) + '\u2026' : json;
    } catch {
        return String(value);
    }
}

export function formatDetailPreview(value: unknown, maxLen = 5000): string {
    if (value == null) return '(empty)';

    if (typeof value === 'string') {
        return value.length > maxLen ? value.slice(0, maxLen) + '\u2026' : value;
    }

    try {
        const json = JSON.stringify(value, null, 2);
        return json.length > maxLen ? json.slice(0, maxLen) + '\u2026' : json;
    } catch {
        return String(value);
    }
}

export function getDataTypeLabel(value: unknown): string {
    if (value == null) return 'null';
    if (typeof value === 'string') return `string (${value.length} chars)`;
    if (typeof value === 'number') return 'number';
    if (typeof value === 'boolean') return 'boolean';
    if (Array.isArray(value)) return `array (${value.length} items)`;
    if (typeof value === 'object') {
        const keys = Object.keys(value as Record<string, unknown>);
        return `object (${keys.length} keys)`;
    }
    return typeof value;
}
