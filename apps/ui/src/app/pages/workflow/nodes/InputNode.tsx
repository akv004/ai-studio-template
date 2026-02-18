import { Handle, Position } from '@xyflow/react';
import { FileInput } from 'lucide-react';
import { NodeShell, OutputPreview } from './NodeShell';

export function InputNode({ id, data, selected }: { id: string; data: Record<string, unknown>; selected?: boolean }) {
    return (
        <NodeShell id={id} type="input" label="INPUT" icon={FileInput} selected={selected}
            collapsed={data.collapsed as boolean}>
            <div className="text-[11px] font-medium">{(data.name as string) || 'untitled'}</div>
            <div className="text-[10px] text-[#888]">Type: {(data.dataType as string) || 'text'}</div>
            <OutputPreview nodeId={id} />
            <div className="handle-row output">
                <span className="handle-label">value</span>
                <Handle type="source" position={Position.Right} className="custom-handle handle-text" />
            </div>
        </NodeShell>
    );
}
