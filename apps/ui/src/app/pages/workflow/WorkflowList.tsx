import { useState, useEffect, useCallback, useRef } from 'react';
import {
    Plus, Trash2, Copy, RefreshCw, Workflow,
    Loader2, Upload, LayoutTemplate, Pencil, Check, X,
} from 'lucide-react';
import { useAppStore } from '../../../state/store';

interface TemplateSummary {
    id: string;
    name: string;
    description: string;
    nodeCount: number;
    source: string;
}

export function WorkflowList({ onSelect, onCreate, onCreateFromTemplate }: {
    onSelect: (id: string) => void;
    onCreate: () => void;
    onCreateFromTemplate: (templateId: string) => void;
}) {
    const { workflows, workflowsLoading, fetchWorkflows, deleteWorkflow, duplicateWorkflow, createWorkflow, updateWorkflow, addToast } = useAppStore();
    const [templates, setTemplates] = useState<TemplateSummary[]>([]);
    const [showTemplates, setShowTemplates] = useState(false);
    const [renamingId, setRenamingId] = useState<string | null>(null);
    const [renameDraft, setRenameDraft] = useState('');
    const fileInputRef = useRef<HTMLInputElement>(null);

    const handleRenameSubmit = useCallback(async (id: string) => {
        const trimmed = renameDraft.trim();
        if (trimmed) {
            try {
                await updateWorkflow(id, { name: trimmed });
                addToast('Renamed', 'success');
            } catch {
                addToast('Failed to rename', 'error');
            }
        }
        setRenamingId(null);
    }, [renameDraft, updateWorkflow, addToast]);

    useEffect(() => {
        fetchWorkflows();
        (async () => {
            try {
                const { invoke } = await import('@tauri-apps/api/core');
                const list = await invoke<TemplateSummary[]>('list_templates');
                setTemplates(list);
            } catch {
                // Templates not available (browser dev mode)
            }
        })();
    }, [fetchWorkflows]);

    const handleImport = useCallback(async (e: React.ChangeEvent<HTMLInputElement>) => {
        const file = e.target.files?.[0];
        if (!file) return;
        try {
            const text = await file.text();
            const graph = JSON.parse(text);
            if (!graph.nodes || !graph.edges) {
                addToast('Invalid workflow file: missing nodes or edges', 'error');
                return;
            }
            const name = file.name.replace(/\.json$/i, '');
            const workflow = await createWorkflow({
                name,
                description: 'Imported workflow',
                graphJson: JSON.stringify(graph),
            });
            onSelect(workflow.id);
            addToast('Workflow imported', 'success');
        } catch {
            addToast('Failed to import workflow', 'error');
        }
        if (fileInputRef.current) fileInputRef.current.value = '';
    }, [createWorkflow, onSelect, addToast]);

    return (
        <div className="page-content">
            <div className="page-header">
                <h1 className="page-title">Workflows</h1>
                <div className="flex items-center gap-3">
                    <button className="btn-icon-sm btn-secondary" onClick={() => fetchWorkflows()} title="Refresh">
                        <RefreshCw size={14} />
                    </button>
                    <button className="btn-icon-sm btn-secondary" onClick={() => fileInputRef.current?.click()} title="Import workflow JSON">
                        <Upload size={14} />
                    </button>
                    <input
                        ref={fileInputRef}
                        type="file"
                        accept=".json"
                        className="hidden"
                        onChange={handleImport}
                    />
                    <div className="toolbar-divider" />
                    <div className="relative">
                        <button className="btn-secondary" onClick={() => setShowTemplates(!showTemplates)}>
                            <LayoutTemplate size={16} /> Templates
                        </button>
                        {showTemplates && templates.length > 0 && (
                            <div className="absolute right-0 top-full mt-1 w-80 bg-[var(--bg-secondary)] border border-[var(--border)] rounded-lg shadow-xl z-50 max-h-[60vh] overflow-y-auto">
                                {templates.map((t) => (
                                    <div
                                        key={t.id}
                                        className="group/tpl flex items-center px-3 py-2 hover:bg-[var(--bg-tertiary)] first:rounded-t-lg last:rounded-b-lg"
                                    >
                                        <button
                                            className="flex-1 text-left min-w-0"
                                            onClick={() => {
                                                setShowTemplates(false);
                                                onCreateFromTemplate(t.id);
                                            }}
                                        >
                                            <div className="flex items-center gap-1.5">
                                                <span className="text-sm font-medium truncate">{t.name}</span>
                                                {t.source === 'user' && (
                                                    <span className="shrink-0 px-1.5 py-0 rounded text-[10px] bg-blue-500/20 text-blue-300">saved</span>
                                                )}
                                            </div>
                                            <div className="text-xs text-[var(--text-muted)] truncate">{t.description}</div>
                                            <div className="text-xs text-[var(--text-muted)] mt-0.5">{t.nodeCount} nodes</div>
                                        </button>
                                        {t.source === 'user' && (
                                            <button
                                                className="shrink-0 ml-2 p-1 rounded text-red-400 opacity-0 group-hover/tpl:opacity-100 hover:bg-red-500/20 transition-opacity"
                                                title="Delete template"
                                                onClick={async (e) => {
                                                    e.stopPropagation();
                                                    try {
                                                        const { invoke } = await import('@tauri-apps/api/core');
                                                        await invoke('delete_user_template', { templateId: t.id });
                                                        setTemplates((prev) => prev.filter((x) => x.id !== t.id));
                                                        addToast('Template deleted', 'success');
                                                    } catch {
                                                        addToast('Failed to delete template', 'error');
                                                    }
                                                }}
                                            >
                                                <Trash2 size={14} />
                                            </button>
                                        )}
                                    </div>
                                ))}
                            </div>
                        )}
                    </div>
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
                    <Workflow size={48} className="text-[var(--text-muted)]" />
                    <h2>No workflows yet</h2>
                    <p>Create your first workflow to build AI pipelines visually.</p>
                    <div className="flex gap-3 mt-4">
                        <button className="btn-primary" onClick={onCreate}>
                            <Plus size={16} /> New Workflow
                        </button>
                        {templates.length > 0 && (
                            <button className="btn-secondary" onClick={() => setShowTemplates(true)}>
                                <LayoutTemplate size={16} /> From Template
                            </button>
                        )}
                    </div>
                </div>
            ) : (
                <div className="grid gap-3">
                    {workflows.map((w) => (
                        <div key={w.id}
                            className="group card cursor-pointer hover:border-[var(--border-accent)]"
                            onClick={() => onSelect(w.id)}>
                            <div className="flex items-center justify-between">
                                <div className="flex-1 min-w-0">
                                    {renamingId === w.id ? (
                                        <div className="flex items-center gap-1" onClick={e => e.stopPropagation()}>
                                            <input
                                                className="font-medium bg-transparent border-b border-[var(--border-accent)] outline-none px-1 text-[var(--text-primary)] w-full"
                                                value={renameDraft}
                                                onChange={e => setRenameDraft(e.target.value)}
                                                onBlur={() => handleRenameSubmit(w.id)}
                                                onKeyDown={e => {
                                                    if (e.key === 'Enter') handleRenameSubmit(w.id);
                                                    if (e.key === 'Escape') setRenamingId(null);
                                                }}
                                                autoFocus
                                            />
                                            <button className="btn-icon" onClick={() => handleRenameSubmit(w.id)}><Check size={12} /></button>
                                            <button className="btn-icon" onClick={() => setRenamingId(null)}><X size={12} /></button>
                                        </div>
                                    ) : (
                                        <div className="font-medium truncate">{w.name}</div>
                                    )}
                                    {w.description && (
                                        <div className="text-sm text-[var(--text-muted)] mt-0.5">{w.description}</div>
                                    )}
                                    <div className="text-xs text-[var(--text-muted)] mt-1">
                                        {w.nodeCount} nodes
                                    </div>
                                </div>
                                <div className="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity" onClick={(e) => e.stopPropagation()}>
                                    <button className="btn-icon hover:bg-[var(--bg-elevated)]" title="Rename"
                                        onClick={() => { setRenamingId(w.id); setRenameDraft(w.name); }}>
                                        <Pencil size={14} />
                                    </button>
                                    <button className="btn-icon hover:bg-[var(--bg-elevated)]" title="Duplicate"
                                        onClick={() => duplicateWorkflow(w.id)}>
                                        <Copy size={14} />
                                    </button>
                                    <button className="btn-icon text-red-400 hover:bg-[var(--bg-elevated)]" title="Delete"
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
