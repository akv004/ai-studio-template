import { Handle, Position } from '@xyflow/react';
import { BookOpen } from 'lucide-react';
import { NodeShell, OutputPreview } from './NodeShell';
import { useNodeData } from '../hooks/useNodeData';

const EMBEDDING_MODELS: Record<string, string[]> = {
    azure_openai: ['text-embedding-3-small', 'text-embedding-3-large'],
    local: ['nomic-embed-text', 'mxbai-embed-large', 'bge-large-en-v1.5'],
    openai: ['text-embedding-3-small', 'text-embedding-3-large'],
    ollama: ['nomic-embed-text', 'mxbai-embed-large'],
};

export function KnowledgeBaseNode({ id, data, selected }: { id: string; data: Record<string, unknown>; selected?: boolean }) {
    const { updateField, updateFields } = useNodeData(id);
    const embeddingProvider = (data.embeddingProvider as string) || 'azure_openai';
    const embeddingModel = (data.embeddingModel as string) || 'text-embedding-3-small';
    const docsFolder = (data.docsFolder as string) || '';
    const chunkStrategy = (data.chunkStrategy as string) || 'recursive';
    const validModels = EMBEDDING_MODELS[embeddingProvider] || [];

    return (
        <NodeShell id={id} type="knowledge_base" label="KNOWLEDGE BASE" icon={BookOpen} selected={selected}
            collapsed={data.collapsed as boolean} customLabel={(data.label as string) || ''}>
            <div className="flex flex-col gap-1">
                {/* Inputs */}
                <div className="handle-row input">
                    <Handle type="target" position={Position.Left} id="query" className="custom-handle handle-text" title="text" />
                    <span className="handle-label font-bold">query</span>
                </div>
                <div className="handle-row input">
                    <Handle type="target" position={Position.Left} id="folder" className="custom-handle handle-text" title="text" />
                    <span className="handle-label text-[10px] text-[#666]">folder (opt)</span>
                </div>

                {/* Inline Controls */}
                <div className="py-1 border-t border-b border-[#333] my-1 flex flex-col gap-1.5" onClick={e => e.stopPropagation()}>
                    <input className="node-inline-input" value={docsFolder}
                        placeholder="~/my-docs/"
                        onChange={e => updateField('docsFolder', e.target.value)}
                        onMouseDown={e => e.stopPropagation()} />
                    <div className="flex items-center gap-1">
                        <span className="text-[9px] text-[#666] w-16">Embed Provider</span>
                        <select className="node-inline-input flex-1" value={embeddingProvider}
                            onChange={e => {
                                const p = e.target.value;
                                const models = EMBEDDING_MODELS[p] || [];
                                updateFields({ embeddingProvider: p, embeddingModel: models[0] || '' });
                            }}
                            onMouseDown={e => e.stopPropagation()}>
                            <option value="azure_openai">Azure OpenAI</option>
                            <option value="openai">OpenAI</option>
                            <option value="local">Local</option>
                            <option value="ollama">Ollama</option>
                        </select>
                    </div>
                    <div className="flex items-center gap-1">
                        <span className="text-[9px] text-[#666] w-16">Embed Model</span>
                        <select className="node-inline-input flex-1" value={embeddingModel}
                            onChange={e => updateField('embeddingModel', e.target.value)}
                            onMouseDown={e => e.stopPropagation()}>
                            <option value="" disabled>Model...</option>
                            {validModels.map(m => <option key={m} value={m}>{m}</option>)}
                        </select>
                    </div>
                    <div className="flex items-center gap-1">
                        <span className="text-[9px] text-[#666] w-12">Strategy</span>
                        <select className="node-inline-input flex-1" value={chunkStrategy}
                            onChange={e => updateField('chunkStrategy', e.target.value)}
                            onMouseDown={e => e.stopPropagation()}>
                            <option value="recursive">Recursive</option>
                            <option value="paragraph">Paragraph</option>
                            <option value="sentence">Sentence</option>
                            <option value="fixed_size">Fixed Size</option>
                        </select>
                    </div>
                </div>

                {/* Outputs */}
                <OutputPreview nodeId={id} />
                <div className="handle-row output justify-end">
                    <span className="handle-label font-bold">context</span>
                    <Handle type="source" position={Position.Right} id="context" className="custom-handle handle-text" title="text" />
                </div>
                <div className="handle-row output justify-end">
                    <span className="handle-label text-[10px] text-[#666]">results</span>
                    <Handle type="source" position={Position.Right} id="results" className="custom-handle handle-json" title="json" />
                </div>
                <div className="handle-row output justify-end">
                    <span className="handle-label text-[10px] text-[#666]">indexStats</span>
                    <Handle type="source" position={Position.Right} id="indexStats" className="custom-handle handle-json" title="json" />
                </div>
            </div>
        </NodeShell>
    );
}
