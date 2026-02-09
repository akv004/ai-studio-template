import { useState, useEffect } from 'react';
import { Play, Square, Clock, Check, Circle, Loader2, Bot } from 'lucide-react';
import { useAppStore } from '../../state/store';

/**
 * Runs Page
 *
 * Headless agent runs — fire-and-forget tasks with status tracking.
 * Wired to real SQLite data via Tauri IPC.
 */
export function RunsPage() {
    const { runs, runsLoading, fetchRuns, error } = useAppStore();
    const [selectedRunId, setSelectedRunId] = useState<string | undefined>();

    useEffect(() => {
        fetchRuns();
    }, [fetchRuns]);

    useEffect(() => {
        if (runs.length > 0 && !selectedRunId) {
            setSelectedRunId(runs[0].id);
        }
    }, [runs, selectedRunId]);

    const selectedRun = runs.find(r => r.id === selectedRunId);

    const statusIcons: Record<string, React.ElementType> = {
        pending: Circle,
        running: Loader2,
        completed: Check,
        failed: Square,
    };

    const statusColors: Record<string, string> = {
        pending: 'status-warning',
        running: 'status-info',
        completed: 'status-success',
        failed: 'status-error',
    };

    return (
        <div className="animate-fade-in h-full flex flex-col">
            <div className="page-header">
                <div>
                    <h1 className="page-title">Runs</h1>
                    <p className="page-description">Headless agent tasks — fire and forget</p>
                </div>
                <button className="btn btn-primary">
                    <Play className="w-4 h-4" />
                    New Run
                </button>
            </div>

            {error && (
                <div className="mt-2 p-3 rounded-lg bg-red-500/10 border border-red-500/30 text-red-400 text-sm">
                    {error}
                </div>
            )}

            <div className="flex-1 flex gap-4 mt-4 overflow-hidden">
                {/* Run List */}
                <div className="w-80 panel flex flex-col">
                    <div className="panel-header">
                        <span className="panel-title">Runs</span>
                        <span className="text-xs text-[var(--text-muted)]">{runs.length} total</span>
                    </div>
                    <div className="flex-1 overflow-y-auto p-2 space-y-2">
                        {runsLoading && runs.length === 0 && (
                            <div className="flex items-center justify-center p-8 text-[var(--text-muted)]">
                                <Loader2 className="w-5 h-5 animate-spin mr-2" /> Loading...
                            </div>
                        )}
                        {runs.map((run) => {
                            const Icon = statusIcons[run.status] || Circle;
                            return (
                                <div
                                    key={run.id}
                                    className={`p-3 rounded-lg cursor-pointer transition-all ${
                                        selectedRunId === run.id
                                            ? 'bg-[var(--accent-glow)] border border-[var(--accent-primary)]'
                                            : 'bg-[var(--bg-tertiary)] hover:bg-[var(--bg-hover)]'
                                    }`}
                                    onClick={() => setSelectedRunId(run.id)}
                                >
                                    <div className="flex items-center gap-3">
                                        <Icon className={`w-4 h-4 ${run.status === 'running' ? 'animate-spin' : ''}`} />
                                        <div className="flex-1 min-w-0">
                                            <div className="font-medium text-sm truncate">{run.name}</div>
                                            <div className="text-xs text-[var(--text-muted)]">{run.agentName}</div>
                                        </div>
                                        <span className={`status-pill ${statusColors[run.status] || ''}`}>
                                            <span className="status-dot" />
                                            {run.status}
                                        </span>
                                    </div>
                                </div>
                            );
                        })}
                        {!runsLoading && runs.length === 0 && (
                            <div className="text-center text-[var(--text-muted)] p-8 text-sm">
                                No runs yet.
                            </div>
                        )}
                    </div>
                </div>

                {/* Run Detail */}
                <div className="flex-1 panel flex flex-col">
                    <div className="panel-header">
                        <div className="flex items-center gap-3">
                            <Play className="w-5 h-5 text-[var(--accent-primary)]" />
                            <span className="panel-title">{selectedRun?.name || 'Select a run'}</span>
                        </div>
                        {selectedRun && (
                            <span className={`status-pill ${statusColors[selectedRun.status] || ''}`}>
                                <span className="status-dot" />
                                {selectedRun.status}
                            </span>
                        )}
                    </div>

                    {selectedRun ? (
                        <div className="panel-content space-y-6">
                            <div className="flex items-center gap-3">
                                <Bot className="w-5 h-5 text-[var(--text-muted)]" />
                                <div>
                                    <label className="block text-xs font-medium text-[var(--text-muted)]">Agent</label>
                                    <div className="text-sm">{selectedRun.agentName || selectedRun.agentId}</div>
                                </div>
                            </div>
                            <div>
                                <label className="block text-xs font-medium text-[var(--text-muted)] mb-1">Input</label>
                                <div className="text-sm bg-[var(--bg-tertiary)] p-3 rounded-lg whitespace-pre-wrap">{selectedRun.input}</div>
                            </div>
                            {selectedRun.output && (
                                <div>
                                    <label className="block text-xs font-medium text-[var(--text-muted)] mb-1">Output</label>
                                    <div className="text-sm bg-[var(--bg-tertiary)] p-3 rounded-lg whitespace-pre-wrap">{selectedRun.output}</div>
                                </div>
                            )}
                            {selectedRun.error && (
                                <div>
                                    <label className="block text-xs font-medium text-red-400 mb-1">Error</label>
                                    <div className="text-sm bg-red-500/10 p-3 rounded-lg text-red-400">{selectedRun.error}</div>
                                </div>
                            )}
                            <div className="flex gap-6 text-sm">
                                {selectedRun.totalTokens > 0 && (
                                    <div>
                                        <span className="text-[var(--text-muted)]">Tokens: </span>
                                        <span className="font-medium">{selectedRun.totalTokens.toLocaleString()}</span>
                                    </div>
                                )}
                                {selectedRun.totalCostUsd > 0 && (
                                    <div>
                                        <span className="text-[var(--text-muted)]">Cost: </span>
                                        <span className="font-medium text-green-400">${selectedRun.totalCostUsd.toFixed(4)}</span>
                                    </div>
                                )}
                                {selectedRun.durationMs != null && (
                                    <div>
                                        <span className="text-[var(--text-muted)]">Duration: </span>
                                        <span className="font-medium">{(selectedRun.durationMs / 1000).toFixed(1)}s</span>
                                    </div>
                                )}
                            </div>
                            <div className="flex gap-4 text-xs text-[var(--text-muted)]">
                                <span><Clock className="w-3 h-3 inline mr-1" />Created: {new Date(selectedRun.createdAt).toLocaleString()}</span>
                                {selectedRun.completedAt && (
                                    <span>Completed: {new Date(selectedRun.completedAt).toLocaleString()}</span>
                                )}
                            </div>
                        </div>
                    ) : (
                        <div className="flex-1 flex items-center justify-center text-[var(--text-muted)]">
                            Select a run to view details
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
}
