import { Handle, Position } from '@xyflow/react';
import { LogOut } from 'lucide-react';
import { NodeShell, OutputPreview } from './NodeShell';

export function ExitNode({ id, data, selected }: { id: string; data: Record<string, unknown>; selected?: boolean }) {
    return (
        <NodeShell id={id} type="exit" label="EXIT" icon={LogOut} selected={selected}
            collapsed={data.collapsed as boolean} customLabel={(data.label as string) || ''}>
            <div className="handle-row input">
                <Handle type="target" position={Position.Left} id="input"
                    className="custom-handle handle-any" title="any" />
                <span className="handle-label">input</span>
            </div>

            <OutputPreview nodeId={id} />

            <div className="handle-row output">
                <span className="handle-label">output</span>
                <Handle type="source" position={Position.Right} id="output"
                    className="custom-handle handle-any" title="any" />
            </div>
        </NodeShell>
    );
}
