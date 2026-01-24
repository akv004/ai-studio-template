import { Play, RotateCcw, Check, Circle, Loader2, X } from 'lucide-react';
import { useAppStore } from '../../state/store';

/**
 * Runs Page
 * 
 * Features:
 * - Phase-based execution timeline
 * - Logs panel
 * - Run management
 */
export function RunsPage() {
    const { runPhases } = useAppStore();

    const statusIcons = {
        pending: Circle,
        running: Loader2,
        completed: Check,
        failed: X,
    };

    const statusStyles = {
        pending: '',
        running: 'active',
        completed: 'complete',
        failed: 'bg-[var(--status-error)]',
    };

    return (
        <div className="animate-fade-in h-full flex flex-col">
            {/* Page Header */}
            <div className="page-header">
                <div>
                    <h1 className="page-title">Runs</h1>
                    <p className="page-description">Monitor execution pipelines and logs</p>
                </div>
                <div className="flex items-center gap-2">
                    <button className="btn btn-secondary">
                        <RotateCcw className="w-4 h-4" />
                        Reset
                    </button>
                    <button className="btn btn-primary">
                        <Play className="w-4 h-4" />
                        Run Pipeline
                    </button>
                </div>
            </div>

            {/* Main Content */}
            <div className="flex-1 flex gap-4 mt-4 overflow-hidden">
                {/* Timeline Panel */}
                <div className="w-96 panel flex flex-col">
                    <div className="panel-header">
                        <span className="panel-title">Pipeline Phases</span>
                        <span className="text-xs text-[var(--text-muted)]">
                            {runPhases.filter(p => p.status === 'completed').length}/{runPhases.length} complete
                        </span>
                    </div>
                    <div className="flex-1 overflow-y-auto p-4">
                        <div className="timeline">
                            {runPhases.map((phase) => {
                                const Icon = statusIcons[phase.status];

                                return (
                                    <div key={phase.id} className="timeline-item">
                                        <div className={`timeline-dot ${statusStyles[phase.status]}`}>
                                            <Icon
                                                className={`w-3 h-3 ${phase.status === 'running' ? 'animate-spin' : ''} ${phase.status === 'completed' ? 'text-white' :
                                                    phase.status === 'failed' ? 'text-white' :
                                                        'text-[var(--text-muted)]'
                                                    }`}
                                            />
                                        </div>
                                        <div className="timeline-content">
                                            <div className="timeline-title">{phase.name}</div>
                                            <div className="timeline-description">
                                                {phase.status === 'running' && 'Processing...'}
                                                {phase.status === 'completed' && phase.completedAt && (
                                                    `Completed at ${new Date(phase.completedAt).toLocaleTimeString()}`
                                                )}
                                                {phase.status === 'pending' && 'Waiting...'}
                                                {phase.status === 'failed' && 'Failed'}
                                            </div>
                                        </div>
                                    </div>
                                );
                            })}
                        </div>
                    </div>
                </div>

                {/* Logs Panel */}
                <div className="flex-1 panel flex flex-col">
                    <div className="panel-header">
                        <span className="panel-title">Logs</span>
                        <div className="flex items-center gap-2">
                            <label className="flex items-center gap-2 text-sm">
                                <input type="checkbox" defaultChecked className="accent-[var(--accent-primary)]" />
                                <span className="text-[var(--text-muted)]">Auto-scroll</span>
                            </label>
                        </div>
                    </div>
                    <div className="flex-1 overflow-y-auto p-4 font-mono text-sm bg-[var(--bg-primary)]">
                        <div className="space-y-1">
                            {runPhases.flatMap((phase) =>
                                phase.logs.map((log, i) => (
                                    <div key={`${phase.id}-${i}`} className="flex gap-4">
                                        <span className="text-[var(--text-muted)] opacity-50 text-xs w-16">
                                            {phase.status === 'completed' ? '✓' : phase.status === 'running' ? '▶' : '○'}
                                        </span>
                                        <span className="text-[var(--accent-secondary)]">[{phase.name}]</span>
                                        <span className="text-[var(--text-primary)]">{log}</span>
                                    </div>
                                ))
                            )}

                            {/* Simulated live output */}
                            <div className="flex gap-4 opacity-70">
                                <span className="text-[var(--text-muted)] opacity-50 text-xs w-16">▶</span>
                                <span className="text-[var(--accent-secondary)]">[Training]</span>
                                <span className="text-[var(--text-primary)]">
                                    <span className="animate-pulse">Processing batch 1,247 / 2,500...</span>
                                </span>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}
