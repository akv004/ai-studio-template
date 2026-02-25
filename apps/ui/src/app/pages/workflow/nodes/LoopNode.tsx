import { Handle, Position } from '@xyflow/react';
import { RefreshCw } from 'lucide-react';
import { NodeShell, OutputPreview } from './NodeShell';
import { useNodeData } from '../hooks/useNodeData';

export function LoopNode({ id, data, selected }: { id: string; data: Record<string, unknown>; selected?: boolean }) {
    const { updateField } = useNodeData(id);

    return (
        <NodeShell id={id} type="loop" label="LOOP" icon={RefreshCw} selected={selected}
            collapsed={data.collapsed as boolean} customLabel={(data.label as string) || ''}>
            <div className="handle-row input">
                <Handle type="target" position={Position.Left} id="input"
                    className="custom-handle handle-any" title="any" />
                <span className="handle-label">input</span>
            </div>

            <div onClick={e => e.stopPropagation()}>
                <div className="flex gap-1 items-center">
                    <span className="text-[9px] text-[var(--text-muted)] whitespace-nowrap">Max</span>
                    <input type="number" className="node-inline-input w-12 text-center"
                        min={1} max={50}
                        value={(data.maxIterations as number) ?? 5}
                        onChange={e => updateField('maxIterations', parseInt(e.target.value) || 5)}
                        onMouseDown={e => e.stopPropagation()} />
                </div>
                <select className="node-inline-input" value={(data.exitCondition as string) || 'max_iterations'}
                    onChange={e => updateField('exitCondition', e.target.value)}
                    onMouseDown={e => e.stopPropagation()}>
                    <option value="max_iterations">Max Iterations</option>
                    <option value="evaluator">Evaluator (Router)</option>
                    <option value="stable_output">Stable Output</option>
                </select>
                <select className="node-inline-input" value={(data.feedbackMode as string) || 'replace'}
                    onChange={e => updateField('feedbackMode', e.target.value)}
                    onMouseDown={e => e.stopPropagation()}>
                    <option value="replace">Replace</option>
                    <option value="append">Append</option>
                </select>
            </div>

            <OutputPreview nodeId={id} />

            <div className="handle-row output">
                <span className="handle-label">output</span>
                <Handle type="source" position={Position.Right} id="output"
                    className="custom-handle handle-any" title="any" />
            </div>
            <div className="handle-row output">
                <span className="handle-label">iterations</span>
                <Handle type="source" position={Position.Right} id="iterations"
                    className="custom-handle handle-json" title="json" />
            </div>
            <div className="handle-row output">
                <span className="handle-label">count</span>
                <Handle type="source" position={Position.Right} id="count"
                    className="custom-handle handle-float" title="number" />
            </div>
        </NodeShell>
    );
}
