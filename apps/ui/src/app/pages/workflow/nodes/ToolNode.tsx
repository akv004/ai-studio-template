import { Handle, Position } from '@xyflow/react';
import { Wrench } from 'lucide-react';
import { NodeShell, OutputPreview } from './NodeShell';
import { useNodeData } from '../hooks/useNodeData';

export function ToolNode({ id, data, selected }: { id: string; data: Record<string, unknown>; selected?: boolean }) {
    const { updateField } = useNodeData(id);

    return (
        <NodeShell id={id} type="tool" label="TOOL" icon={Wrench} selected={selected}
            collapsed={data.collapsed as boolean}>
            <div className="handle-row input">
                <Handle type="target" position={Position.Left} className="custom-handle handle-json" title="json" />
                <span className="handle-label">input</span>
            </div>
            <div onClick={e => e.stopPropagation()}>
                <input className="node-inline-input" value={(data.toolName as string) || ''}
                    placeholder="tool name" onChange={e => updateField('toolName', e.target.value)}
                    onMouseDown={e => e.stopPropagation()} />
            </div>
            {Boolean(data.serverName) && (
                <div className="text-[10px] text-[#888]">Server: {data.serverName as string}</div>
            )}
            <OutputPreview nodeId={id} />
            <div className="handle-row output">
                <span className="handle-label">result</span>
                <Handle type="source" position={Position.Right} id="result" className="custom-handle handle-json" title="json" />
            </div>
        </NodeShell>
    );
}
