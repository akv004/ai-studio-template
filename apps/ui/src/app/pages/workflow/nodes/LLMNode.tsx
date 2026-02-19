import { Handle, Position } from '@xyflow/react';
import { Cpu } from 'lucide-react';
import { NodeShell, OutputPreview } from './NodeShell';
import { useNodeData } from '../hooks/useNodeData';
import { PROVIDER_MODELS } from '../nodeColors';

export function LLMNode({ id, data, selected }: { id: string; data: Record<string, unknown>; selected?: boolean }) {
    const { updateField, updateFields } = useNodeData(id);
    const provider = (data.provider as string) || '';
    const model = (data.model as string) || '';
    const temperature = (data.temperature as number) ?? 0.7;
    const sessionMode = (data.sessionMode as string) || 'stateless';
    const maxHistory = (data.maxHistory as number) ?? 20;
    const validModels = PROVIDER_MODELS[provider] || [];

    return (
        <NodeShell id={id} type="llm" label="LLM" icon={Cpu} selected={selected}
            collapsed={data.collapsed as boolean} customLabel={(data.label as string) || ''}>
            <div className="flex flex-col gap-1">
                {/* Inputs */}
                <div className="handle-row input">
                    <Handle type="target" position={Position.Left} id="system" className="custom-handle handle-text" title="text" />
                    <span className="handle-label text-[10px] text-[#666]">system (opt)</span>
                </div>
                <div className="handle-row input">
                    <Handle type="target" position={Position.Left} id="context" className="custom-handle handle-json" title="json" />
                    <span className="handle-label text-[10px] text-[#666]">context (opt)</span>
                </div>
                <div className="handle-row input">
                    <Handle type="target" position={Position.Left} id="prompt" className="custom-handle handle-text" title="text" />
                    <span className="handle-label font-bold">prompt</span>
                </div>

                {/* Inline Controls */}
                <div className="py-1 border-t border-b border-[#333] my-1 flex flex-col gap-1.5" onClick={e => e.stopPropagation()}>
                    <select className="node-inline-input" value={provider}
                        onChange={e => {
                            const p = e.target.value;
                            const models = PROVIDER_MODELS[p] || [];
                            updateFields({ provider: p, model: models[0] || '' });
                        }}
                        onMouseDown={e => e.stopPropagation()}>
                        <option value="" disabled>Provider...</option>
                        <option value="anthropic">Anthropic</option>
                        <option value="google">Google</option>
                        <option value="azure_openai">Azure OpenAI</option>
                        <option value="ollama">Ollama</option>
                        <option value="local">Local</option>
                    </select>
                    <select className="node-inline-input" value={model}
                        onChange={e => updateField('model', e.target.value)}
                        onMouseDown={e => e.stopPropagation()}>
                        <option value="" disabled>Model...</option>
                        {validModels.map(m => <option key={m} value={m}>{m}</option>)}
                    </select>
                    <div className="flex items-center gap-1">
                        <span className="text-[9px] text-[#666] w-8">T={temperature.toFixed(1)}</span>
                        <input type="range" min="0" max="2" step="0.1"
                            className="node-inline-slider flex-1"
                            value={temperature}
                            onChange={e => updateField('temperature', parseFloat(e.target.value))}
                            onMouseDown={e => e.stopPropagation()} />
                    </div>
                    {/* Session Mode */}
                    <div className="flex items-center gap-1">
                        <span className="text-[9px] text-[#666] w-12">Session</span>
                        <select className="node-inline-input flex-1" value={sessionMode}
                            onChange={e => updateField('sessionMode', e.target.value)}
                            onMouseDown={e => e.stopPropagation()}>
                            <option value="stateless">Stateless</option>
                            <option value="session">Session</option>
                        </select>
                    </div>
                    {sessionMode === 'session' && (
                        <div className="flex items-center gap-1">
                            <span className="text-[9px] text-[#666] w-12">History</span>
                            <input type="number" className="node-inline-input flex-1" min={1} max={100}
                                value={maxHistory}
                                onChange={e => updateField('maxHistory', parseInt(e.target.value) || 20)}
                                onMouseDown={e => e.stopPropagation()} />
                        </div>
                    )}
                </div>

                {/* Outputs */}
                <OutputPreview nodeId={id} />
                <div className="handle-row output justify-end">
                    <span className="handle-label font-bold">response</span>
                    <Handle type="source" position={Position.Right} id="response" className="custom-handle handle-text" title="text" />
                </div>
                <div className="handle-row output justify-end">
                    <span className="handle-label text-[10px] text-[#666]">usage</span>
                    <Handle type="source" position={Position.Right} id="usage" className="custom-handle handle-json" title="json" />
                </div>
                <div className="handle-row output justify-end">
                    <span className="handle-label text-[10px] text-[#666]">cost</span>
                    <Handle type="source" position={Position.Right} id="cost" className="custom-handle handle-float" title="float" />
                </div>
            </div>
        </NodeShell>
    );
}
