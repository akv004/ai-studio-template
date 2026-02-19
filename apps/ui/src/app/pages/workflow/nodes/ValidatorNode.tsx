import { Handle, Position } from '@xyflow/react';
import { ShieldCheck } from 'lucide-react';
import { NodeShell } from './NodeShell';
import { useNodeData } from '../hooks/useNodeData';
import { OutputPreview } from './NodeShell';

export function ValidatorNode({ id, data, selected }: { id: string; data: Record<string, unknown>; selected?: boolean }) {
    const { updateField } = useNodeData(id);

    return (
        <NodeShell id={id} type="validator" label="VALIDATOR" icon={ShieldCheck} selected={selected}
            collapsed={data.collapsed as boolean}>
            <div className="handle-row input">
                <Handle type="target" position={Position.Left} id="data"
                    className="custom-handle handle-json" title="json" />
                <span className="handle-label">data</span>
            </div>

            <div onClick={e => e.stopPropagation()}>
                <textarea className="node-inline-input font-mono" style={{ minHeight: 48, fontSize: 10 }}
                    value={(data.schema as string) || '{}'}
                    placeholder='{"type":"object","required":["name"]}'
                    onChange={e => updateField('schema', e.target.value)}
                    onMouseDown={e => e.stopPropagation()} />
            </div>

            <OutputPreview nodeId={id} />

            <div className="handle-row output">
                <span className="handle-label">valid</span>
                <Handle type="source" position={Position.Right} id="valid"
                    className="custom-handle handle-bool" title="bool" />
            </div>
            <div className="handle-row output">
                <span className="handle-label">data</span>
                <Handle type="source" position={Position.Right} id="data"
                    className="custom-handle handle-json" title="json" />
            </div>
            <div className="handle-row output">
                <span className="handle-label">errors</span>
                <Handle type="source" position={Position.Right} id="errors"
                    className="custom-handle handle-json" title="json" />
            </div>
        </NodeShell>
    );
}
