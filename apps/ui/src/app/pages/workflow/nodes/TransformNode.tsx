import { Handle, Position } from '@xyflow/react';
import { Repeat } from 'lucide-react';
import { NodeShell, OutputPreview } from './NodeShell';
import { useNodeData } from '../hooks/useNodeData';

export function TransformNode({ id, data, selected }: { id: string; data: Record<string, unknown>; selected?: boolean }) {
    const { updateField } = useNodeData(id);
    const inputs = (data.inputs as string[]) || ['input'];

    return (
        <NodeShell id={id} type="transform" label="TRANSFORM" icon={Repeat} selected={selected}
            collapsed={data.collapsed as boolean} customLabel={(data.label as string) || ''}>
            {inputs.map((inputName) => (
                <div key={inputName} className="handle-row input">
                    <Handle type="target" position={Position.Left} id={inputName} className="custom-handle handle-any" title="any" />
                    <span className="handle-label">{inputName}</span>
                </div>
            ))}
            <div onClick={e => e.stopPropagation()}>
                <select className="node-inline-input" value={(data.mode as string) || 'template'}
                    onChange={e => updateField('mode', e.target.value)}
                    onMouseDown={e => e.stopPropagation()}>
                    <option value="template">Template</option>
                    <option value="jsonpath">JSONPath</option>
                    <option value="script">Expression</option>
                </select>
            </div>
            {Boolean(data.template) && (
                <div className="text-[10px] mt-0.5 truncate max-w-[160px] font-mono text-[#777]">
                    {(data.template as string).slice(0, 30)}
                </div>
            )}
            <OutputPreview nodeId={id} />
            <div className="handle-row output">
                <span className="handle-label">output</span>
                <Handle type="source" position={Position.Right} id="output" className="custom-handle handle-any" title="any" />
            </div>
        </NodeShell>
    );
}
