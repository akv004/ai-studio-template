import { Handle, Position } from '@xyflow/react';
import { FolderOpen } from 'lucide-react';
import { NodeShell, OutputPreview } from './NodeShell';
import { useNodeData } from '../hooks/useNodeData';

export function FileGlobNode({ id, data, selected }: { id: string; data: Record<string, unknown>; selected?: boolean }) {
    const { updateField } = useNodeData(id);

    return (
        <NodeShell id={id} type="file_glob" label="FILE GLOB" icon={FolderOpen} selected={selected}
            collapsed={data.collapsed as boolean} customLabel={(data.label as string) || ''}>
            <div className="handle-row input">
                <Handle type="target" position={Position.Left} id="directory"
                    className="custom-handle handle-text" title="text" />
                <span className="handle-label">directory</span>
            </div>

            <div onClick={e => e.stopPropagation()}>
                <input className="node-inline-input" value={(data.directory as string) || ''}
                    placeholder="/path/to/directory" onChange={e => updateField('directory', e.target.value)}
                    onMouseDown={e => e.stopPropagation()} />
                <input className="node-inline-input" value={(data.pattern as string) || '*'}
                    placeholder="*.csv" onChange={e => updateField('pattern', e.target.value)}
                    onMouseDown={e => e.stopPropagation()} />
                <div className="flex gap-1">
                    <select className="node-inline-input flex-1" value={(data.mode as string) || 'text'}
                        onChange={e => updateField('mode', e.target.value)}
                        onMouseDown={e => e.stopPropagation()}>
                        <option value="text">Text</option>
                        <option value="json">JSON</option>
                        <option value="csv">CSV</option>
                        <option value="binary">Binary</option>
                    </select>
                    <label className="flex items-center gap-1 text-[10px] text-[var(--text-muted)] whitespace-nowrap"
                        onMouseDown={e => e.stopPropagation()}>
                        <input type="checkbox" checked={(data.recursive as boolean) ?? false}
                            onChange={e => updateField('recursive', e.target.checked)} />
                        recursive
                    </label>
                </div>
            </div>

            <OutputPreview nodeId={id} />

            <div className="handle-row output">
                <span className="handle-label">files</span>
                <Handle type="source" position={Position.Right} id="files"
                    className="custom-handle handle-json" title="json" />
            </div>
            <div className="handle-row output">
                <span className="handle-label">count</span>
                <Handle type="source" position={Position.Right} id="count"
                    className="custom-handle handle-float" title="number" />
            </div>
            <div className="handle-row output">
                <span className="handle-label">paths</span>
                <Handle type="source" position={Position.Right} id="paths"
                    className="custom-handle handle-json" title="json" />
            </div>
        </NodeShell>
    );
}
