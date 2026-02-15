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
    Plus, Save, Play, Trash2, Copy, ChevronLeft,
    Loader2, RefreshCw, MessageSquare, Wrench, GitFork,
    ShieldCheck, Repeat, FileInput, FileOutput, Cpu,
} from 'lucide-react';
import { useAppStore } from '../../state/store';
import type { Workflow, CreateWorkflowRequest } from '@ai-studio/shared';

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

const nodeColors: Record<string, string> = {
    input: '#22c55e',
    output: '#f59e0b',
    llm: '#6366f1',
    tool: '#ec4899',
    router: '#14b8a6',
    approval: '#eab308',
    transform: '#8b5cf6',
    subworkflow: '#06b6d4',
};

function InputNode({ data, selected }: { data: Record<string, unknown>; selected?: boolean }) {
    return (
        <div className={`custom-node ${selected ? 'selected' : ''}`} style={{ borderColor: nodeColors.input }}>
            <div className="custom-node-header" style={{ background: nodeColors.input }}>
                <FileInput size={14} /> INPUT
            </div>
            <div className="custom-node-body">
                <div className="text-xs text-[var(--text-muted)]">Name</div>
                <div className="text-sm font-medium">{(data.name as string) || 'untitled'}</div>
                <div className="text-xs text-[var(--text-muted)] mt-1">Type: {(data.dataType as string) || 'text'}</div>
            </div>
            <Handle type="source" position={Position.Right} className="custom-handle" />
        </div>
    );
}

function OutputNode({ data, selected }: { data: Record<string, unknown>; selected?: boolean }) {
    return (
        <div className={`custom-node ${selected ? 'selected' : ''}`} style={{ borderColor: nodeColors.output }}>
            <div className="custom-node-header" style={{ background: nodeColors.output }}>
                <FileOutput size={14} /> OUTPUT
            </div>
            <div className="custom-node-body">
                <div className="text-xs text-[var(--text-muted)]">Name</div>
                <div className="text-sm font-medium">{(data.name as string) || 'result'}</div>
                <div className="text-xs text-[var(--text-muted)] mt-1">Format: {(data.format as string) || 'text'}</div>
            </div>
            <Handle type="target" position={Position.Left} className="custom-handle" />
        </div>
    );
}

function LLMNode({ data, selected }: { data: Record<string, unknown>; selected?: boolean }) {
    return (
        <div className={`custom-node ${selected ? 'selected' : ''}`} style={{ borderColor: nodeColors.llm }}>
            <div className="custom-node-header" style={{ background: nodeColors.llm }}>
                <Cpu size={14} /> LLM
            </div>
            <div className="custom-node-body">
                <div className="text-sm font-medium">{(data.model as string) || 'Select model'}</div>
                <div className="text-xs text-[var(--text-muted)]">{(data.provider as string) || 'No provider'}</div>
                {Boolean(data.systemPrompt) && (
                    <div className="text-xs text-[var(--text-muted)] mt-1 truncate max-w-[160px]">
                        {(data.systemPrompt as string).slice(0, 40)}...
                    </div>
                )}
            </div>
            <Handle type="target" position={Position.Left} id="prompt" className="custom-handle" />
            <Handle type="source" position={Position.Right} id="response" className="custom-handle" />
        </div>
    );
}

function ToolNode({ data, selected }: { data: Record<string, unknown>; selected?: boolean }) {
    return (
        <div className={`custom-node ${selected ? 'selected' : ''}`} style={{ borderColor: nodeColors.tool }}>
            <div className="custom-node-header" style={{ background: nodeColors.tool }}>
                <Wrench size={14} /> TOOL
            </div>
            <div className="custom-node-body">
                <div className="text-sm font-medium">{(data.toolName as string) || 'Select tool'}</div>
                {Boolean(data.serverName) && (
                    <div className="text-xs text-[var(--text-muted)]">Server: {data.serverName as string}</div>
                )}
            </div>
            <Handle type="target" position={Position.Left} className="custom-handle" />
            <Handle type="source" position={Position.Right} id="result" className="custom-handle" />
        </div>
    );
}

function RouterNode({ data, selected }: { data: Record<string, unknown>; selected?: boolean }) {
    const branches = (data.branches as string[]) || ['true', 'false'];
    return (
        <div className={`custom-node ${selected ? 'selected' : ''}`} style={{ borderColor: nodeColors.router }}>
            <div className="custom-node-header" style={{ background: nodeColors.router }}>
                <GitFork size={14} /> ROUTER
            </div>
            <div className="custom-node-body">
                <div className="text-xs text-[var(--text-muted)]">Mode: {(data.mode as string) || 'pattern'}</div>
                <div className="text-xs mt-1">
                    {branches.map((b, i) => (
                        <span key={i} className="inline-block bg-[var(--bg-tertiary)] px-1.5 py-0.5 rounded mr-1 mb-1">{b}</span>
                    ))}
                </div>
            </div>
            <Handle type="target" position={Position.Left} className="custom-handle" />
            {branches.map((_, i) => (
                <Handle
                    key={i}
                    type="source"
                    position={Position.Right}
                    id={`branch-${i}`}
                    className="custom-handle"
                    style={{ top: `${30 + (i * 20)}%` }}
                />
            ))}
        </div>
    );
}

