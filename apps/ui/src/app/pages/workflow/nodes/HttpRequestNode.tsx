import { Handle, Position } from '@xyflow/react';
import { Globe } from 'lucide-react';
import { NodeShell } from './NodeShell';
import { useNodeData } from '../hooks/useNodeData';
import { OutputPreview } from './NodeShell';

export function HttpRequestNode({ id, data, selected }: { id: string; data: Record<string, unknown>; selected?: boolean }) {
    const { updateField } = useNodeData(id);

    return (
        <NodeShell id={id} type="http_request" label="HTTP REQUEST" icon={Globe} selected={selected}
            collapsed={data.collapsed as boolean} customLabel={(data.label as string) || ''}>
            <div className="handle-row input">
                <Handle type="target" position={Position.Left} id="url"
                    className="custom-handle handle-text" title="text" />
                <span className="handle-label">url</span>
            </div>
            <div className="handle-row input">
                <Handle type="target" position={Position.Left} id="body"
                    className="custom-handle handle-json" title="json" />
                <span className="handle-label">body</span>
            </div>
            <div className="handle-row input">
                <Handle type="target" position={Position.Left} id="headers"
                    className="custom-handle handle-json" title="json" />
                <span className="handle-label">headers</span>
            </div>

            <div onClick={e => e.stopPropagation()}>
                <input className="node-inline-input" value={(data.url as string) || ''}
                    placeholder="https://api.example.com" onChange={e => updateField('url', e.target.value)}
                    onMouseDown={e => e.stopPropagation()} />
                <select className="node-inline-input" value={(data.method as string) || 'GET'}
                    onChange={e => updateField('method', e.target.value)}
                    onMouseDown={e => e.stopPropagation()}>
                    <option value="GET">GET</option>
                    <option value="POST">POST</option>
                    <option value="PUT">PUT</option>
                    <option value="PATCH">PATCH</option>
                    <option value="DELETE">DELETE</option>
                    <option value="HEAD">HEAD</option>
                </select>
            </div>

            <OutputPreview nodeId={id} />

            <div className="handle-row output">
                <span className="handle-label">body</span>
                <Handle type="source" position={Position.Right} id="body"
                    className="custom-handle handle-text" title="text" />
            </div>
            <div className="handle-row output">
                <span className="handle-label">status</span>
                <Handle type="source" position={Position.Right} id="status"
                    className="custom-handle handle-float" title="number" />
            </div>
            <div className="handle-row output">
                <span className="handle-label">headers</span>
                <Handle type="source" position={Position.Right} id="headers"
                    className="custom-handle handle-json" title="json" />
            </div>
        </NodeShell>
    );
}
