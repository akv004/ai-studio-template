import type { ConnectionLineComponentProps } from '@xyflow/react';

const HANDLE_COLORS: Record<string, string> = {
    text: '#E879F9',
    json: '#F59E0B',
    bool: '#EF4444',
    float: '#10B981',
    number: '#06B6D4',
    rows: '#14B8A6',
    binary: '#8B5CF6',
    any: '#9CA3AF',
    exec: '#FFFFFF',
};

function getSourceHandleType(fromNodeId: string, fromHandleId: string | null): string {
    const selector = fromHandleId
        ? `[data-nodeid="${fromNodeId}"] .react-flow__handle[data-handleid="${fromHandleId}"].source`
        : `[data-nodeid="${fromNodeId}"] .react-flow__handle.source`;
    const el = document.querySelector(selector);
    if (!el) return 'any';
    const classes = el.className;
    if (classes.includes('handle-text')) return 'text';
    if (classes.includes('handle-json')) return 'json';
    if (classes.includes('handle-bool')) return 'bool';
    if (classes.includes('handle-float')) return 'float';
    if (classes.includes('handle-number')) return 'number';
    if (classes.includes('handle-rows')) return 'rows';
    if (classes.includes('handle-binary')) return 'binary';
    if (classes.includes('handle-exec')) return 'exec';
    return 'any';
}

export function TypedConnectionLine({
    fromX, fromY,
    toX, toY,
    fromNode,
    fromHandle,
}: ConnectionLineComponentProps) {
    const handleType = getSourceHandleType(
        fromNode?.id || '',
        fromHandle?.id || null,
    );
    const color = HANDLE_COLORS[handleType] || HANDLE_COLORS.any;

    // Simple bezier control points
    const dx = Math.abs(toX - fromX) * 0.5;
    const path = `M${fromX},${fromY} C${fromX + dx},${fromY} ${toX - dx},${toY} ${toX},${toY}`;

    return (
        <g>
            {/* Glow */}
            <path
                d={path}
                fill="none"
                stroke={color}
                strokeWidth={6}
                strokeOpacity={0.15}
            />
            {/* Dashed preview line */}
            <path
                d={path}
                fill="none"
                stroke={color}
                strokeWidth={2.5}
                strokeDasharray="6,4"
                strokeLinecap="round"
            />
            {/* Endpoint dot */}
            <circle
                cx={toX}
                cy={toY}
                r={4}
                fill={color}
                fillOpacity={0.6}
            />
        </g>
    );
}
