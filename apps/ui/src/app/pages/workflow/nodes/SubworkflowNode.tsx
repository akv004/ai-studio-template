import { Handle, Position } from '@xyflow/react';
import { MessageSquare } from 'lucide-react';
import { NodeShell } from './NodeShell';

export function SubworkflowNode({ id, data, selected }: { id: string; data: Record<string, unknown>; selected?: boolean }) {
    return (
        <NodeShell id={id} type="subworkflow" label="SUBWORKFLOW" icon={MessageSquare} selected={selected}
            collapsed={data.collapsed as boolean}>
            <div className="handle-row input">
                <Handle type="target" position={Position.Left} className="custom-handle handle-any" />
                <span className="handle-label">input</span>
            </div>
            <div className="text-[11px] font-medium">{(data.workflowName as string) || 'Select workflow'}</div>
            <div className="handle-row output">
                <span className="handle-label">output</span>
                <Handle type="source" position={Position.Right} className="custom-handle handle-any" />
            </div>
        </NodeShell>
    );
}
