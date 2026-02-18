import { Handle, Position } from '@xyflow/react';
import { Cpu } from 'lucide-react';
import { NodeShell, OutputPreview } from './NodeShell';

export function LLMNode({ id, data, selected }: { id: string; data: Record<string, unknown>; selected?: boolean }) {
    return (
        <NodeShell id={id} type="llm" label="LLM" icon={Cpu} selected={selected}
            collapsed={data.collapsed as boolean}>
            <div className="flex flex-col gap-1">
                {/* Inputs */}
                <div className="handle-row input">
                    <Handle type="target" position={Position.Left} id="system" className="custom-handle handle-text" />
                    <span className="handle-label text-[10px] text-[#666]">system (opt)</span>
                </div>
                <div className="handle-row input">
                    <Handle type="target" position={Position.Left} id="context" className="custom-handle handle-json" />
                    <span className="handle-label text-[10px] text-[#666]">context (opt)</span>
                </div>
                <div className="handle-row input">
                    <Handle type="target" position={Position.Left} id="prompt" className="custom-handle handle-text" />
                    <span className="handle-label font-bold">prompt</span>
                </div>

                {/* Body Content */}
                <div className="py-2 border-t border-b border-[#333] my-1 flex flex-col gap-1">
                    <div className="text-[11px] font-medium text-[var(--accent-secondary)]">{(data.model as string) || 'Select model'}</div>
                    <div className="text-[10px] text-[#888]">{(data.provider as string) || 'No provider'}</div>
                    {Boolean(data.systemPrompt) && (
                        <div className="text-[10px] text-[#666] mt-0.5 truncate max-w-[160px] italic">
                            System: {(data.systemPrompt as string).slice(0, 30)}...
                        </div>
                    )}
                </div>

                {/* Outputs */}
                <OutputPreview nodeId={id} />
                <div className="handle-row output justify-end">
                    <span className="handle-label font-bold">response</span>
                    <Handle type="source" position={Position.Right} id="response" className="custom-handle handle-text" />
                </div>
                <div className="handle-row output justify-end">
                    <span className="handle-label text-[10px] text-[#666]">usage</span>
                    <Handle type="source" position={Position.Right} id="usage" className="custom-handle handle-json" />
                </div>
                <div className="handle-row output justify-end">
                    <span className="handle-label text-[10px] text-[#666]">cost</span>
                    <Handle type="source" position={Position.Right} id="cost" className="custom-handle handle-float" />
                </div>
            </div>
        </NodeShell>
    );
}
