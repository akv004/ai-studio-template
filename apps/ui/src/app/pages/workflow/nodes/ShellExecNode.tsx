import { Handle, Position } from '@xyflow/react';
import { Terminal } from 'lucide-react';
import { NodeShell } from './NodeShell';
import { useNodeData } from '../hooks/useNodeData';
import { OutputPreview } from './NodeShell';

export function ShellExecNode({ id, data, selected }: { id: string; data: Record<string, unknown>; selected?: boolean }) {
    const { updateField } = useNodeData(id);

    return (
        <NodeShell id={id} type="shell_exec" label="SHELL EXEC" icon={Terminal} selected={selected}
            collapsed={data.collapsed as boolean} customLabel={(data.label as string) || ''}>
            <div className="handle-row input">
                <Handle type="target" position={Position.Left} id="command"
                    className="custom-handle handle-text" title="text" />
                <span className="handle-label">command</span>
            </div>
            <div className="handle-row input">
                <Handle type="target" position={Position.Left} id="stdin"
                    className="custom-handle handle-text" title="text" />
                <span className="handle-label">stdin</span>
            </div>

            <div onClick={e => e.stopPropagation()}>
                <input className="node-inline-input font-mono" value={(data.command as string) || ''}
                    placeholder="echo hello" onChange={e => updateField('command', e.target.value)}
                    onMouseDown={e => e.stopPropagation()} />
                <div className="flex gap-1 mt-1">
                    <select className="node-inline-input" style={{ flex: 1 }}
                        value={(data.shell as string) || 'bash'}
                        onChange={e => updateField('shell', e.target.value)}
                        onMouseDown={e => e.stopPropagation()}>
                        <option value="bash">bash</option>
                        <option value="sh">sh</option>
                        <option value="zsh">zsh</option>
                    </select>
                    <input className="node-inline-input" style={{ width: 50 }} type="number"
                        value={(data.timeout as number) ?? 30}
                        title="Timeout (seconds)"
                        onChange={e => updateField('timeout', parseInt(e.target.value) || 30)}
                        onMouseDown={e => e.stopPropagation()} />
                </div>
            </div>

            <OutputPreview nodeId={id} />

            <div className="handle-row output">
                <span className="handle-label">stdout</span>
                <Handle type="source" position={Position.Right} id="stdout"
                    className="custom-handle handle-text" title="text" />
            </div>
            <div className="handle-row output">
                <span className="handle-label">stderr</span>
                <Handle type="source" position={Position.Right} id="stderr"
                    className="custom-handle handle-text" title="text" />
            </div>
            <div className="handle-row output">
                <span className="handle-label">exit_code</span>
                <Handle type="source" position={Position.Right} id="exit_code"
                    className="custom-handle handle-float" title="number" />
            </div>
        </NodeShell>
    );
}
