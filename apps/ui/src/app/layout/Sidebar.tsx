import {
    Bot,
    MessageSquare,
    Play,
    Search,
    Workflow,
    Settings
} from 'lucide-react';
import { useAppStore, type ModuleId } from '../../state/store';

interface NavItem {
    id: ModuleId;
    label: string;
    icon: React.ElementType;
    shortcut: string;
}

const mainNavItems: NavItem[] = [
    { id: 'agents', label: 'Agents', icon: Bot, shortcut: '⌘1' },
    { id: 'sessions', label: 'Sessions', icon: MessageSquare, shortcut: '⌘2' },
    { id: 'runs', label: 'Runs', icon: Play, shortcut: '⌘3' },
    { id: 'inspector', label: 'Inspector', icon: Search, shortcut: '⌘4' },
    { id: 'workflows', label: 'Workflows', icon: Workflow, shortcut: '⌘5' },
];

/**
 * Sidebar Navigation
 *
 * 6-pillar navigation: Agents, Sessions, Runs, Inspector, Workflows, Settings
 */
export function Sidebar() {
    const { activeModule, setActiveModule } = useAppStore();

    return (
        <aside className="app-sidebar">
            {/* Main Navigation */}
            <div className="sidebar-section">
                <div className="sidebar-section-title">Modules</div>
                <nav className="sidebar-nav">
                    {mainNavItems.map((item) => (
                        <button
                            key={item.id}
                            className={`sidebar-item ${activeModule === item.id ? 'active' : ''}`}
                            onClick={() => setActiveModule(item.id)}
                        >
                            <item.icon />
                            <span>{item.label}</span>
                            <span className="ml-auto text-xs text-[var(--text-muted)] opacity-0 group-hover:opacity-100">
                                {item.shortcut}
                            </span>
                        </button>
                    ))}
                </nav>
            </div>

            {/* Spacer */}
            <div className="flex-1" />

            {/* Settings Section */}
            <div className="sidebar-section border-t border-[var(--border-subtle)]">
                <nav className="sidebar-nav">
                    <button
                        className={`sidebar-item ${activeModule === 'settings' ? 'active' : ''}`}
                        onClick={() => setActiveModule('settings')}
                    >
                        <Settings />
                        <span>Settings</span>
                    </button>
                </nav>
            </div>

            {/* Version Info */}
            <div className="p-4 text-xs text-[var(--text-muted)]">
                v0.1.0
            </div>
        </aside>
    );
}
