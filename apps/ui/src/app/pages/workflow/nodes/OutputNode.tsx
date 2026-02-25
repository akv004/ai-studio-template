import { Handle, Position } from '@xyflow/react';
import { FileOutput } from 'lucide-react';
import { NodeShell, OutputPreview } from './NodeShell';
import { useNodeData } from '../hooks/useNodeData';

export function OutputNode({ id, data, selected }: { id: string; data: Record<string, unknown>; selected?: boolean }) {
    const { updateField } = useNodeData(id);

    return (
        <NodeShell id={id} type="output" label="OUTPUT" icon={FileOutput} selected={selected}
            collapsed={data.collapsed as boolean} customLabel={(data.label as string) || ''}>
            <div className="handle-row input">
                <Handle type="target" position={Position.Left} id="value" className="custom-handle handle-text" title="text" />
                <span className="handle-label">value</span>
            </div>
            <div className="flex flex-col gap-1" onClick={e => e.stopPropagation()}>
                <input className="node-inline-input" value={(data.name as string) || ''}
                    placeholder="name" onChange={e => updateField('name', e.target.value)}
                    onMouseDown={e => e.stopPropagation()} />
                <select className="node-inline-input" value={(data.format as string) || 'text'}
                    onChange={e => updateField('format', e.target.value)}
                    onMouseDown={e => e.stopPropagation()}>
                    <option value="text">Text</option>
                    <option value="markdown">Markdown</option>
                    <option value="json">JSON</option>
                </select>
            </div>
            <OutputPreview nodeId={id} />
        </NodeShell>
    );
}
