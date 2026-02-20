import { Handle, Position } from '@xyflow/react';
import { FileInput } from 'lucide-react';
import { useCallback } from 'react';
import { NodeShell, OutputPreview } from './NodeShell';
import { useNodeData } from '../hooks/useNodeData';

export function InputNode({ id, data, selected }: { id: string; data: Record<string, unknown>; selected?: boolean }) {
    const { updateField } = useNodeData(id);

    const autoResize = useCallback((el: HTMLTextAreaElement | null) => {
        if (!el) return;
        el.style.height = 'auto';
        el.style.height = Math.min(el.scrollHeight, 100) + 'px';
    }, []);

    return (
        <NodeShell id={id} type="input" label="INPUT" icon={FileInput} selected={selected}
            collapsed={data.collapsed as boolean} customLabel={(data.label as string) || ''}>
            <div className="flex flex-col gap-1" onClick={e => e.stopPropagation()}>
                <textarea
                    ref={autoResize}
                    className="node-inline-input resize-none overflow-hidden leading-snug"
                    rows={1}
                    value={(data.defaultValue as string) || ''}
                    placeholder="Enter value..."
                    onChange={e => { updateField('defaultValue', e.target.value); autoResize(e.target); }}
                    onMouseDown={e => e.stopPropagation()}
                />
                <input className="node-inline-input text-[10px] opacity-60" value={(data.name as string) || ''}
                    placeholder="Variable name (optional)"
                    onChange={e => updateField('name', e.target.value)}
                    onMouseDown={e => e.stopPropagation()} />
                <select className="node-inline-input" value={(data.dataType as string) || 'text'}
                    onChange={e => updateField('dataType', e.target.value)}
                    onMouseDown={e => e.stopPropagation()}>
                    <option value="text">Text</option>
                    <option value="json">JSON</option>
                    <option value="boolean">Boolean</option>
                    <option value="file">File</option>
                </select>
            </div>
            <OutputPreview nodeId={id} />
            <div className="handle-row output">
                <span className="handle-label">value</span>
                <Handle type="source" position={Position.Right} id="value" className="custom-handle handle-text" title="text" />
            </div>
        </NodeShell>
    );
}
