import { Play, Pause, RotateCcw, Upload, Sparkles } from 'lucide-react';
import { useAppStore } from '../../state/store';

/**
 * Training Page
 * 
 * Features:
 * - Dataset table
 * - Augmentation toggles
 * - Fake progress bar
 */
export function TrainingPage() {
    const { trainingRuns } = useAppStore();

    // Mock datasets
    const datasets = [
        { id: '1', name: 'custom-objects-v2', samples: 10000, format: 'images', size: '2.4 GB' },
        { id: '2', name: 'voice-samples', samples: 5000, format: 'audio', size: '1.2 GB' },
        { id: '3', name: 'reviews-2024', samples: 50000, format: 'json', size: '156 MB' },
    ];

    // Mock augmentations
    const augmentations = [
        { id: 'flip', name: 'Horizontal Flip', enabled: true },
        { id: 'rotate', name: 'Random Rotation', enabled: true },
        { id: 'brightness', name: 'Brightness Adjust', enabled: false },
        { id: 'noise', name: 'Gaussian Noise', enabled: false },
        { id: 'crop', name: 'Random Crop', enabled: true },
    ];

    const statusColors = {
        queued: 'status-info',
        running: 'status-success',
        completed: 'status-info',
        failed: 'status-error',
    };

    return (
        <div className="animate-fade-in">
            {/* Page Header */}
            <div className="page-header">
                <div>
                    <h1 className="page-title">Training</h1>
                    <p className="page-description">Train and fine-tune your models</p>
                </div>
                <div className="flex items-center gap-2">
                    <button className="btn btn-secondary">
                        <Upload className="w-4 h-4" />
                        Import Dataset
                    </button>
                    <button className="btn btn-primary">
                        <Sparkles className="w-4 h-4" />
                        Start Training
                    </button>
                </div>
            </div>

            <div className="grid grid-cols-1 lg:grid-cols-3 gap-4 mt-6">
                {/* Datasets Panel */}
                <div className="lg:col-span-2 panel">
                    <div className="panel-header">
                        <span className="panel-title">Datasets</span>
                    </div>
                    <div className="table-container m-4">
                        <table className="table">
                            <thead>
                                <tr>
                                    <th>Name</th>
                                    <th>Samples</th>
                                    <th>Format</th>
                                    <th>Size</th>
                                    <th>Actions</th>
                                </tr>
                            </thead>
                            <tbody>
                                {datasets.map((ds) => (
                                    <tr key={ds.id}>
                                        <td className="font-medium">{ds.name}</td>
                                        <td>{ds.samples.toLocaleString()}</td>
                                        <td>
                                            <span className="px-2 py-1 rounded text-xs bg-[var(--bg-tertiary)]">
                                                {ds.format}
                                            </span>
                                        </td>
                                        <td className="text-[var(--text-muted)]">{ds.size}</td>
                                        <td>
                                            <button className="btn btn-ghost btn-sm">View</button>
                                        </td>
                                    </tr>
                                ))}
                            </tbody>
                        </table>
                    </div>
                </div>

                {/* Augmentations Panel */}
                <div className="panel">
                    <div className="panel-header">
                        <span className="panel-title">Augmentations</span>
                    </div>
                    <div className="panel-content space-y-3">
                        {augmentations.map((aug) => (
                            <label
                                key={aug.id}
                                className="flex items-center gap-3 p-2 rounded hover:bg-[var(--bg-tertiary)] cursor-pointer"
                            >
                                <input
                                    type="checkbox"
                                    defaultChecked={aug.enabled}
                                    className="w-4 h-4 rounded accent-[var(--accent-primary)]"
                                />
                                <span className="text-sm">{aug.name}</span>
                            </label>
                        ))}
                    </div>
                </div>
            </div>

            {/* Training Runs */}
            <div className="panel mt-4">
                <div className="panel-header">
                    <span className="panel-title">Training Runs</span>
                </div>
                <div className="p-4 space-y-4">
                    {trainingRuns.map((run) => (
                        <div
                            key={run.id}
                            className="p-4 rounded-lg bg-[var(--bg-tertiary)] border border-[var(--border-subtle)]"
                        >
                            <div className="flex items-center justify-between mb-3">
                                <div className="flex items-center gap-3">
                                    <span className="font-medium">{run.name}</span>
                                    <span className={`status-pill ${statusColors[run.status]}`}>
                                        <span className="status-dot" />
                                        {run.status}
                                    </span>
                                </div>
                                <div className="flex items-center gap-2">
                                    {run.status === 'running' ? (
                                        <button className="btn btn-ghost btn-sm">
                                            <Pause className="w-4 h-4" />
                                            Pause
                                        </button>
                                    ) : run.status === 'queued' ? (
                                        <button className="btn btn-ghost btn-sm">
                                            <Play className="w-4 h-4" />
                                            Start
                                        </button>
                                    ) : (
                                        <button className="btn btn-ghost btn-sm">
                                            <RotateCcw className="w-4 h-4" />
                                            Restart
                                        </button>
                                    )}
                                </div>
                            </div>

                            {/* Progress */}
                            <div className="space-y-2">
                                <div className="flex justify-between text-sm">
                                    <span className="text-[var(--text-muted)]">
                                        Epoch {run.currentEpoch} / {run.epochs}
                                    </span>
                                    <span className="text-[var(--text-muted)]">{run.progress}%</span>
                                </div>
                                <div className="progress-bar">
                                    <div
                                        className="progress-fill"
                                        style={{ width: `${run.progress}%` }}
                                    />
                                </div>
                            </div>

                            {/* Metrics */}
                            <div className="flex gap-6 mt-3 text-sm">
                                <div>
                                    <span className="text-[var(--text-muted)]">Dataset: </span>
                                    <span>{run.dataset}</span>
                                </div>
                                {run.status === 'running' && (
                                    <>
                                        <div>
                                            <span className="text-[var(--text-muted)]">Loss: </span>
                                            <span className="text-[var(--status-success)]">0.0234</span>
                                        </div>
                                        <div>
                                            <span className="text-[var(--text-muted)]">Accuracy: </span>
                                            <span className="text-[var(--status-success)]">94.2%</span>
                                        </div>
                                    </>
                                )}
                            </div>
                        </div>
                    ))}
                </div>
            </div>
        </div>
    );
}
