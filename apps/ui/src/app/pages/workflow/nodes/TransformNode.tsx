import { Handle, Position } from '@xyflow/react';
import { Repeat } from 'lucide-react';
import { NodeShell, OutputPreview } from './NodeShell';

export function TransformNode({ id, data, selected }: { id: string; data: Record<string, unknown>; selected?: boolean }) {
    const inputs = (data.inputs as string[]) || ['input'];

    return (
        <NodeShell id={id} type="transform" label="TRANSFORM" icon={Repeat} selected={selected}
            collapsed={data.collapsed as boolean}>
            {inputs.map((inputName) => (
                <div key={inputName} className="handle-row input">
                    <Handle type="target" position={Position.Left} id={inputName} className="custom-handle handle-any" />
                    <span className="handle-label">{inputName}</span>
                </div>
            ))}
            <div className="text-[10px] text-[#888] mt-1">Mode: {(data.mode as string) || 'template'}</div>
            {Boolean(data.template) && (
                <div className="text-[10px] mt-0.5 truncate max-w-[160px] font-mono text-[#777]">
                    {(data.template as string).slice(0, 30)}
                </div>
            )}
            <OutputPreview nodeId={id} />
            <div className="handle-row output">
                <span className="handle-label">output</span>
                <Handle type="source" position={Position.Right} className="custom-handle handle-any" />
            </div>
        </NodeShell>
    );
}
