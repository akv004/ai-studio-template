import { Handle, Position } from '@xyflow/react';
import { Repeat } from 'lucide-react';
import { NodeShell, OutputPreview } from './NodeShell';
import { useNodeData } from '../hooks/useNodeData';

export function IteratorNode({ id, data, selected }: { id: string; data: Record<string, unknown>; selected?: boolean }) {
    const { updateField } = useNodeData(id);

    return (
        <NodeShell id={id} type="iterator" label="ITERATOR" icon={Repeat} selected={selected}
            collapsed={data.collapsed as boolean} customLabel={(data.label as string) || ''}>
            <div className="handle-row input">
                <Handle type="target" position={Position.Left} id="items"
                    className="custom-handle handle-json" title="json" />
                <span className="handle-label">items</span>
            </div>

            <div onClick={e => e.stopPropagation()}>
                <select className="node-inline-input" value={(data.mode as string) || 'sequential'}
                    onChange={e => updateField('mode', e.target.value)}
                    onMouseDown={e => e.stopPropagation()}>
                    <option value="sequential">Sequential</option>
                    <option value="parallel">Parallel</option>
                </select>
                {(data.expression as string) !== undefined && (
                    <input className="node-inline-input" value={(data.expression as string) || ''}
                        placeholder="JSONPath (optional)" onChange={e => updateField('expression', e.target.value)}
                        onMouseDown={e => e.stopPropagation()} />
                )}
            </div>

            <OutputPreview nodeId={id} />

            <div className="handle-row output">
                <span className="handle-label">item</span>
                <Handle type="source" position={Position.Right} id="output"
                    className="custom-handle handle-any" title="any" />
            </div>
        </NodeShell>
    );
}
