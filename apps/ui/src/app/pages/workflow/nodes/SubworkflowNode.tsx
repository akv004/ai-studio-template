import { Handle, Position } from '@xyflow/react';
import { MessageSquare } from 'lucide-react';
import { NodeShell } from './NodeShell';
import { useNodeData } from '../hooks/useNodeData';

export function SubworkflowNode({ id, data, selected }: { id: string; data: Record<string, unknown>; selected?: boolean }) {
    const { updateField } = useNodeData(id);

    return (
        <NodeShell id={id} type="subworkflow" label="SUBWORKFLOW" icon={MessageSquare} selected={selected}
            collapsed={data.collapsed as boolean}>
            <div className="handle-row input">
                <Handle type="target" position={Position.Left} className="custom-handle handle-any" title="any" />
                <span className="handle-label">input</span>
            </div>
            <div onClick={e => e.stopPropagation()}>
                <input className="node-inline-input" value={(data.workflowName as string) || ''}
                    placeholder="workflow name" onChange={e => updateField('workflowName', e.target.value)}
                    onMouseDown={e => e.stopPropagation()} />
            </div>
            <div className="handle-row output">
                <span className="handle-label">output</span>
                <Handle type="source" position={Position.Right} className="custom-handle handle-any" title="any" />
            </div>
        </NodeShell>
    );
}
