import { Handle, Position } from '@xyflow/react';
import { GitFork } from 'lucide-react';
import { NodeShell } from './NodeShell';

export function RouterNode({ id, data, selected }: { id: string; data: Record<string, unknown>; selected?: boolean }) {
    const branches = (data.branches as string[]) || ['true', 'false'];
    return (
        <NodeShell id={id} type="router" label="ROUTER" icon={GitFork} selected={selected}
            collapsed={data.collapsed as boolean}>
            <div className="handle-row input">
                <Handle type="target" position={Position.Left} className="custom-handle handle-text" />
                <span className="handle-label">input</span>
            </div>
            <div className="text-[10px] text-[#888]">Mode: {(data.mode as string) || 'llm'}</div>
            {branches.map((b, i) => (
                <div key={i} className="handle-row output">
                    <span className="handle-label">{b}</span>
                    <Handle type="source" position={Position.Right} id={`branch-${i}`}
                        className="custom-handle handle-bool" />
                </div>
            ))}
        </NodeShell>
    );
}
