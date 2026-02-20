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
    type OnConnect,
    type ReactFlowInstance,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import {
    Save, Play, Copy, ChevronLeft,
    Loader2, Check, X, Download, ShieldCheck,
    Square, Radio, Settings2,
} from 'lucide-react';
import { useAppStore } from '../../../state/store';
import type { Workflow, LiveFeedItem } from '@ai-studio/shared';
import { customNodeTypes } from './nodeTypes';
import { NODE_CATEGORIES } from './nodeCategories';
import { nodeColors } from './nodeColors';
import { defaultDataForType } from './nodeDefaults';
import { NodeConfigPanel } from './NodeConfigPanel';
import { RichOutput } from './components/RichOutput';
import { generateNodeId, formatRuntimeError } from './utils';
import { TypedEdge } from './edges/TypedEdge';
import { TypedConnectionLine } from './edges/TypedConnectionLine';
import { LiveFeedPanel } from './LiveFeedPanel';

const edgeTypes = { typed: TypedEdge };

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

export function WorkflowCanvas({ workflow, onBack }: {
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
        liveMode,
        startLiveWorkflow,
        stopLiveWorkflow,
        pushLiveFeedItem,
        liveConfig,
        setLiveConfig,
        liveFeedItems,
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
    const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null);
    const [saving, setSaving] = useState(false);
    const [hasChanges, setHasChanges] = useState(false);
    const [showRunModal, setShowRunModal] = useState(false);
    const [runInputs, setRunInputs] = useState<Record<string, unknown>>({});
    const [approvalRequest, setApprovalRequest] = useState<{ id: string; message: string; dataPreview?: string } | null>(null);
    const [lastRunDebug, setLastRunDebug] = useState<LastRunDebugInfo | null>(null);
    const [lastRunResult, setLastRunResult] = useState<LastRunResult | null>(null);
    const [contextMenu, setContextMenu] = useState<{ x: number; y: number; nodeId?: string } | null>(null);
    const [editingName, setEditingName] = useState(false);
    const [nameDraft, setNameDraft] = useState(workflow.name);
    const [pendingNodeType, setPendingNodeType] = useState<string | null>(null);
    const [showLiveSettings, setShowLiveSettings] = useState(false);
    const clipboardRef = useRef<{ nodes: Node[]; edges: Edge[] } | null>(null);
    const reactFlowRef = useRef<HTMLDivElement>(null);
    const rfInstanceRef = useRef<ReactFlowInstance | null>(null);
    const nameInputRef = useRef<HTMLInputElement>(null);

    // A1: Derive selectedNode from nodes array — always fresh, never stale
    const selectedNode = selectedNodeId ? nodes.find(n => n.id === selectedNodeId) ?? null : null;

    const handleNameSubmit = useCallback(async () => {
        const trimmed = nameDraft.trim();
        if (trimmed && trimmed !== workflow.name) {
            try {
                await updateWorkflow(workflow.id, { name: trimmed });
                addToast('Workflow renamed', 'success');
            } catch {
                addToast('Failed to rename', 'error');
                setNameDraft(workflow.name);
            }
        } else {
            setNameDraft(workflow.name);
        }
        setEditingName(false);
    }, [nameDraft, workflow.id, workflow.name, updateWorkflow, addToast]);

    // Track changes
    useEffect(() => {
        setHasChanges(true);
    }, [nodes, edges]);

    // Listen for workflow node events to update execution state visuals
    useEffect(() => {
        let unlistenEvents: (() => void) | undefined;
        let unlistenApproval: (() => void) | undefined;
        let unlistenLive: (() => void) | undefined;

        (async () => {
            try {
                const { listen } = await import('@tauri-apps/api/event');

                // Live workflow feed events
                unlistenLive = await listen<LiveFeedItem>('live_workflow_feed', (event) => {
                    pushLiveFeedItem(event.payload);
                });

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
                            output: (payload.output_full || payload.output_preview || payload.output) as string | undefined,
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
            unlistenLive?.();
        };
    }, [setNodeState, pushLiveFeedItem]);

    // Handle type compatibility for connection validation
    const getHandleType = useCallback((nodeId: string, handleId: string | null, isSource: boolean): string => {
        const el = document.querySelector(
            `[data-nodeid="${nodeId}"] .react-flow__handle[data-handleid="${handleId || ''}"]`
            + (isSource ? '.source' : '.target')
        ) || document.querySelector(
            `[data-nodeid="${nodeId}"] .react-flow__handle[data-handleid="${handleId || ''}"]`
        );
        if (!el) return 'any';
        const classes = el.className;
        if (classes.includes('handle-text')) return 'text';
        if (classes.includes('handle-json')) return 'json';
        if (classes.includes('handle-bool')) return 'bool';
        if (classes.includes('handle-float')) return 'float';
        if (classes.includes('handle-number')) return 'number';
        if (classes.includes('handle-rows')) return 'rows';
        if (classes.includes('handle-binary')) return 'binary';
        return 'any';
    }, []);

    const isValidConnection = useCallback((connection: Edge | Connection): boolean => {
        if (connection.source === connection.target) return false;
        const sourceType = getHandleType(connection.source, connection.sourceHandle ?? null, true);
        const targetType = getHandleType(connection.target, connection.targetHandle ?? null, false);
        if (sourceType === 'any' || targetType === 'any') return true;
        if (sourceType === targetType) return true;
        // text ↔ json coercion
        if ((sourceType === 'text' && targetType === 'json') || (sourceType === 'json' && targetType === 'text')) return true;
        // number ↔ float (both numeric)
        if ((sourceType === 'number' && targetType === 'float') || (sourceType === 'float' && targetType === 'number')) return true;
        // number/float → text (stringify)
        if ((sourceType === 'number' || sourceType === 'float') && targetType === 'text') return true;
        // text → number/float (parse)
        if (sourceType === 'text' && (targetType === 'float' || targetType === 'number')) return true;
        // rows ↔ json (rows are JSON arrays)
        if ((sourceType === 'rows' && targetType === 'json') || (sourceType === 'json' && targetType === 'rows')) return true;
        // binary → text (base64 encode)
        if (sourceType === 'binary' && targetType === 'text') return true;
        return false;
    }, [getHandleType]);

    const onConnect: OnConnect = useCallback(
        (connection: Connection) => {
            const handleType = getHandleType(connection.source, connection.sourceHandle ?? null, true);
            const typedEdge = {
                ...connection,
                type: 'typed',
                data: { handleType },
            };
            setEdges((eds) => addEdge(typedEdge, eds));
        },
        [setEdges, getHandleType],
    );

    const onNodeClick = useCallback((_: React.MouseEvent, node: Node) => {
        setSelectedNodeId(node.id);
    }, []);

    const onPaneClick = useCallback((event: React.MouseEvent) => {
        setContextMenu(null);

        // Click-to-place: if a palette item is selected, place it at click position
        if (pendingNodeType) {
            const position = rfInstanceRef.current
                ? rfInstanceRef.current.screenToFlowPosition({ x: event.clientX, y: event.clientY })
                : { x: 100, y: 100 };

            const newNode: Node = {
                id: generateNodeId(pendingNodeType),
                type: pendingNodeType,
                position,
                data: defaultDataForType(pendingNodeType),
            };
            setNodes((nds) => [...nds, newNode]);
            setPendingNodeType(null);
            return;
        }

        setSelectedNodeId(null);
    }, [pendingNodeType, setNodes]);

    const onDragOver = useCallback((event: React.DragEvent) => {
        event.preventDefault();
        event.dataTransfer.dropEffect = 'move';
    }, []);

    const onDrop = useCallback(
        (event: React.DragEvent) => {
            event.preventDefault();
            const type = event.dataTransfer.getData('application/reactflow');
            if (!type) return;

            // Use screenToFlowPosition for correct placement at any zoom/pan
            const position = rfInstanceRef.current
                ? rfInstanceRef.current.screenToFlowPosition({ x: event.clientX, y: event.clientY })
                : (() => {
                    const bounds = reactFlowRef.current?.getBoundingClientRect();
                    if (!bounds) return { x: 100, y: 100 };
                    return { x: event.clientX - bounds.left, y: event.clientY - bounds.top };
                })();

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

    const handleRunClick = useCallback(() => {
        const defaults: Record<string, unknown> = {};
        nodes.forEach((n) => {
            if (n.type === 'input') {
                const name = (n.data.name as string) || 'input';
                const defaultVal = (n.data.defaultValue as string) ?? (n.data.default as string) ?? '';
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

    const handleGoLive = useCallback(async () => {
        // Auto-save before going live
        try {
            const graphJson = JSON.stringify({
                nodes,
                edges,
                viewport: { x: 0, y: 0, zoom: 1 },
            });
            await updateWorkflow(workflow.id, { graphJson });
            setHasChanges(false);
        } catch {
            addToast('Failed to save workflow before going live', 'error');
            return;
        }

        // Collect inputs from Input nodes
        const defaults: Record<string, unknown> = {};
        nodes.forEach((n) => {
            if (n.type === 'input') {
                const name = (n.data.name as string) || 'input';
                const defaultVal = (n.data.defaultValue as string) ?? (n.data.default as string) ?? '';
                defaults[name] = defaultVal;
            }
        });

        resetNodeStates();
        await startLiveWorkflow(workflow.id, defaults);
    }, [workflow.id, nodes, edges, updateWorkflow, addToast, resetNodeStates, startLiveWorkflow]);

    const handleStopLive = useCallback(async () => {
        await stopLiveWorkflow(workflow.id);
    }, [workflow.id, stopLiveWorkflow]);

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

    const disconnectNode = useCallback((nodeId: string) => {
        setEdges((eds) => eds.filter((e) => e.source !== nodeId && e.target !== nodeId));
    }, [setEdges]);

    // Keyboard shortcuts
    useEffect(() => {
        const handler = (e: KeyboardEvent) => {
            const inInput = e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement;

            if ((e.metaKey || e.ctrlKey) && e.key === 's') {
                e.preventDefault();
                handleSave();
                return;
            }

            if (e.key === 'Escape') {
                setContextMenu(null);
                setPendingNodeType(null);
                return;
            }

            if (inInput) return;

            if (e.key === 'Delete' || e.key === 'Backspace') {
                if (selectedNodeId) {
                    setNodes((nds) => nds.filter((n) => n.id !== selectedNodeId));
                    setEdges((eds) => eds.filter((e) => e.source !== selectedNodeId && e.target !== selectedNodeId));
                    setSelectedNodeId(null);
                }
                return;
            }

            if ((e.metaKey || e.ctrlKey) && e.key === 'd') {
                e.preventDefault();
                if (selectedNodeId) duplicateNode(selectedNodeId);
                return;
            }

            if ((e.metaKey || e.ctrlKey) && e.key === 'a') {
                e.preventDefault();
                setNodes((nds) => nds.map((n) => ({ ...n, selected: true })));
                return;
            }

            if ((e.metaKey || e.ctrlKey) && e.key === 'c') {
                const selected = nodes.filter((n) => n.selected || n.id === selectedNodeId);
                if (selected.length === 0) return;
                const selectedIds = new Set(selected.map((n) => n.id));
                const connectedEdges = edges.filter((e) => selectedIds.has(e.source) && selectedIds.has(e.target));
                clipboardRef.current = { nodes: selected, edges: connectedEdges };
                return;
            }

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
    }, [handleSave, selectedNodeId, nodes, edges, setNodes, setEdges, duplicateNode]);

    const handleNodeDataChange = useCallback((newData: Record<string, unknown>) => {
        if (!selectedNodeId) return;
        setNodes((nds) =>
            nds.map((n) => n.id === selectedNodeId ? { ...n, data: newData } : n)
        );
    }, [selectedNodeId, setNodes]);

    const handleDeleteNode = useCallback(() => {
        if (!selectedNodeId) return;
        setNodes((nds) => nds.filter((n) => n.id !== selectedNodeId));
        setEdges((eds) => eds.filter((e) => e.source !== selectedNodeId && e.target !== selectedNodeId));
        setSelectedNodeId(null);
    }, [selectedNodeId, setNodes, setEdges]);

    return (
        <div className="flex flex-col h-full">
            {/* Top bar */}
            <div className="flex items-center justify-between px-4 py-2 border-b border-[var(--border-subtle)] bg-[var(--bg-secondary)]">
                <div className="flex items-center gap-3">
                    <button className="btn-icon" onClick={onBack} title="Back to list">
                        <ChevronLeft size={18} />
                    </button>
                    {editingName ? (
                        <input
                            ref={nameInputRef}
                            className="font-medium bg-transparent border-b border-[var(--border-accent)] outline-none px-1 py-0 text-[var(--text-primary)] w-48"
                            value={nameDraft}
                            onChange={e => setNameDraft(e.target.value)}
                            onBlur={handleNameSubmit}
                            onKeyDown={e => {
                                if (e.key === 'Enter') handleNameSubmit();
                                if (e.key === 'Escape') { setNameDraft(workflow.name); setEditingName(false); }
                            }}
                            autoFocus
                        />
                    ) : (
                        <span
                            className="font-medium cursor-pointer hover:text-[var(--text-accent)] transition-colors"
                            onClick={() => { setEditingName(true); setNameDraft(workflow.name); }}
                            title="Click to rename"
                        >{workflow.name}</span>
                    )}
                    {hasChanges && (
                        <span className="text-xs text-yellow-400">unsaved</span>
                    )}
                </div>
                <div className="flex items-center gap-3">
                    <span className="px-2 py-0.5 rounded bg-[var(--bg-tertiary)] text-xs text-[var(--text-muted)]">
                        {nodes.length} nodes
                    </span>
                    <div className="toolbar-divider" />
                    <button className="btn-secondary" onClick={handleSave} disabled={saving || !hasChanges}>
                        {saving ? <Loader2 size={14} className="animate-spin" /> : <Save size={14} />}
                        Save
                    </button>
                    <button className="btn-icon btn-secondary" title="Export workflow as JSON" onClick={() => {
                        const graph = JSON.stringify({ nodes, edges, viewport: { x: 0, y: 0, zoom: 1 } }, null, 2);
                        const blob = new Blob([graph], { type: 'application/json' });
                        const url = URL.createObjectURL(blob);
                        const a = document.createElement('a');
                        a.href = url;
                        a.download = `${workflow.name.replace(/\s+/g, '-').toLowerCase()}.json`;
                        a.click();
                        URL.revokeObjectURL(url);
                    }}>
                        <Download size={14} />
                    </button>
                    <div className="toolbar-divider" />
                    <button
                        className="btn-primary"
                        disabled={workflowRunning || liveMode || nodes.length === 0}
                        onClick={handleRunClick}
                        title={workflowRunning ? 'Workflow running...' : 'Run workflow'}
                    >
                        {workflowRunning ? <Loader2 size={14} className="animate-spin" /> : <Play size={14} />}
                        {workflowRunning ? 'Running...' : 'Run'}
                    </button>
                    <div className="toolbar-divider" />
                    {liveMode ? (
                        <button
                            className="btn-primary bg-red-600 hover:bg-red-700"
                            onClick={handleStopLive}
                            title="Stop live workflow"
                        >
                            <Square size={14} />
                            Stop
                        </button>
                    ) : (
                        <button
                            className="btn-primary bg-green-700 hover:bg-green-600"
                            disabled={workflowRunning || nodes.length === 0}
                            onClick={handleGoLive}
                            title="Start continuous execution"
                        >
                            <Radio size={14} />
                            Go Live
                        </button>
                    )}
                    <div className="relative">
                        <button
                            className="btn-icon-sm"
                            onClick={() => setShowLiveSettings(!showLiveSettings)}
                            title="Live settings"
                        >
                            <Settings2 size={14} />
                        </button>
                        {showLiveSettings && (
                            <div
                                className="absolute right-0 top-full mt-1 w-64 bg-[var(--bg-secondary)] border border-[var(--border-subtle)] rounded-lg p-3 shadow-lg z-50"
                                onClick={(e) => e.stopPropagation()}
                            >
                                <div className="text-xs font-semibold text-[var(--text-muted)] uppercase mb-2">Live Settings</div>
                                <label className="block mb-2">
                                    <span className="text-xs text-[var(--text-muted)]">
                                        Interval: {(liveConfig.intervalMs / 1000).toFixed(0)}s
                                    </span>
                                    <input
                                        type="range"
                                        min={1000}
                                        max={60000}
                                        step={1000}
                                        value={liveConfig.intervalMs}
                                        onChange={(e) => setLiveConfig({ intervalMs: Number(e.target.value) })}
                                        className="w-full mt-1"
                                        disabled={liveMode}
                                    />
                                </label>
                                <label className="block mb-2">
                                    <span className="text-xs text-[var(--text-muted)]">Max iterations</span>
                                    <input
                                        type="number"
                                        className="config-input mt-1"
                                        value={liveConfig.maxIterations}
                                        onChange={(e) => setLiveConfig({ maxIterations: Number(e.target.value) || 1000 })}
                                        min={1}
                                        max={10000}
                                        disabled={liveMode}
                                    />
                                </label>
                                <label className="flex items-center gap-2 text-xs">
                                    <input
                                        type="checkbox"
                                        checked={liveConfig.errorPolicy === 'stop'}
                                        onChange={(e) => setLiveConfig({ errorPolicy: e.target.checked ? 'stop' : 'skip' })}
                                        disabled={liveMode}
                                    />
                                    <span className="text-[var(--text-muted)]">Stop on error</span>
                                </label>
                            </div>
                        )}
                    </div>
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
                    {Object.entries(lastRunResult.outputs).map(([key, value]) => {
                        const text = typeof value === 'string' ? value : JSON.stringify(value, null, 2);
                        return (
                            <div key={key} className="mt-2">
                                <RichOutput content={text} />
                            </div>
                        );
                    })}
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
                                        className={`flex items-center gap-2 px-2 py-1.5 rounded cursor-grab hover:bg-[var(--bg-tertiary)] text-sm ${pendingNodeType === t.type ? 'ring-1 ring-blue-500 bg-[var(--bg-tertiary)]' : ''}`}
                                        draggable
                                        onDragStart={(e) => {
                                            e.dataTransfer.setData('application/reactflow', t.type);
                                            e.dataTransfer.effectAllowed = 'move';
                                        }}
                                        onClick={() => setPendingNodeType(pendingNodeType === t.type ? null : t.type)}
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
                <div className={`flex-1 ${pendingNodeType ? 'cursor-crosshair' : ''}`} ref={reactFlowRef}>
                    <ReactFlow
                        nodes={nodes}
                        edges={edges}
                        onNodesChange={onNodesChange}
                        onEdgesChange={onEdgesChange}
                        onConnect={onConnect}
                        isValidConnection={isValidConnection}
                        onNodeClick={onNodeClick}
                        onPaneClick={onPaneClick}
                        onInit={(instance) => { rfInstanceRef.current = instance; }}
                        onNodeContextMenu={(e, node) => {
                            e.preventDefault();
                            setSelectedNodeId(node.id);
                            setContextMenu({ x: e.clientX, y: e.clientY, nodeId: node.id });
                        }}
                        onPaneContextMenu={(e) => {
                            e.preventDefault();
                            setContextMenu({ x: e.clientX, y: e.clientY });
                        }}
                        onDragOver={onDragOver}
                        onDrop={onDrop}
                        nodeTypes={customNodeTypes}
                        edgeTypes={edgeTypes}
                        connectionLineComponent={TypedConnectionLine}
                        defaultEdgeOptions={{ type: 'typed', animated: false }}
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
                        {pendingNodeType && (
                            <Panel position="top-center">
                                <div className="flex items-center gap-2 px-3 py-1.5 rounded bg-blue-600/90 text-white text-xs mt-2">
                                    <div className="w-2.5 h-2.5 rounded-sm" style={{ background: nodeColors[pendingNodeType] }} />
                                    Click on canvas to place {pendingNodeType} node
                                    <button className="ml-1 hover:text-blue-200" onClick={() => setPendingNodeType(null)}>
                                        <X className="w-3 h-3" />
                                    </button>
                                </div>
                            </Panel>
                        )}
                        {nodes.length === 0 && !pendingNodeType && (
                            <Panel position="top-center">
                                <div className="text-sm text-[var(--text-muted)] mt-20 text-center">
                                    Drag or click nodes from the palette, then click on canvas to place
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
                                            const position = rfInstanceRef.current
                                                ? rfInstanceRef.current.screenToFlowPosition({ x: contextMenu.x, y: contextMenu.y })
                                                : { x: contextMenu.x, y: contextMenu.y };
                                            const newNode: Node = {
                                                id: generateNodeId(t.type),
                                                type: t.type,
                                                position,
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

            {/* Live Feed Panel */}
            {(liveMode || liveFeedItems.length > 0) && <LiveFeedPanel />}

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
