import { Handle, Position } from '@xyflow/react';
import { FileOutput } from 'lucide-react';
import { NodeShell } from './NodeShell';
import { useNodeData } from '../hooks/useNodeData';
import { OutputPreview } from './NodeShell';

export function FileWriteNode({ id, data, selected }: { id: string; data: Record<string, unknown>; selected?: boolean }) {
    const { updateField } = useNodeData(id);

    return (
        <NodeShell id={id} type="file_write" label="FILE WRITE" icon={FileOutput} selected={selected}
            collapsed={data.collapsed as boolean}>
            <div className="handle-row input">
                <Handle type="target" position={Position.Left} id="path"
                    className="custom-handle handle-text" title="text" />
                <span className="handle-label">path</span>
            </div>
            <div className="handle-row input">
                <Handle type="target" position={Position.Left} id="content"
                    className="custom-handle handle-any" title="any" />
                <span className="handle-label">content</span>
            </div>

            <div onClick={e => e.stopPropagation()}>
                <input className="node-inline-input" value={(data.path as string) || ''}
                    placeholder="/path/to/output.json" onChange={e => updateField('path', e.target.value)}
                    onMouseDown={e => e.stopPropagation()} />
                <select className="node-inline-input" value={(data.mode as string) || 'text'}
                    onChange={e => updateField('mode', e.target.value)}
                    onMouseDown={e => e.stopPropagation()}>
                    <option value="text">Text</option>
                    <option value="json">JSON</option>
                    <option value="csv">CSV</option>
                </select>
            </div>

            <OutputPreview nodeId={id} />

            <div className="handle-row output">
                <span className="handle-label">path</span>
                <Handle type="source" position={Position.Right} id="path"
                    className="custom-handle handle-text" title="text" />
            </div>
            <div className="handle-row output">
                <span className="handle-label">bytes</span>
                <Handle type="source" position={Position.Right} id="bytes"
                    className="custom-handle handle-float" title="number" />
            </div>
        </NodeShell>
    );
}
