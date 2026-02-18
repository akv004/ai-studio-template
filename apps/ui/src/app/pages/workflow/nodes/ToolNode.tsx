import { Handle, Position } from '@xyflow/react';
import { Wrench } from 'lucide-react';
import { NodeShell, OutputPreview } from './NodeShell';

export function ToolNode({ id, data, selected }: { id: string; data: Record<string, unknown>; selected?: boolean }) {
    return (
        <NodeShell id={id} type="tool" label="TOOL" icon={Wrench} selected={selected}
            collapsed={data.collapsed as boolean}>
            <div className="handle-row input">
                <Handle type="target" position={Position.Left} className="custom-handle handle-json" />
                <span className="handle-label">input</span>
            </div>
            <div className="text-[11px] font-medium">{(data.toolName as string) || 'Select tool'}</div>
            {Boolean(data.serverName) && (
                <div className="text-[10px] text-[#888]">Server: {data.serverName as string}</div>
            )}
            <OutputPreview nodeId={id} />
            <div className="handle-row output">
                <span className="handle-label">result</span>
                <Handle type="source" position={Position.Right} id="result" className="custom-handle handle-json" />
            </div>
        </NodeShell>
    );
}
