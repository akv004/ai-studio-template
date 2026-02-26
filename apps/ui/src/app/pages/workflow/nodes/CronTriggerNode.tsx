import { Handle, Position } from '@xyflow/react';
import { Clock, Circle } from 'lucide-react';
import { NodeShell, OutputPreview } from './NodeShell';

const CRON_PRESETS: Record<string, string> = {
    '*/5 * * * *': 'Every 5 minutes',
    '0 * * * *': 'Every hour',
    '0 9 * * *': 'Daily at 9:00 AM',
    '0 18 * * 1-5': 'Weekdays at 6:00 PM',
    '0 8 * * 1': 'Weekly Monday 8:00 AM',
    '0 0 1 * *': 'First of month midnight',
};

function describeExpression(expression: string): string {
    return CRON_PRESETS[expression] || expression || 'No schedule';
}

export function CronTriggerNode({ id, data, selected }: { id: string; data: Record<string, unknown>; selected?: boolean }) {
    const expression = (data.expression as string) || '';
    const timezone = (data.timezone as string) || 'UTC';
    const armed = (data._armed as boolean) || false;

    return (
        <NodeShell id={id} type="cron_trigger" label="CRON" icon={Clock} selected={selected}
            collapsed={data.collapsed as boolean} customLabel={(data.label as string) || ''}>

            <div className="flex flex-col gap-1" onClick={e => e.stopPropagation()}>
                {/* Status */}
                <div className="flex items-center gap-1.5">
                    <Circle
                        size={8}
                        fill={armed ? '#22c55e' : '#666'}
                        stroke={armed ? '#22c55e' : '#666'}
                    />
                    <span className="text-[10px] text-[var(--text-muted)]">
                        {armed ? 'Armed' : 'Disarmed'}
                    </span>
                </div>

                {/* Schedule description */}
                <div className="text-[11px] text-[var(--text-secondary)] font-medium">
                    {describeExpression(expression)}
                </div>

                {/* Expression + timezone */}
                <div className="flex items-center gap-1 px-1 py-0.5 rounded bg-[#1e1e1e] border border-[#3a3a3a]">
                    <span className="text-[9px] font-mono text-[#888] truncate flex-1">
                        {expression || '0 9 * * *'}
                    </span>
                    <span className="text-[9px] text-[#666] shrink-0">
                        {timezone}
                    </span>
                </div>
            </div>

            <OutputPreview nodeId={id} />

            {/* Source handles — cron trigger is an entry point */}
            <div className="handle-row output">
                <span className="handle-label">timestamp</span>
                <Handle type="source" position={Position.Right} id="timestamp"
                    className="custom-handle handle-text" title="text" />
            </div>
            <div className="handle-row output">
                <span className="handle-label">iteration</span>
                <Handle type="source" position={Position.Right} id="iteration"
                    className="custom-handle handle-number" title="number" />
            </div>
            <div className="handle-row output">
                <span className="handle-label">input</span>
                <Handle type="source" position={Position.Right} id="input"
                    className="custom-handle handle-json" title="json" />
            </div>
            <div className="handle-row output">
                <span className="handle-label">schedule</span>
                <Handle type="source" position={Position.Right} id="schedule"
                    className="custom-handle handle-text" title="text" />
            </div>
        </NodeShell>
    );
}
