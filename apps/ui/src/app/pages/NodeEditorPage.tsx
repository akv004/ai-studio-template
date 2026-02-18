import { useState, useEffect, useCallback, useRef, useMemo } from 'react';
import {
    ReactFlow,
    Background,
    Controls,
    MiniMap,
    Panel,
    useNodesState,
    useEdgesState,
    addEdge,
    type Node,
    type Edge,
    type Connection,
    type NodeTypes,
    type OnConnect,
    Handle,
    Position,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import {
    Plus, Save, Play, Trash2, Copy, ChevronLeft, ChevronDown, ChevronRight,
    Loader2, RefreshCw, MessageSquare, Wrench, GitFork,
    ShieldCheck, Repeat, FileInput, FileOutput, Cpu,
    Check, X, Clock,
} from 'lucide-react';
import { useAppStore } from '../../state/store';
import type { Workflow, CreateWorkflowRequest, NodeExecutionStatus } from '@ai-studio/shared';

// ============================================
// NODE TYPE DEFINITIONS
// ============================================

interface NodeCategory {
    label: string;
    types: { type: string; label: string; icon: React.ElementType; description: string }[];
}

const NODE_CATEGORIES: NodeCategory[] = [
    {
        label: 'Inputs / Outputs',
        types: [
            { type: 'input', label: 'Input', icon: FileInput, description: 'Workflow entry point' },
            { type: 'output', label: 'Output', icon: FileOutput, description: 'Workflow exit point' },
        ],
    },
    {
        label: 'AI',
        types: [
            { type: 'llm', label: 'LLM', icon: Cpu, description: 'Language model call' },
            { type: 'router', label: 'Router', icon: GitFork, description: 'Conditional branching' },
        ],
    },
    {
        label: 'Tools',
        types: [
            { type: 'tool', label: 'Tool', icon: Wrench, description: 'MCP or built-in tool' },
        ],
    },
    {
        label: 'Logic',
        types: [
            { type: 'approval', label: 'Approval', icon: ShieldCheck, description: 'Human approval gate' },
            { type: 'transform', label: 'Transform', icon: Repeat, description: 'Data transformation' },
        ],
    },
    {
        label: 'Composition',
        types: [
            { type: 'subworkflow', label: 'Subworkflow', icon: MessageSquare, description: 'Embed another workflow' },
        ],
    },
];

// ============================================
// CUSTOM NODE COMPONENTS
// ============================================

// Execution state badge labels
const execBadgeConfig: Record<NodeExecutionStatus, { icon: React.ElementType | null; label: string }> = {
    idle: { icon: null, label: '' },
    running: { icon: Loader2, label: 'Running' },
    completed: { icon: Check, label: 'Done' },
    error: { icon: X, label: 'Error' },
    waiting: { icon: Clock, label: 'Waiting' },
    skipped: { icon: null, label: 'Skipped' },
};

function ExecutionBadge({ nodeId }: { nodeId: string }) {
    const state = useAppStore((s) => s.workflowNodeStates[nodeId]);
    if (!state || state.status === 'idle') return null;
    const cfg = execBadgeConfig[state.status];
    const Icon = cfg.icon;
    return (
        <div className="absolute -top-2 -right-2 flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] font-medium bg-[#1e1e1e] border border-[#3a3a3a] z-10">
            {Icon && <Icon size={10} className={state.status === 'running' ? 'animate-spin' : ''} />}
            {cfg.label}
        </div>
    );
}

function OutputPreview({ nodeId }: { nodeId: string }) {
    const state = useAppStore((s) => s.workflowNodeStates[nodeId]);
    if (!state || state.status !== 'completed' || !state.output) return null;
    return (
        <div className="mt-1 text-[10px] text-[#888] truncate max-w-[160px] font-mono">
            {state.output.slice(0, 80)}{state.output.length > 80 ? '...' : ''}
        </div>
    );
}

function useExecClass(nodeId: string): string {
    const state = useAppStore((s) => s.workflowNodeStates[nodeId]);
    if (!state || state.status === 'idle') return '';
    return `exec-${state.status}`;
}

// Muted Blender-inspired palette
const nodeColors: Record<string, string> = {
    input: '#2d5a27',
    output: '#8a5a1e',
    llm: '#3a3a8a',
    tool: '#8a2a5a',
    router: '#1a6a6a',
    approval: '#8a7a1a',
    transform: '#5a3a8a',
    subworkflow: '#1a5a7a',
};

// Shared node wrapper with collapse support
function NodeShell({ id, type, label, icon: Icon, selected, collapsed, onToggleCollapse, children }: {
    id: string; type: string; label: string; icon: React.ElementType;
    selected?: boolean; collapsed?: boolean; onToggleCollapse?: () => void;
    children: React.ReactNode;
}) {
    const execClass = useExecClass(id);
    return (
        <div className={`custom-node ${selected ? 'selected' : ''} ${collapsed ? 'collapsed' : ''} ${execClass} relative`}>
            <ExecutionBadge nodeId={id} />
            <div className="custom-node-header" style={{ background: nodeColors[type] }}
                onClick={onToggleCollapse}>
                <span className="collapse-chevron">
                    {collapsed ? <ChevronRight size={8} /> : <ChevronDown size={8} />}
                </span>
                <Icon size={12} />
                {label}
            </div>
            <div className="custom-node-body">
                {children}
            </div>
        </div>
    );
}

function InputNode({ id, data, selected }: { id: string; data: Record<string, unknown>; selected?: boolean }) {
    return (
        <NodeShell id={id} type="input" label="INPUT" icon={FileInput} selected={selected}
            collapsed={data.collapsed as boolean}>
            <div className="text-[11px] font-medium">{(data.name as string) || 'untitled'}</div>
            <div className="text-[10px] text-[#888]">Type: {(data.dataType as string) || 'text'}</div>
            <OutputPreview nodeId={id} />
            <div className="handle-row output">
                <span className="handle-label">value</span>
                <Handle type="source" position={Position.Right} className="custom-handle handle-text" />
            </div>
        </NodeShell>
    );
}

function OutputNode({ id, data, selected }: { id: string; data: Record<string, unknown>; selected?: boolean }) {
    return (
        <NodeShell id={id} type="output" label="OUTPUT" icon={FileOutput} selected={selected}
            collapsed={data.collapsed as boolean}>
            <div className="handle-row input">
                <Handle type="target" position={Position.Left} className="custom-handle handle-text" />
                <span className="handle-label">value</span>
            </div>
            <div className="text-[11px] font-medium">{(data.name as string) || 'result'}</div>
            <div className="text-[10px] text-[#888]">Format: {(data.format as string) || 'text'}</div>
            <OutputPreview nodeId={id} />
        </NodeShell>
    );
}

function LLMNode({ id, data, selected }: { id: string; data: Record<string, unknown>; selected?: boolean }) {
    return (
        <NodeShell id={id} type="llm" label="LLM" icon={Cpu} selected={selected}
            collapsed={data.collapsed as boolean}>
            <div className="handle-row input">
                <Handle type="target" position={Position.Left} id="prompt" className="custom-handle handle-text" />
                <span className="handle-label">prompt</span>
            </div>
            <div className="text-[11px] font-medium">{(data.model as string) || 'Select model'}</div>
            <div className="text-[10px] text-[#888]">{(data.provider as string) || 'No provider'}</div>
            {Boolean(data.systemPrompt) && (
                <div className="text-[10px] text-[#666] mt-0.5 truncate max-w-[160px]">
                    {(data.systemPrompt as string).slice(0, 40)}...
                </div>
            )}
            <OutputPreview nodeId={id} />
            <div className="handle-row output">
                <span className="handle-label">response</span>
                <Handle type="source" position={Position.Right} id="response" className="custom-handle handle-text" />
            </div>
        </NodeShell>
    );
}

function ToolNode({ id, data, selected }: { id: string; data: Record<string, unknown>; selected?: boolean }) {
    return (
        <NodeShell id={id} type="tool" label="TOOL" icon={Wrench} selected={selected}
            collapsed={data.collapsed as boolean}>
            <div className="handle-row input">
                <Handle type="target" position={Position.Left} className="custom-handle handle-json" />
                <span className="handle-label">input</span>
            </div>
            <div className="text-[11px] font-medium">{(data.toolName as string) || 'Select tool'}</div>
            {Boolean(data.serverName) && (
                <div className="text-[10px] text-[#888]">Server: {data.serverName as string}</div>
            )}
            <OutputPreview nodeId={id} />
            <div className="handle-row output">
                <span className="handle-label">result</span>
                <Handle type="source" position={Position.Right} id="result" className="custom-handle handle-json" />
            </div>
        </NodeShell>
    );
}

function RouterNode({ id, data, selected }: { id: string; data: Record<string, unknown>; selected?: boolean }) {
    const branches = (data.branches as string[]) || ['true', 'false'];
    return (
        <NodeShell id={id} type="router" label="ROUTER" icon={GitFork} selected={selected}
            collapsed={data.collapsed as boolean}>
            <div className="handle-row input">
                <Handle type="target" position={Position.Left} className="custom-handle handle-text" />
                <span className="handle-label">input</span>
            </div>
            <div className="text-[10px] text-[#888]">Mode: {(data.mode as string) || 'llm'}</div>
            {branches.map((b, i) => (
                <div key={i} className="handle-row output">
                    <span className="handle-label">{b}</span>
                    <Handle type="source" position={Position.Right} id={`branch-${i}`}
                        className="custom-handle handle-bool" />
                </div>
            ))}
        </NodeShell>
    );
}

function ApprovalNode({ id, data, selected }: { id: string; data: Record<string, unknown>; selected?: boolean }) {
    return (
        <NodeShell id={id} type="approval" label="APPROVAL" icon={ShieldCheck} selected={selected}
            collapsed={data.collapsed as boolean}>
            <div className="handle-row input">
                <Handle type="target" position={Position.Left} className="custom-handle handle-any" />
                <span className="handle-label">data</span>
            </div>
            <div className="text-[11px]">{((data.message as string) || 'Approve?').slice(0, 40)}</div>
            <div className="handle-row output">
                <span className="handle-label">approved</span>
                <Handle type="source" position={Position.Right} id="approved" className="custom-handle handle-bool" />
            </div>
            <div className="handle-row output">
                <span className="handle-label">rejected</span>
                <Handle type="source" position={Position.Right} id="rejected" className="custom-handle handle-bool" />
            </div>
        </NodeShell>
    );
}

function TransformNode({ id, data, selected }: { id: string; data: Record<string, unknown>; selected?: boolean }) {
    return (
        <NodeShell id={id} type="transform" label="TRANSFORM" icon={Repeat} selected={selected}
            collapsed={data.collapsed as boolean}>
            <div className="handle-row input">
                <Handle type="target" position={Position.Left} className="custom-handle handle-any" />
                <span className="handle-label">input</span>
            </div>
            <div className="text-[10px] text-[#888]">Mode: {(data.mode as string) || 'template'}</div>
            {Boolean(data.template) && (
                <div className="text-[10px] mt-0.5 truncate max-w-[160px] font-mono text-[#777]">
                    {(data.template as string).slice(0, 30)}
                </div>
            )}
            <OutputPreview nodeId={id} />
            <div className="handle-row output">
                <span className="handle-label">output</span>
                <Handle type="source" position={Position.Right} className="custom-handle handle-any" />
            </div>
        </NodeShell>
    );
}

function SubworkflowNode({ id, data, selected }: { id: string; data: Record<string, unknown>; selected?: boolean }) {
    return (
        <NodeShell id={id} type="subworkflow" label="SUBWORKFLOW" icon={MessageSquare} selected={selected}
            collapsed={data.collapsed as boolean}>
            <div className="handle-row input">
                <Handle type="target" position={Position.Left} className="custom-handle handle-any" />
                <span className="handle-label">input</span>
            </div>
            <div className="text-[11px] font-medium">{(data.workflowName as string) || 'Select workflow'}</div>
            <div className="handle-row output">
                <span className="handle-label">output</span>
                <Handle type="source" position={Position.Right} className="custom-handle handle-any" />
            </div>
        </NodeShell>
    );
}

const customNodeTypes: NodeTypes = {
    input: InputNode,
    output: OutputNode,
    llm: LLMNode,
    tool: ToolNode,
    router: RouterNode,
    approval: ApprovalNode,
    transform: TransformNode,
    subworkflow: SubworkflowNode,
};

// ============================================
// DEFAULT NODE DATA
// ============================================

function defaultDataForType(type: string): Record<string, unknown> {
    switch (type) {
        case 'input': return { name: 'input', dataType: 'text', default: '' };
        case 'output': return { name: 'result', format: 'text' };
        case 'llm': return { provider: '', model: '', systemPrompt: '', temperature: 0.7, maxTokens: 4096 };
        case 'tool': return { toolName: '', serverName: '', approval: 'auto' };
        case 'router': return { mode: 'pattern', branches: ['true', 'false'] };
        case 'approval': return { message: 'Review before continuing', showData: true, timeout: null };
        case 'transform': return { mode: 'template', template: '{{input}}' };
        case 'subworkflow': return { workflowId: '', workflowName: '' };
        default: return {};
    }
}

// ============================================
// NODE CONFIG PANEL
// ============================================

const PROVIDER_MODELS: Record<string, string[]> = {
    anthropic: ['claude-sonnet-4-5-20250929', 'claude-haiku-4-5-20251001', 'claude-opus-4-6'],
    google: ['gemini-2.0-flash', 'gemini-2.5-pro', 'gemini-2.5-flash'],
    azure_openai: ['gpt-4o', 'gpt-4o-mini', 'gpt-4.1'],
    ollama: ['llama3.2', 'llama3.1', 'mistral', 'codellama', 'qwen2.5'],
};

function NodeConfigPanel({ node, onChange, onDelete }: {
    node: Node;
    onChange: (data: Record<string, unknown>) => void;
    onDelete: () => void;
}) {
    const nodeState = useAppStore((s) => s.workflowNodeStates[node.id]);
    const data = node.data as Record<string, unknown>;
    const type = node.type || 'input';

    const update = (field: string, value: unknown) => {
        onChange({ ...data, [field]: value });
    };

    // Auto-correct model if it doesn't belong to the selected provider
    const provider = (data.provider as string) || '';
    const currentModel = (data.model as string) || '';
    const validModels = PROVIDER_MODELS[provider] || [];
    if (type === 'llm' && provider && currentModel && validModels.length > 0 && !validModels.includes(currentModel)) {
        onChange({ ...data, model: validModels[0] });
    }

    return (
        <div className="p-4 space-y-3">
            <h3 className="text-sm font-semibold uppercase text-[var(--text-muted)]">
                {type} Configuration
            </h3>

            {/* Common: name field for input/output */}
            {(type === 'input' || type === 'output') && (
                <label className="block">
                    <span className="text-xs text-[var(--text-muted)]">Name</span>
                    <input
                        className="config-input"
                        value={(data.name as string) || ''}
                        onChange={(e) => update('name', e.target.value)}
                    />
                </label>
            )}

            {type === 'input' && (
                <label className="block">
                    <span className="text-xs text-[var(--text-muted)]">Data Type</span>
                    <select className="config-input" value={(data.dataType as string) || 'text'}
                        onChange={(e) => update('dataType', e.target.value)}>
                        <option value="text">Text</option>
                        <option value="json">JSON</option>
                        <option value="boolean">Boolean</option>
                        <option value="file">File</option>
                    </select>
                </label>
            )}

            {type === 'output' && (
                <label className="block">
                    <span className="text-xs text-[var(--text-muted)]">Format</span>
                    <select className="config-input" value={(data.format as string) || 'text'}
                        onChange={(e) => update('format', e.target.value)}>
                        <option value="text">Text</option>
                        <option value="markdown">Markdown</option>
                        <option value="json">JSON</option>
                    </select>
                </label>
            )}

            {type === 'llm' && (
                <>
                    <label className="block">
                        <span className="text-xs text-[var(--text-muted)]">Provider</span>
                        <select className="config-input" value={(data.provider as string) || ''}
                            onChange={(e) => {
                                const p = e.target.value;
                                const models = PROVIDER_MODELS[p] || [];
                                onChange({ ...data, provider: p, model: models[0] || '' });
                            }}>
                            <option value="" disabled>Select provider...</option>
                            <option value="anthropic">Anthropic</option>
                            <option value="google">Google</option>
                            <option value="azure_openai">Azure OpenAI</option>
                            <option value="ollama">Ollama (local)</option>
                        </select>
                    </label>
                    <label className="block">
                        <span className="text-xs text-[var(--text-muted)]">Model</span>
                        <select className="config-input" value={(data.model as string) || ''}
                            onChange={(e) => update('model', e.target.value)}>
                            <option value="" disabled>Select model...</option>
                            {validModels.map((m) => (
                                <option key={m} value={m}>{m}</option>
                            ))}
                        </select>
                    </label>
                    <label className="block">
                        <span className="text-xs text-[var(--text-muted)]">System Prompt</span>
                        <textarea className="config-input min-h-[80px]" value={(data.systemPrompt as string) || ''}
                            onChange={(e) => update('systemPrompt', e.target.value)} />
                    </label>
                    <label className="block">
                        <span className="text-xs text-[var(--text-muted)]">Temperature</span>
                        <input type="number" step="0.1" min="0" max="2" className="config-input"
                            value={(data.temperature as number) ?? 0.7}
                            onChange={(e) => update('temperature', parseFloat(e.target.value))} />
                    </label>
                </>
            )}

            {type === 'tool' && (
                <>
                    <label className="block">
                        <span className="text-xs text-[var(--text-muted)]">Tool Name</span>
                        <input className="config-input" value={(data.toolName as string) || ''}
                            onChange={(e) => update('toolName', e.target.value)}
                            placeholder="e.g. builtin__shell" />
                    </label>
                    <label className="block">
                        <span className="text-xs text-[var(--text-muted)]">Approval</span>
                        <select className="config-input" value={(data.approval as string) || 'auto'}
                            onChange={(e) => update('approval', e.target.value)}>
                            <option value="auto">Auto</option>
                            <option value="ask">Ask</option>
                            <option value="deny">Deny</option>
                        </select>
                    </label>
                </>
            )}

            {type === 'router' && (
                <>
                    <label className="block">
                        <span className="text-xs text-[var(--text-muted)]">Mode</span>
                        <select className="config-input" value={(data.mode as string) || 'pattern'}
                            onChange={(e) => update('mode', e.target.value)}>
                            <option value="pattern">Pattern Match</option>
                            <option value="llm">LLM Classify</option>
                        </select>
                    </label>
                    <label className="block">
                        <span className="text-xs text-[var(--text-muted)]">Branches (comma-separated)</span>
                        <input className="config-input"
                            value={((data.branches as string[]) || ['true', 'false']).join(', ')}
                            onChange={(e) => update('branches', e.target.value.split(',').map(s => s.trim()).filter(Boolean))} />
                    </label>
                </>
            )}

            {type === 'approval' && (
                <label className="block">
                    <span className="text-xs text-[var(--text-muted)]">Message</span>
                    <textarea className="config-input min-h-[60px]" value={(data.message as string) || ''}
                        onChange={(e) => update('message', e.target.value)} />
                </label>
            )}

            {type === 'transform' && (
                <>
                    <label className="block">
                        <span className="text-xs text-[var(--text-muted)]">Mode</span>
                        <select className="config-input" value={(data.mode as string) || 'template'}
                            onChange={(e) => update('mode', e.target.value)}>
                            <option value="template">Template</option>
                            <option value="jsonpath">JSONPath</option>
                            <option value="script">Script</option>
                        </select>
                    </label>
                    <label className="block">
                        <span className="text-xs text-[var(--text-muted)]">Template / Expression</span>
                        <textarea className="config-input min-h-[60px] font-mono text-xs"
                            value={(data.template as string) || ''}
                            onChange={(e) => update('template', e.target.value)} />
                    </label>
                </>
            )}

            {/* Last run output */}
            {nodeState && nodeState.status === 'completed' && nodeState.output && (
                <div className="pt-2 border-t border-[var(--border-subtle)]">
                    <div className="flex items-center gap-1 mb-1">
                        <Check size={12} className="text-green-400" />
                        <span className="text-xs font-medium text-green-400">Output</span>
                        {nodeState.durationMs != null && (
                            <span className="text-[10px] text-[var(--text-muted)] ml-auto">
                                {(nodeState.durationMs / 1000).toFixed(1)}s
                            </span>
                        )}
                    </div>
                    <pre className="text-xs text-[var(--text-secondary)] bg-[var(--bg-primary)] p-2 rounded max-h-[200px] overflow-y-auto whitespace-pre-wrap font-mono">
                        {nodeState.output}
                    </pre>
                </div>
            )}
            {nodeState && nodeState.status === 'error' && nodeState.error && (
                <div className="pt-2 border-t border-[var(--border-subtle)]">
                    <div className="flex items-center gap-1 mb-1">
                        <X size={12} className="text-red-400" />
                        <span className="text-xs font-medium text-red-400">Error</span>
                    </div>
                    <pre className="text-xs text-red-300 bg-red-500/10 p-2 rounded max-h-[120px] overflow-y-auto whitespace-pre-wrap font-mono">
                        {nodeState.error}
                    </pre>
                </div>
            )}

            <div className="pt-2 flex items-center justify-between">
                <span className="text-xs text-[var(--text-muted)]">ID: {node.id}</span>
                <button className="btn-icon text-red-400 hover:text-red-300" title="Delete node" onClick={onDelete}>
                    <Trash2 size={14} />
                </button>
            </div>
        </div>
    );
}

// ============================================
// WORKFLOW LIST VIEW
// ============================================

function WorkflowList({ onSelect, onCreate }: {
    onSelect: (id: string) => void;
    onCreate: () => void;
}) {
    const { workflows, workflowsLoading, fetchWorkflows, deleteWorkflow, duplicateWorkflow } = useAppStore();

    useEffect(() => {
        fetchWorkflows();
    }, [fetchWorkflows]);

    return (
        <div className="page-content">
            <div className="page-header">
                <h1 className="page-title">Node Editor</h1>
                <div className="flex gap-2">
                    <button className="btn-secondary" onClick={() => fetchWorkflows()}>
                        <RefreshCw size={16} />
                    </button>
                    <button className="btn-primary" onClick={onCreate}>
                        <Plus size={16} /> New Workflow
                    </button>
                </div>
            </div>

            {workflowsLoading ? (
                <div className="flex items-center justify-center py-20">
                    <Loader2 className="animate-spin" size={24} />
                </div>
            ) : workflows.length === 0 ? (
                <div className="empty-state">
                    <GitFork size={48} className="text-[var(--text-muted)]" />
                    <h2>No workflows yet</h2>
                    <p>Create your first workflow to build AI pipelines visually.</p>
                    <button className="btn-primary mt-4" onClick={onCreate}>
                        <Plus size={16} /> New Workflow
                    </button>
                </div>
            ) : (
                <div className="grid gap-3">
                    {workflows.map((w) => (
                        <div key={w.id}
                            className="card cursor-pointer hover:border-[var(--border-accent)]"
                            onClick={() => onSelect(w.id)}>
                            <div className="flex items-center justify-between">
                                <div>
                                    <div className="font-medium">{w.name}</div>
                                    {w.description && (
                                        <div className="text-sm text-[var(--text-muted)] mt-0.5">{w.description}</div>
                                    )}
                                    <div className="text-xs text-[var(--text-muted)] mt-1">
                                        {w.nodeCount} nodes
                                    </div>
                                </div>
                                <div className="flex gap-1" onClick={(e) => e.stopPropagation()}>
                                    <button className="btn-icon" title="Duplicate"
                                        onClick={() => duplicateWorkflow(w.id)}>
                                        <Copy size={14} />
                                    </button>
                                    <button className="btn-icon text-red-400" title="Delete"
                                        onClick={() => deleteWorkflow(w.id)}>
                                        <Trash2 size={14} />
                                    </button>
                                </div>
                            </div>
                        </div>
                    ))}
                </div>
            )}
        </div>
    );
}

// ============================================
// NODE EDITOR CANVAS
// ============================================

let nodeIdCounter = 0;
function generateNodeId(type: string): string {
    return `${type}_${++nodeIdCounter}_${Date.now().toString(36)}`;
}

function formatRuntimeError(error: unknown): string {
    if (typeof error === 'string') return error;
    if (error instanceof Error) return error.message;
    if (error && typeof error === 'object') {
        const maybeError = error as { message?: unknown; error?: unknown; detail?: unknown };
        if (typeof maybeError.message === 'string') return maybeError.message;
        if (typeof maybeError.error === 'string') return maybeError.error;
        if (typeof maybeError.detail === 'string') return maybeError.detail;
        try {
            return JSON.stringify(error);
        } catch {
            // fall through
        }
    }
    return String(error);
}

interface LastRunDebugInfo {
    workflowId: string;
    sessionId: string | null;
    status: string;
    error: string;
    timestamp: string;
}

interface LastRunResult {
    sessionId: string;
    tokens: number;
    costUsd: number;
    durationMs: number;
    nodeCount: number;
    outputs: Record<string, unknown>;
}

function WorkflowCanvas({ workflow, onBack }: {
    workflow: Workflow;
    onBack: () => void;
}) {
    const {
        updateWorkflow,
        addToast,
        runWorkflow,
        setNodeState,
        resetNodeStates,
        workflowRunning,
        workflowNodeStates,
        openInspector,
    } = useAppStore();

    // Parse graph from workflow
    const initialGraph = useMemo(() => {
        try {
            const parsed = JSON.parse(workflow.graphJson);
            return {
                nodes: (parsed.nodes || []) as Node[],
                edges: (parsed.edges || []) as Edge[],
                viewport: parsed.viewport || { x: 0, y: 0, zoom: 1 },
            };
        } catch {
            return { nodes: [], edges: [], viewport: { x: 0, y: 0, zoom: 1 } };
        }
    }, [workflow.graphJson]);

    const [nodes, setNodes, onNodesChange] = useNodesState(initialGraph.nodes);
    const [edges, setEdges, onEdgesChange] = useEdgesState(initialGraph.edges);
    const [selectedNode, setSelectedNode] = useState<Node | null>(null);
    const [saving, setSaving] = useState(false);
    const [hasChanges, setHasChanges] = useState(false);
    const [showRunModal, setShowRunModal] = useState(false);
    const [runInputs, setRunInputs] = useState<Record<string, unknown>>({});
    const [approvalRequest, setApprovalRequest] = useState<{ id: string; message: string; dataPreview?: string } | null>(null);
    const [lastRunDebug, setLastRunDebug] = useState<LastRunDebugInfo | null>(null);
    const [lastRunResult, setLastRunResult] = useState<LastRunResult | null>(null);
    const [contextMenu, setContextMenu] = useState<{ x: number; y: number; nodeId?: string } | null>(null);
    const clipboardRef = useRef<{ nodes: Node[]; edges: Edge[] } | null>(null);
    const reactFlowRef = useRef<HTMLDivElement>(null);

    // Track changes
    useEffect(() => {
        setHasChanges(true);
    }, [nodes, edges]);

    // Listen for workflow node events to update execution state visuals
    useEffect(() => {
        let unlistenEvents: (() => void) | undefined;
        let unlistenApproval: (() => void) | undefined;

        (async () => {
            try {
                const { listen } = await import('@tauri-apps/api/event');

                unlistenEvents = await listen<{
                    type: string;
                    payload: Record<string, unknown>;
                }>('agent_event', (tauriEvent) => {
                    const { type, payload } = tauriEvent.payload;
                    if (!type?.startsWith('workflow.node.')) return;

                    const nodeId = payload.node_id as string;
                    if (!nodeId) return;

                    if (type === 'workflow.node.started') {
                        setNodeState(nodeId, 'running');
                    } else if (type === 'workflow.node.completed') {
                        setNodeState(nodeId, 'completed', {
                            output: (payload.output_preview || payload.output) as string | undefined,
                            durationMs: payload.duration_ms as number | undefined,
                            tokens: payload.tokens as number | undefined,
                            costUsd: payload.cost_usd as number | undefined,
                        });
                    } else if (type === 'workflow.node.error') {
                        setNodeState(nodeId, 'error', {
                            error: payload.error as string | undefined,
                        });
                    } else if (type === 'workflow.node.waiting') {
                        setNodeState(nodeId, 'waiting');
                    } else if (type === 'workflow.node.skipped') {
                        setNodeState(nodeId, 'skipped');
                    }
                });

                // Listen for approval requests from workflow execution
                unlistenApproval = await listen<{
                    id: string;
                    message: string;
                    dataPreview?: string;
                }>('workflow_approval_requested', (event) => {
                    setApprovalRequest(event.payload);
                });
            } catch {
                // Not running under Tauri
            }
        })();

        return () => {
            unlistenEvents?.();
            unlistenApproval?.();
        };
    }, [setNodeState]);

    // Handle new connections
    const onConnect: OnConnect = useCallback(
        (connection: Connection) => {
            setEdges((eds) => addEdge(connection, eds));
        },
        [setEdges],
    );

    // Handle node selection
    const onNodeClick = useCallback((_: React.MouseEvent, node: Node) => {
        setSelectedNode(node);
    }, []);

    const onPaneClick = useCallback(() => {
        setSelectedNode(null);
        setContextMenu(null);
    }, []);

    // Drag and drop from palette
    const onDragOver = useCallback((event: React.DragEvent) => {
        event.preventDefault();
        event.dataTransfer.dropEffect = 'move';
    }, []);

    const onDrop = useCallback(
        (event: React.DragEvent) => {
            event.preventDefault();
            const type = event.dataTransfer.getData('application/reactflow');
            if (!type) return;

            const bounds = reactFlowRef.current?.getBoundingClientRect();
            if (!bounds) return;

            const position = {
                x: event.clientX - bounds.left - 80,
                y: event.clientY - bounds.top - 20,
            };

            const newNode: Node = {
                id: generateNodeId(type),
                type,
                position,
                data: defaultDataForType(type),
            };

            setNodes((nds) => [...nds, newNode]);
        },
        [setNodes],
    );

    // Save workflow
    const handleSave = useCallback(async () => {
        setSaving(true);
        try {
            const graphJson = JSON.stringify({
                nodes,
                edges,
                viewport: { x: 0, y: 0, zoom: 1 },
            });
            await updateWorkflow(workflow.id, { graphJson });
            setHasChanges(false);
            addToast('Workflow saved', 'success');
        } catch {
            // Error handled by store
        } finally {
            setSaving(false);
        }
    }, [nodes, edges, workflow.id, updateWorkflow, addToast]);

    // Handle approval decision
    const handleApprovalDecision = useCallback(async (approve: boolean) => {
        if (!approvalRequest) return;
        try {
            const { invoke } = await import('@tauri-apps/api/core');
            await invoke('approve_tool_request', { id: approvalRequest.id, approve });
        } catch {
            addToast('Failed to send approval decision', 'error');
        }
        setApprovalRequest(null);
    }, [approvalRequest, addToast]);

    // Handle run workflow
    const handleRunClick = useCallback(() => {
        // Build default inputs from Input nodes — use logical name only
        const defaults: Record<string, unknown> = {};
        nodes.forEach((n) => {
            if (n.type === 'input') {
                const name = (n.data.name as string) || 'input';
                const defaultVal = n.data.default ?? '';
                defaults[name] = defaultVal;
            }
        });
        setRunInputs(defaults);
        setShowRunModal(true);
    }, [nodes]);

    const handleRunSubmit = useCallback(async () => {
        setShowRunModal(false);
        resetNodeStates();
        try {
            // Auto-save current graph before running so execution uses latest state
            const graphJson = JSON.stringify({
                nodes,
                edges,
                viewport: { x: 0, y: 0, zoom: 1 },
            });
            await updateWorkflow(workflow.id, { graphJson });
            setHasChanges(false);

            const result = await runWorkflow(workflow.id, runInputs);
            if (result.status === 'completed') {
                setLastRunDebug(null);
                setLastRunResult({
                    sessionId: result.sessionId,
                    tokens: result.totalTokens,
                    costUsd: result.totalCostUsd,
                    durationMs: result.durationMs,
                    nodeCount: result.nodeCount,
                    outputs: result.outputs,
                });
                addToast(`Workflow completed in ${(result.durationMs / 1000).toFixed(1)}s (${result.totalTokens} tokens)`, 'success');
                return;
            }
            setLastRunResult(null);
            setLastRunDebug({
                workflowId: workflow.id,
                sessionId: result.sessionId || null,
                status: result.status,
                error: result.error || 'Workflow failed with unknown error',
                timestamp: new Date().toISOString(),
            });
        } catch (e) {
            setLastRunDebug({
                workflowId: workflow.id,
                sessionId: null,
                status: 'invoke_error',
                error: formatRuntimeError(e),
                timestamp: new Date().toISOString(),
            });
        }
    }, [workflow.id, nodes, edges, runInputs, runWorkflow, resetNodeStates, updateWorkflow, addToast]);

    const handleCopyDebugLog = useCallback(async () => {
        if (!lastRunDebug) return;
        const failedNodes = Object.values(workflowNodeStates)
            .filter((n) => n.status === 'error')
            .map((n) => `${n.nodeId}: ${n.error || 'unknown node error'}`);
        const debugText = [
            '[AI Studio Workflow Run Error]',
            `workflowId=${lastRunDebug.workflowId}`,
            `sessionId=${lastRunDebug.sessionId || 'n/a'}`,
            `status=${lastRunDebug.status}`,
            `time=${lastRunDebug.timestamp}`,
            `error=${lastRunDebug.error}`,
            `failedNodes=${failedNodes.length > 0 ? failedNodes.join(' | ') : 'none recorded'}`,
        ].join('\n');

        try {
            await navigator.clipboard.writeText(debugText);
            addToast('Workflow debug log copied', 'success');
        } catch {
            addToast('Failed to copy workflow debug log', 'error');
        }
    }, [lastRunDebug, workflowNodeStates, addToast]);

    // Duplicate node helper
    const duplicateNode = useCallback((nodeId: string) => {
        const node = nodes.find((n) => n.id === nodeId);
        if (!node) return;
        const newNode: Node = {
            ...node,
            id: generateNodeId(node.type || 'node'),
            position: { x: node.position.x + 50, y: node.position.y + 50 },
            data: { ...node.data },
            selected: false,
        };
        setNodes((nds) => [...nds, newNode]);
    }, [nodes, setNodes]);

    // Disconnect all edges from a node
    const disconnectNode = useCallback((nodeId: string) => {
        setEdges((eds) => eds.filter((e) => e.source !== nodeId && e.target !== nodeId));
    }, [setEdges]);

    // Keyboard shortcuts
    useEffect(() => {
        const handler = (e: KeyboardEvent) => {
            const inInput = e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement;

            // Ctrl+S: Save
            if ((e.metaKey || e.ctrlKey) && e.key === 's') {
                e.preventDefault();
                handleSave();
                return;
            }

            // Escape: Close context menu
            if (e.key === 'Escape') {
                setContextMenu(null);
                return;
            }

            if (inInput) return;

            // Delete/Backspace: Delete selected
            if (e.key === 'Delete' || e.key === 'Backspace') {
                if (selectedNode) {
                    setNodes((nds) => nds.filter((n) => n.id !== selectedNode.id));
                    setEdges((eds) => eds.filter((e) => e.source !== selectedNode.id && e.target !== selectedNode.id));
                    setSelectedNode(null);
                }
                return;
            }

            // Ctrl+D: Duplicate selected
            if ((e.metaKey || e.ctrlKey) && e.key === 'd') {
                e.preventDefault();
                if (selectedNode) duplicateNode(selectedNode.id);
                return;
            }

            // Ctrl+A: Select all
            if ((e.metaKey || e.ctrlKey) && e.key === 'a') {
                e.preventDefault();
                setNodes((nds) => nds.map((n) => ({ ...n, selected: true })));
                return;
            }

            // Ctrl+C: Copy selected nodes
            if ((e.metaKey || e.ctrlKey) && e.key === 'c') {
                const selected = nodes.filter((n) => n.selected || n.id === selectedNode?.id);
                if (selected.length === 0) return;
                const selectedIds = new Set(selected.map((n) => n.id));
                const connectedEdges = edges.filter((e) => selectedIds.has(e.source) && selectedIds.has(e.target));
                clipboardRef.current = { nodes: selected, edges: connectedEdges };
                return;
            }

            // Ctrl+V: Paste copied nodes
            if ((e.metaKey || e.ctrlKey) && e.key === 'v') {
                if (!clipboardRef.current) return;
                const { nodes: copiedNodes, edges: copiedEdges } = clipboardRef.current;
                const idMap = new Map<string, string>();
                const newNodes = copiedNodes.map((n) => {
                    const newId = generateNodeId(n.type || 'node');
                    idMap.set(n.id, newId);
                    return {
                        ...n,
                        id: newId,
                        position: { x: n.position.x + 60, y: n.position.y + 60 },
                        data: { ...n.data },
                        selected: true,
                    };
                });
                const newEdges = copiedEdges.map((edge) => ({
                    ...edge,
                    id: `e-${idMap.get(edge.source)}-${idMap.get(edge.target)}`,
                    source: idMap.get(edge.source) || edge.source,
                    target: idMap.get(edge.target) || edge.target,
                }));
                setNodes((nds) => [...nds.map((n) => ({ ...n, selected: false })), ...newNodes]);
                setEdges((eds) => [...eds, ...newEdges]);
                return;
            }
        };
        window.addEventListener('keydown', handler);
        return () => window.removeEventListener('keydown', handler);
    }, [handleSave, selectedNode, nodes, edges, setNodes, setEdges, duplicateNode]);

    // Update node data from config panel
    const handleNodeDataChange = useCallback((newData: Record<string, unknown>) => {
        if (!selectedNode) return;
        setNodes((nds) =>
            nds.map((n) => n.id === selectedNode.id ? { ...n, data: newData } : n)
        );
        setSelectedNode((prev) => prev ? { ...prev, data: newData } : null);
    }, [selectedNode, setNodes]);

    // Delete selected node
    const handleDeleteNode = useCallback(() => {
        if (!selectedNode) return;
        setNodes((nds) => nds.filter((n) => n.id !== selectedNode.id));
        setEdges((eds) => eds.filter((e) => e.source !== selectedNode.id && e.target !== selectedNode.id));
        setSelectedNode(null);
    }, [selectedNode, setNodes, setEdges]);

    return (
        <div className="flex flex-col h-full">
            {/* Top bar */}
            <div className="flex items-center justify-between px-4 py-2 border-b border-[var(--border-subtle)] bg-[var(--bg-secondary)]">
                <div className="flex items-center gap-3">
                    <button className="btn-icon" onClick={onBack} title="Back to list">
                        <ChevronLeft size={18} />
                    </button>
                    <span className="font-medium">{workflow.name}</span>
                    {hasChanges && (
                        <span className="text-xs text-yellow-400">unsaved</span>
                    )}
                </div>
                <div className="flex items-center gap-2">
                    <span className="text-xs text-[var(--text-muted)]">
                        {nodes.length} nodes
                    </span>
                    <button className="btn-secondary" onClick={handleSave} disabled={saving || !hasChanges}>
                        {saving ? <Loader2 size={14} className="animate-spin" /> : <Save size={14} />}
                        Save
                    </button>
                    <button
                        className="btn-primary"
                        disabled={workflowRunning || nodes.length === 0}
                        onClick={handleRunClick}
                        title={workflowRunning ? 'Workflow running...' : 'Run workflow'}
                    >
                        {workflowRunning ? <Loader2 size={14} className="animate-spin" /> : <Play size={14} />}
                        {workflowRunning ? 'Running...' : 'Run'}
                    </button>
                </div>
            </div>

            {lastRunResult && !lastRunDebug && (
                <div className="mx-4 mt-3 p-3 rounded border border-green-500/60 bg-green-950/20">
                    <div className="flex items-center justify-between gap-3">
                        <div className="flex items-center gap-2">
                            <Check size={16} className="text-green-400" />
                            <span className="text-sm font-medium text-green-300">Workflow completed</span>
                            <span className="text-xs text-[var(--text-muted)]">
                                {(lastRunResult.durationMs / 1000).toFixed(1)}s &middot; {lastRunResult.tokens} tokens &middot; {lastRunResult.nodeCount} nodes
                            </span>
                        </div>
                        <div className="flex items-center gap-2">
                            <button className="btn-secondary" onClick={() => openInspector(lastRunResult.sessionId)}>
                                Open Inspector
                            </button>
                            <button className="btn-icon" onClick={() => setLastRunResult(null)} title="Dismiss">
                                <X size={14} />
                            </button>
                        </div>
                    </div>
                    {Object.entries(lastRunResult.outputs).map(([key, value]) => (
                        <div key={key} className="mt-2">
                            <pre className="text-sm text-[var(--text-secondary)] bg-[var(--bg-primary)] p-3 rounded max-h-[300px] overflow-y-auto whitespace-pre-wrap font-mono">
                                {typeof value === 'string' ? value : JSON.stringify(value, null, 2)}
                            </pre>
                        </div>
                    ))}
                </div>
            )}

            {lastRunDebug && (
                <div className="mx-4 mt-3 p-3 rounded border border-red-500/60 bg-red-950/20">
                    <div className="flex items-center justify-between gap-3 mb-2">
                        <div className="text-sm font-medium text-red-300">
                            Last workflow run failed
                        </div>
                        <button className="btn-icon" onClick={() => setLastRunDebug(null)} title="Dismiss">
                            <X size={14} />
                        </button>
                    </div>
                    <div className="text-xs font-mono text-red-200 whitespace-pre-wrap break-words">
                        {lastRunDebug.error}
                    </div>
                    <div className="text-[11px] text-[var(--text-muted)] mt-2 font-mono break-all">
                        Session: {lastRunDebug.sessionId || 'n/a'}
                    </div>
                    <div className="flex items-center gap-2 mt-3">
                        <button className="btn-secondary" onClick={handleCopyDebugLog}>
                            <Copy size={14} />
                            Copy Debug Log
                        </button>
                        {lastRunDebug.sessionId && (
                            <button className="btn-secondary" onClick={() => openInspector(lastRunDebug.sessionId as string)}>
                                Open Inspector
                            </button>
                        )}
                    </div>
                </div>
            )}

            {/* Main editor area */}
            <div className="flex flex-1 min-h-0">
                {/* Node Palette */}
                <div className="w-48 border-r border-[var(--border-subtle)] bg-[var(--bg-secondary)] overflow-y-auto">
                    <div className="p-2">
                        <div className="text-xs font-semibold text-[var(--text-muted)] uppercase px-2 py-1">
                            Node Palette
                        </div>
                        {NODE_CATEGORIES.map((cat) => (
                            <div key={cat.label} className="mb-2">
                                <div className="text-[10px] text-[var(--text-muted)] uppercase px-2 py-1 mt-1">
                                    {cat.label}
                                </div>
                                {cat.types.map((t) => (
                                    <div
                                        key={t.type}
                                        className="flex items-center gap-2 px-2 py-1.5 rounded cursor-grab hover:bg-[var(--bg-tertiary)] text-sm"
                                        draggable
                                        onDragStart={(e) => {
                                            e.dataTransfer.setData('application/reactflow', t.type);
                                            e.dataTransfer.effectAllowed = 'move';
                                        }}
                                    >
                                        <div className="w-3 h-3 rounded-sm" style={{ background: nodeColors[t.type] }} />
                                        <span>{t.label}</span>
                                    </div>
                                ))}
                            </div>
                        ))}
                    </div>
                </div>

                {/* React Flow Canvas */}
                <div className="flex-1" ref={reactFlowRef}>
                    <ReactFlow
                        nodes={nodes}
                        edges={edges}
                        onNodesChange={onNodesChange}
                        onEdgesChange={onEdgesChange}
                        onConnect={onConnect}
                        onNodeClick={onNodeClick}
                        onPaneClick={onPaneClick}
                        onNodeContextMenu={(e, node) => {
                            e.preventDefault();
                            setSelectedNode(node);
                            setContextMenu({ x: e.clientX, y: e.clientY, nodeId: node.id });
                        }}
                        onPaneContextMenu={(e) => {
                            e.preventDefault();
                            setContextMenu({ x: e.clientX, y: e.clientY });
                        }}
                        onDragOver={onDragOver}
                        onDrop={onDrop}
                        nodeTypes={customNodeTypes}
                        defaultEdgeOptions={{ type: 'smoothstep', animated: false }}
                        defaultViewport={initialGraph.viewport}
                        fitView
                        deleteKeyCode={null}
                        className="bg-[var(--bg-primary)]"
                    >
                        <Background color="var(--border-subtle)" gap={20} />
                        <Controls className="react-flow-controls" />
                        <MiniMap
                            nodeColor={(n) => nodeColors[n.type || 'input'] || '#666'}
                            maskColor="rgba(0,0,0,0.6)"
                            className="react-flow-minimap"
                        />
                        {nodes.length === 0 && (
                            <Panel position="top-center">
                                <div className="text-sm text-[var(--text-muted)] mt-20 text-center">
                                    Drag nodes from the palette to get started
                                </div>
                            </Panel>
                        )}
                    </ReactFlow>

                    {/* Context Menu */}
                    {contextMenu && (
                        <div className="context-menu" style={{ left: contextMenu.x, top: contextMenu.y }}
                            onClick={() => setContextMenu(null)}>
                            {contextMenu.nodeId ? (
                                <>
                                    <div className="context-menu-item" onClick={() => { if (contextMenu.nodeId) duplicateNode(contextMenu.nodeId); }}>
                                        Duplicate <span className="shortcut">Ctrl+D</span>
                                    </div>
                                    <div className="context-menu-item" onClick={() => { if (contextMenu.nodeId) disconnectNode(contextMenu.nodeId); }}>
                                        Disconnect All
                                    </div>
                                    <div className="context-menu-divider" />
                                    <div className="context-menu-item" onClick={handleDeleteNode}>
                                        Delete <span className="shortcut">Del</span>
                                    </div>
                                </>
                            ) : (
                                <>
                                    {NODE_CATEGORIES.flatMap((cat) => cat.types).map((t) => (
                                        <div key={t.type} className="context-menu-item" onClick={() => {
                                            const bounds = reactFlowRef.current?.getBoundingClientRect();
                                            if (!bounds) return;
                                            const newNode: Node = {
                                                id: generateNodeId(t.type),
                                                type: t.type,
                                                position: { x: contextMenu.x - bounds.left - 80, y: contextMenu.y - bounds.top - 20 },
                                                data: defaultDataForType(t.type),
                                            };
                                            setNodes((nds) => [...nds, newNode]);
                                        }}>
                                            Add {t.label}
                                        </div>
                                    ))}
                                    <div className="context-menu-divider" />
                                    <div className="context-menu-item" onClick={() => setNodes((nds) => nds.map((n) => ({ ...n, selected: true })))}>
                                        Select All <span className="shortcut">Ctrl+A</span>
                                    </div>
                                </>
                            )}
                        </div>
                    )}
                </div>

                {/* Config Panel (right sidebar) */}
                {selectedNode && (
                    <div className="w-64 border-l border-[var(--border-subtle)] bg-[var(--bg-secondary)] overflow-y-auto">
                        <NodeConfigPanel
                            node={selectedNode}
                            onChange={handleNodeDataChange}
                            onDelete={handleDeleteNode}
                        />
                    </div>
                )}
            </div>

            {/* Run Input Modal */}
            {showRunModal && (
                <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50" onClick={() => setShowRunModal(false)}>
                    <div className="bg-[var(--bg-secondary)] border border-[var(--border-subtle)] rounded-lg p-6 w-[420px] max-h-[80vh] overflow-y-auto"
                        onClick={(e) => e.stopPropagation()}>
                        <h2 className="text-lg font-semibold mb-4">Run Workflow</h2>
                        {Object.keys(runInputs).length === 0 ? (
                            <p className="text-sm text-[var(--text-muted)] mb-4">
                                This workflow has no Input nodes. It will run with no inputs.
                            </p>
                        ) : (
                            <div className="space-y-3 mb-4">
                                {Object.entries(runInputs).map(([name, value]) => {
                                    const inputNode = nodes.find((n) => n.type === 'input' && (n.data.name as string) === name);
                                    const dataType = (inputNode?.data.dataType as string) || 'text';
                                    return (
                                        <label key={name} className="block">
                                            <span className="text-xs text-[var(--text-muted)] uppercase">{name}</span>
                                            {dataType === 'boolean' ? (
                                                <div className="mt-1">
                                                    <input
                                                        type="checkbox"
                                                        checked={Boolean(value)}
                                                        onChange={(e) => setRunInputs((prev) => ({ ...prev, [name]: e.target.checked }))}
                                                    />
                                                </div>
                                            ) : dataType === 'json' ? (
                                                <textarea
                                                    className="config-input min-h-[80px] font-mono text-xs"
                                                    value={typeof value === 'string' ? value : JSON.stringify(value, null, 2)}
                                                    onChange={(e) => setRunInputs((prev) => ({ ...prev, [name]: e.target.value }))}
                                                    placeholder='{"key": "value"}'
                                                />
                                            ) : (
                                                <input
                                                    className="config-input"
                                                    value={String(value ?? '')}
                                                    onChange={(e) => setRunInputs((prev) => ({ ...prev, [name]: e.target.value }))}
                                                    placeholder={`Enter ${name}...`}
                                                />
                                            )}
                                        </label>
                                    );
                                })}
                            </div>
                        )}
                        <div className="flex justify-end gap-2">
                            <button className="btn-secondary" onClick={() => setShowRunModal(false)}>Cancel</button>
                            <button className="btn-primary" onClick={handleRunSubmit}>
                                <Play size={14} /> Run
                            </button>
                        </div>
                    </div>
                </div>
            )}

            {/* Approval Dialog */}
            {approvalRequest && (
                <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
                    <div className="bg-[var(--bg-secondary)] border border-[var(--border-subtle)] rounded-lg p-6 w-[420px]">
                        <div className="flex items-center gap-2 mb-3">
                            <ShieldCheck size={20} className="text-yellow-400" />
                            <h2 className="text-lg font-semibold">Approval Required</h2>
                        </div>
                        <p className="text-sm mb-3">{approvalRequest.message}</p>
                        {approvalRequest.dataPreview && (
                            <pre className="text-xs bg-[var(--bg-tertiary)] p-3 rounded mb-4 overflow-auto max-h-[200px] font-mono">
                                {approvalRequest.dataPreview}
                            </pre>
                        )}
                        <div className="flex justify-end gap-2">
                            <button className="btn-secondary" onClick={() => handleApprovalDecision(false)}>
                                <X size={14} /> Reject
                            </button>
                            <button className="btn-primary" onClick={() => handleApprovalDecision(true)}>
                                <Check size={14} /> Approve
                            </button>
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
}

// ============================================
// MAIN PAGE
// ============================================

export function NodeEditorPage() {
    const { fetchWorkflow, createWorkflow, selectedWorkflow, setSelectedWorkflow } = useAppStore();
    const handleSelectWorkflow = async (id: string) => {
        await fetchWorkflow(id);
    };

    const handleCreate = async () => {
        const req: CreateWorkflowRequest = {
            name: `Workflow ${new Date().toLocaleDateString()}`,
            description: '',
        };
        const workflow = await createWorkflow(req);
        setSelectedWorkflow(workflow);
    };

    const handleBack = () => {
        setSelectedWorkflow(null);
    };

    if (selectedWorkflow) {
        return <WorkflowCanvas workflow={selectedWorkflow} onBack={handleBack} />;
    }

    return <WorkflowList onSelect={handleSelectWorkflow} onCreate={handleCreate} />;
}
