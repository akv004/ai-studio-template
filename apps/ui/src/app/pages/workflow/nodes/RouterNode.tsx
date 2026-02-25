import { Handle, Position } from '@xyflow/react';
import { GitFork } from 'lucide-react';
import { NodeShell } from './NodeShell';
import { useNodeData } from '../hooks/useNodeData';

export function RouterNode({ id, data, selected }: { id: string; data: Record<string, unknown>; selected?: boolean }) {
    const { updateField } = useNodeData(id);
    const branches = (data.branches as string[]) || ['true', 'false'];

    return (
        <NodeShell id={id} type="router" label="ROUTER" icon={GitFork} selected={selected}
            collapsed={data.collapsed as boolean} customLabel={(data.label as string) || ''}>
            <div className="handle-row input">
                <Handle type="target" position={Position.Left} id="input" className="custom-handle handle-text" title="text" />
                <span className="handle-label">input</span>
            </div>
            <div onClick={e => e.stopPropagation()}>
                <select className="node-inline-input" value={(data.mode as string) || 'pattern'}
                    onChange={e => updateField('mode', e.target.value)}
                    onMouseDown={e => e.stopPropagation()}>
                    <option value="pattern">Pattern</option>
                    <option value="llm">LLM</option>
                </select>
            </div>
            {branches.map((b, i) => (
                <div key={i} className="handle-row output">
                    <span className="handle-label">{b}</span>
                    <Handle type="source" position={Position.Right} id={`branch-${i}`}
                        className="custom-handle handle-bool" title="bool" />
                </div>
            ))}
        </NodeShell>
    );
}
