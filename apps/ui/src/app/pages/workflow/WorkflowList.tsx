import { useState, useEffect, useCallback, useRef } from 'react';
import {
    Plus, Trash2, Copy, RefreshCw, GitFork,
    Loader2, Upload, LayoutTemplate,
} from 'lucide-react';
import { useAppStore } from '../../../state/store';

interface TemplateSummary {
    id: string;
    name: string;
    description: string;
    nodeCount: number;
}

export function WorkflowList({ onSelect, onCreate, onCreateFromTemplate }: {
    onSelect: (id: string) => void;
    onCreate: () => void;
    onCreateFromTemplate: (templateId: string) => void;
}) {
    const { workflows, workflowsLoading, fetchWorkflows, deleteWorkflow, duplicateWorkflow, createWorkflow, addToast } = useAppStore();
    const [templates, setTemplates] = useState<TemplateSummary[]>([]);
    const [showTemplates, setShowTemplates] = useState(false);
    const fileInputRef = useRef<HTMLInputElement>(null);

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
                <h1 className="page-title">Node Editor</h1>
                <div className="flex gap-2">
                    <button className="btn-secondary" onClick={() => fetchWorkflows()}>
                        <RefreshCw size={16} />
                    </button>
                    <button className="btn-secondary" onClick={() => fileInputRef.current?.click()} title="Import workflow JSON">
                        <Upload size={16} />
                    </button>
                    <input
                        ref={fileInputRef}
                        type="file"
                        accept=".json"
                        className="hidden"
                        onChange={handleImport}
                    />
                    <div className="relative">
                        <button className="btn-secondary" onClick={() => setShowTemplates(!showTemplates)}>
                            <LayoutTemplate size={16} /> Templates
                        </button>
                        {showTemplates && templates.length > 0 && (
                            <div className="absolute right-0 top-full mt-1 w-72 bg-[var(--bg-secondary)] border border-[var(--border)] rounded-lg shadow-xl z-50">
                                {templates.map((t) => (
                                    <button
                                        key={t.id}
                                        className="w-full text-left px-3 py-2 hover:bg-[var(--bg-tertiary)] first:rounded-t-lg last:rounded-b-lg"
                                        onClick={() => {
                                            setShowTemplates(false);
                                            onCreateFromTemplate(t.id);
                                        }}
                                    >
                                        <div className="text-sm font-medium">{t.name}</div>
                                        <div className="text-xs text-[var(--text-muted)]">{t.description}</div>
                                        <div className="text-xs text-[var(--text-muted)] mt-0.5">{t.nodeCount} nodes</div>
                                    </button>
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
                    <GitFork size={48} className="text-[var(--text-muted)]" />
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
