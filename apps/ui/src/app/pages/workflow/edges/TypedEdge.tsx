import { type EdgeProps, getBezierPath } from '@xyflow/react';
import { useAppStore } from '../../../../state/store';

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

export function TypedEdge({
    id,
    sourceX, sourceY,
    targetX, targetY,
    sourcePosition, targetPosition,
    data,
    selected,
    source,
}: EdgeProps) {
    const handleType = (data?.handleType as string) || 'any';
    const color = HANDLE_COLORS[handleType] || HANDLE_COLORS.any;

    const sourceNodeState = useAppStore(s => s.workflowNodeStates[source]);
    const isRunning = sourceNodeState?.status === 'running';
    const isCompleted = sourceNodeState?.status === 'completed';
    const isError = sourceNodeState?.status === 'error';
    const workflowRunning = useAppStore(s => s.workflowRunning);

    const [edgePath] = getBezierPath({
        sourceX, sourceY,
        targetX, targetY,
        sourcePosition, targetPosition,
    });

    const strokeColor = isError ? '#EF4444' : color;
    const strokeWidth = selected ? 3 : 2.5;
    const animated = workflowRunning && (isRunning || isCompleted);

    return (
        <>
            {/* Wider invisible path for easier selection */}
            <path
                id={`${id}-hitarea`}
                d={edgePath}
                fill="none"
                stroke="transparent"
                strokeWidth={20}
                className="react-flow__edge-interaction"
            />
            {/* Glow layer */}
            <path
                d={edgePath}
                fill="none"
                stroke={strokeColor}
                strokeWidth={strokeWidth + 3}
                strokeOpacity={selected ? 0.25 : 0.1}
                filter="url(#edge-glow)"
            />
            {/* Main edge */}
            <path
                id={id}
                d={edgePath}
                fill="none"
                stroke={strokeColor}
                strokeWidth={strokeWidth}
                strokeLinecap="round"
                className={`react-flow__edge-path ${animated ? 'edge-animated' : ''} ${isCompleted && workflowRunning ? 'edge-completed' : ''}`}
            />
            {/* SVG filter for glow effect */}
            <defs>
                <filter id="edge-glow" x="-20%" y="-20%" width="140%" height="140%">
                    <feGaussianBlur stdDeviation="3" result="blur" />
                    <feMerge>
                        <feMergeNode in="blur" />
                        <feMergeNode in="SourceGraphic" />
                    </feMerge>
                </filter>
            </defs>
        </>
    );
}
