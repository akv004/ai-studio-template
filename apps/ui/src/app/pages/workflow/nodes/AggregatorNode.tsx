import { Handle, Position } from '@xyflow/react';
import { Layers } from 'lucide-react';
import { NodeShell, OutputPreview } from './NodeShell';
import { useNodeData } from '../hooks/useNodeData';

export function AggregatorNode({ id, data, selected }: { id: string; data: Record<string, unknown>; selected?: boolean }) {
    const { updateField } = useNodeData(id);

    return (
        <NodeShell id={id} type="aggregator" label="AGGREGATOR" icon={Layers} selected={selected}
            collapsed={data.collapsed as boolean}>
            <div className="handle-row input">
                <Handle type="target" position={Position.Left} id="input"
                    className="custom-handle handle-any" title="any" />
                <span className="handle-label">input</span>
            </div>

            <div onClick={e => e.stopPropagation()}>
                <select className="node-inline-input" value={(data.strategy as string) || 'array'}
                    onChange={e => updateField('strategy', e.target.value)}
                    onMouseDown={e => e.stopPropagation()}>
                    <option value="array">Array (collect all)</option>
                    <option value="concat">Concat (join text)</option>
                    <option value="merge">Merge (combine objects)</option>
                </select>
                {(data.strategy as string) === 'concat' && (
                    <input className="node-inline-input" value={(data.separator as string) ?? '\\n'}
                        placeholder="Separator" onChange={e => updateField('separator', e.target.value)}
                        onMouseDown={e => e.stopPropagation()} />
                )}
            </div>

            <OutputPreview nodeId={id} />

            <div className="handle-row output">
                <span className="handle-label">result</span>
                <Handle type="source" position={Position.Right} id="result"
                    className="custom-handle handle-any" title="any" />
            </div>
            <div className="handle-row output">
                <span className="handle-label">count</span>
                <Handle type="source" position={Position.Right} id="count"
                    className="custom-handle handle-float" title="number" />
            </div>
        </NodeShell>
    );
}
