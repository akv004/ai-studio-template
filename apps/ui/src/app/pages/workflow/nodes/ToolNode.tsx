import { Handle, Position } from '@xyflow/react';
import { Wrench } from 'lucide-react';
import { NodeShell, OutputPreview } from './NodeShell';

export function ToolNode({ id, data, selected }: { id: string; data: Record<string, unknown>; selected?: boolean }) {
    const toolName = (data.toolName as string) || '';
    // Parse "server__name" → friendly display
    const parts = toolName.split('__');
    const displayName = parts.length >= 2 ? parts.slice(1).join('__') : toolName;
    const serverName = parts.length >= 2 ? parts[0] : (data.serverName as string) || '';

    return (
        <NodeShell id={id} type="tool" label="TOOL" icon={Wrench} selected={selected}
            collapsed={data.collapsed as boolean} customLabel={(data.label as string) || ''}>
            <div className="handle-row input">
                <Handle type="target" position={Position.Left} id="input" className="custom-handle handle-json" title="json" />
                <span className="handle-label">input</span>
            </div>
            {toolName ? (
                <div className="px-1">
                    <div className="text-[11px] text-[var(--text-primary)] font-medium">{displayName}</div>
                    {serverName && (
                        <div className="text-[9px] text-[var(--text-muted)]">{serverName}</div>
                    )}
                </div>
            ) : (
                <div className="text-[10px] text-[var(--text-muted)] px-1 italic">No tool selected</div>
            )}
            <OutputPreview nodeId={id} />
            <div className="handle-row output">
                <span className="handle-label">result</span>
                <Handle type="source" position={Position.Right} id="result" className="custom-handle handle-json" title="json" />
            </div>
        </NodeShell>
    );
}
