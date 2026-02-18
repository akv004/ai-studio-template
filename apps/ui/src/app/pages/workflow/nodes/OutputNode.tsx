import { Handle, Position } from '@xyflow/react';
import { FileOutput } from 'lucide-react';
import { NodeShell, OutputPreview } from './NodeShell';

export function OutputNode({ id, data, selected }: { id: string; data: Record<string, unknown>; selected?: boolean }) {
    return (
        <NodeShell id={id} type="output" label="OUTPUT" icon={FileOutput} selected={selected}
            collapsed={data.collapsed as boolean}>
            <div className="handle-row input">
                <Handle type="target" position={Position.Left} className="custom-handle handle-text" />
                <span className="handle-label">value</span>
            </div>
            <div className="text-[11px] font-medium">{(data.name as string) || 'result'}</div>
            <div className="text-[10px] text-[#888]">Format: {(data.format as string) || 'text'}</div>
            <OutputPreview nodeId={id} />
        </NodeShell>
    );
}