function ApprovalNode({ data, selected }: { data: Record<string, unknown>; selected?: boolean }) {
    return (
        <div className={`custom-node ${selected ? 'selected' : ''}`} style={{ borderColor: nodeColors.approval }}>
            <div className="custom-node-header" style={{ background: nodeColors.approval, color: '#000' }}>
                <ShieldCheck size={14} /> APPROVAL
            </div>
            <div className="custom-node-body">
                <div className="text-sm">{(data.message as string) || 'Approve?'}</div>
            </div>
            <Handle type="target" position={Position.Left} className="custom-handle" />
            <Handle type="source" position={Position.Right} id="approved" className="custom-handle" style={{ top: '35%' }} />
            <Handle type="source" position={Position.Right} id="rejected" className="custom-handle" style={{ top: '65%' }} />
        </div>
    );
}

function TransformNode({ data, selected }: { data: Record<string, unknown>; selected?: boolean }) {
    return (
        <div className={`custom-node ${selected ? 'selected' : ''}`} style={{ borderColor: nodeColors.transform }}>
            <div className="custom-node-header" style={{ background: nodeColors.transform }}>
                <Repeat size={14} /> TRANSFORM
            </div>
            <div className="custom-node-body">
                <div className="text-xs text-[var(--text-muted)]">Mode: {(data.mode as string) || 'template'}</div>
                {Boolean(data.template) && (
                    <div className="text-xs mt-1 truncate max-w-[160px] font-mono">
                        {(data.template as string).slice(0, 30)}
                    </div>
                )}
            </div>
            <Handle type="target" position={Position.Left} className="custom-handle" />
            <Handle type="source" position={Position.Right} className="custom-handle" />
        </div>
    );
}

function SubworkflowNode({ data, selected }: { data: Record<string, unknown>; selected?: boolean }) {
    return (
        <div className={`custom-node ${selected ? 'selected' : ''}`} style={{ borderColor: nodeColors.subworkflow }}>
            <div className="custom-node-header" style={{ background: nodeColors.subworkflow }}>
                <MessageSquare size={14} /> SUBWORKFLOW
            </div>
            <div className="custom-node-body">
                <div className="text-sm font-medium">{(data.workflowName as string) || 'Select workflow'}</div>
            </div>
            <Handle type="target" position={Position.Left} className="custom-handle" />
            <Handle type="source" position={Position.Right} className="custom-handle" />
        </div>
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
        case 'llm': return { provider: 'anthropic', model: 'claude-sonnet-4-5-20250929', systemPrompt: '', temperature: 0.7, maxTokens: 4096 };
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

function NodeConfigPanel({ node, onChange }: {
    node: Node;
    onChange: (data: Record<string, unknown>) => void;
}) {
    const data = node.data as Record<string, unknown>;
    const type = node.type || 'input';

    const update = (field: string, value: unknown) => {
        onChange({ ...data, [field]: value });
    };

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
                        <select className="config-input" value={(data.provider as string) || 'anthropic'}
                            onChange={(e) => update('provider', e.target.value)}>
                            <option value="anthropic">Anthropic</option>
                            <option value="google">Google</option>
                            <option value="azure_openai">Azure OpenAI</option>
                            <option value="ollama">Ollama (local)</option>
                        </select>
                    </label>
                    <label className="block">
                        <span className="text-xs text-[var(--text-muted)]">Model</span>
                        <input className="config-input" value={(data.model as string) || ''}
                            onChange={(e) => update('model', e.target.value)} />
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

            <div className="pt-2 text-xs text-[var(--text-muted)]">
                Node ID: {node.id}
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

function WorkflowCanvas({ workflow, onBack }: {
    workflow: Workflow;
    onBack: () => void;
}) {
    const { updateWorkflow, addToast } = useAppStore();

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
    const reactFlowRef = useRef<HTMLDivElement>(null);

    // Track changes
    useEffect(() => {
        setHasChanges(true);
    }, [nodes, edges]);

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

    // Keyboard shortcuts
    useEffect(() => {
        const handler = (e: KeyboardEvent) => {
            if ((e.metaKey || e.ctrlKey) && e.key === 's') {
                e.preventDefault();
                handleSave();
            }
            if (e.key === 'Delete' || e.key === 'Backspace') {
                if (selectedNode && !(e.target instanceof HTMLInputElement) && !(e.target instanceof HTMLTextAreaElement)) {
                    setNodes((nds) => nds.filter((n) => n.id !== selectedNode.id));
                    setEdges((eds) => eds.filter((e) => e.source !== selectedNode.id && e.target !== selectedNode.id));
                    setSelectedNode(null);
                }
            }
        };
        window.addEventListener('keydown', handler);
        return () => window.removeEventListener('keydown', handler);
    }, [handleSave, selectedNode, setNodes, setEdges]);

    // Update node data from config panel
    const handleNodeDataChange = useCallback((newData: Record<string, unknown>) => {
        if (!selectedNode) return;
        setNodes((nds) =>
            nds.map((n) => n.id === selectedNode.id ? { ...n, data: newData } : n)
        );
        setSelectedNode((prev) => prev ? { ...prev, data: newData } : null);
    }, [selectedNode, setNodes]);

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
                    <button className="btn-primary" disabled title="Run (Phase 3B)">
                        <Play size={14} /> Run
                    </button>
                </div>
            </div>

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
                        onDragOver={onDragOver}
                        onDrop={onDrop}
                        nodeTypes={customNodeTypes}
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
                </div>

                {/* Config Panel (right sidebar) */}
                {selectedNode && (
                    <div className="w-64 border-l border-[var(--border-subtle)] bg-[var(--bg-secondary)] overflow-y-auto">
                        <NodeConfigPanel
                            node={selectedNode}
                            onChange={handleNodeDataChange}
                        />
                    </div>
                )}
            </div>
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
