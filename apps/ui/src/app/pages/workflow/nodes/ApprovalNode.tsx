import { Handle, Position } from '@xyflow/react';
import { ShieldCheck } from 'lucide-react';
import { NodeShell } from './NodeShell';

export function ApprovalNode({ id, data, selected }: { id: string; data: Record<string, unknown>; selected?: boolean }) {
    return (
        <NodeShell id={id} type="approval" label="APPROVAL" icon={ShieldCheck} selected={selected}
            collapsed={data.collapsed as boolean} customLabel={(data.label as string) || ''}>
            <div className="handle-row input">
                <Handle type="target" position={Position.Left} className="custom-handle handle-any" title="any" />
                <span className="handle-label">data</span>
            </div>
            <div className="text-[11px]">{((data.message as string) || 'Approve?').slice(0, 40)}</div>
            <div className="handle-row output">
                <span className="handle-label">approved</span>
                <Handle type="source" position={Position.Right} id="approved" className="custom-handle handle-bool" title="bool" />
            </div>
            <div className="handle-row output">
                <span className="handle-label">rejected</span>
                <Handle type="source" position={Position.Right} id="rejected" className="custom-handle handle-bool" title="bool" />
            </div>
        </NodeShell>
    );
}
