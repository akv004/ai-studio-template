import { Plus, Search, MoreVertical, Clock } from 'lucide-react';
import { useAppStore } from '../../state/store';

/**
 * Projects Page
 * 
 * Features:
 * - Project list with cards
 * - Local JSON persistence (mocked)
 * - Create/Open/Delete actions
 */
export function ProjectsPage() {
    const { projects, selectedProjectId, selectProject } = useAppStore();

    const formatDate = (dateString: string) => {
        return new Date(dateString).toLocaleDateString('en-US', {
            month: 'short',
            day: 'numeric',
            year: 'numeric'
        });
    };

    return (
        <div className="animate-fade-in">
            {/* Page Header */}
            <div className="page-header">
                <div>
                    <h1 className="page-title">Projects</h1>
                    <p className="page-description">Manage your AI projects and models</p>
                </div>
                <div className="flex items-center gap-2">
                    <div className="relative">
                        <Search className="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-[var(--text-muted)]" />
                        <input
                            type="text"
                            className="input pl-9 w-64"
                            placeholder="Search projects..."
                        />
                    </div>
                    <button className="btn btn-primary">
                        <Plus className="w-4 h-4" />
                        New Project
                    </button>
                </div>
            </div>

            {/* Project Grid */}
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 mt-6">
                {projects.map((project) => (
                    <div
                        key={project.id}
                        className={`card cursor-pointer ${selectedProjectId === project.id ? 'card-selected' : ''}`}
                        onClick={() => selectProject(project.id)}
                    >
                        {/* Thumbnail */}
                        <div
                            className="h-32 rounded-md mb-4 flex items-center justify-center"
                            style={{
                                background: 'linear-gradient(135deg, var(--bg-tertiary) 0%, var(--bg-hover) 100%)'
                            }}
                        >
                            <div className="text-4xl opacity-30">🤖</div>
                        </div>

                        {/* Content */}
                        <div className="flex items-start justify-between">
                            <div className="flex-1 min-w-0">
                                <h3 className="font-semibold text-sm truncate">{project.name}</h3>
                                <p className="text-xs text-[var(--text-secondary)] mt-1 line-clamp-2">
                                    {project.description}
                                </p>
                            </div>
                            <button className="btn btn-ghost btn-icon ml-2">
                                <MoreVertical className="w-4 h-4" />
                            </button>
                        </div>

                        {/* Footer */}
                        <div className="flex items-center gap-2 mt-4 pt-4 border-t border-[var(--border-subtle)]">
                            <Clock className="w-3 h-3 text-[var(--text-muted)]" />
                            <span className="text-xs text-[var(--text-muted)]">
                                Updated {formatDate(project.updatedAt)}
                            </span>
                        </div>
                    </div>
                ))}

                {/* New Project Card */}
                <div
                    className="card cursor-pointer border-dashed flex flex-col items-center justify-center h-64 hover:border-[var(--accent-primary)]"
                    onClick={() => console.log('Create new project')}
                >
                    <div
                        className="w-12 h-12 rounded-full flex items-center justify-center mb-3"
                        style={{ background: 'var(--accent-glow)' }}
                    >
                        <Plus className="w-6 h-6 text-[var(--accent-primary)]" />
                    </div>
                    <span className="text-sm font-medium">Create New Project</span>
                    <span className="text-xs text-[var(--text-muted)] mt-1">Start from scratch</span>
                </div>
            </div>
        </div>
    );
}
