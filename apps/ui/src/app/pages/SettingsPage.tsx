import { useState } from 'react';
import { Cpu, Keyboard, Palette, FolderOpen, Zap } from 'lucide-react';

type SettingsTab = 'models' | 'paths' | 'performance' | 'hotkeys' | 'appearance';

interface HotkeyConfig {
    action: string;
    shortcut: string;
}

/**
 * Settings Page
 * 
 * Features:
 * - Models configuration
 * - Paths settings
 * - Performance toggles
 * - Hotkeys customization
 */
export function SettingsPage() {
    const [activeTab, setActiveTab] = useState<SettingsTab>('models');

    const tabs: { id: SettingsTab; label: string; icon: React.ElementType }[] = [
        { id: 'models', label: 'Models', icon: Cpu },
        { id: 'paths', label: 'Paths', icon: FolderOpen },
        { id: 'performance', label: 'Performance', icon: Zap },
        { id: 'hotkeys', label: 'Hotkeys', icon: Keyboard },
        { id: 'appearance', label: 'Appearance', icon: Palette },
    ];

    const models = [
        { id: 'yolov8', name: 'YOLOv8', path: '/models/yolov8.pt', size: '6.3 MB' },
        { id: 'whisper', name: 'Whisper Base', path: '/models/whisper-base.pt', size: '142 MB' },
        { id: 'llama', name: 'LLaMA 7B', path: '', size: '13 GB', missing: true },
    ];

    const hotkeys: HotkeyConfig[] = [
        { action: 'Open Command Palette', shortcut: '⌘K' },
        { action: 'Navigate to Projects', shortcut: '⌘1' },
        { action: 'Navigate to Vision', shortcut: '⌘2' },
        { action: 'Navigate to Audio', shortcut: '⌘3' },
        { action: 'Navigate to Agents', shortcut: '⌘4' },
        { action: 'Navigate to Training', shortcut: '⌘5' },
        { action: 'Navigate to Runs', shortcut: '⌘6' },
        { action: 'Open Settings', shortcut: '⌘,' },
        { action: 'New Project', shortcut: '⌘N' },
        { action: 'Start Training', shortcut: '⌘⇧T' },
    ];

    return (
        <div className="animate-fade-in">
            {/* Page Header */}
            <div className="page-header">
                <div>
                    <h1 className="page-title">Settings</h1>
                    <p className="page-description">Configure your AI Studio environment</p>
                </div>
            </div>

            <div className="flex gap-4 mt-6">
                {/* Tabs */}
                <div className="w-56 space-y-1">
                    {tabs.map((tab) => (
                        <button
                            key={tab.id}
                            className={`w-full sidebar-item ${activeTab === tab.id ? 'active' : ''}`}
                            onClick={() => setActiveTab(tab.id)}
                        >
                            <tab.icon className="w-4 h-4" />
                            <span>{tab.label}</span>
                        </button>
                    ))}
                </div>

                {/* Content */}
                <div className="flex-1 panel">
                    <div className="panel-header">
                        <span className="panel-title">
                            {tabs.find(t => t.id === activeTab)?.label}
                        </span>
                    </div>
                    <div className="panel-content">
                        {activeTab === 'models' && (
                            <div className="space-y-4">
                                <p className="text-sm text-[var(--text-secondary)] mb-4">
                                    Configure local model paths and download options.
                                </p>
                                {models.map((model) => (
                                    <div
                                        key={model.id}
                                        className="flex items-center justify-between p-4 rounded-lg bg-[var(--bg-tertiary)]"
                                    >
                                        <div className="flex items-center gap-3">
                                            <Cpu className="w-5 h-5 text-[var(--accent-primary)]" />
                                            <div>
                                                <div className="font-medium">{model.name}</div>
                                                <div className="text-xs text-[var(--text-muted)]">
                                                    {model.path || 'Not configured'}
                                                </div>
                                            </div>
                                        </div>
                                        <div className="flex items-center gap-3">
                                            <span className="text-sm text-[var(--text-muted)]">{model.size}</span>
                                            {model.missing ? (
                                                <button className="btn btn-primary btn-sm">Download</button>
                                            ) : (
                                                <span className="status-pill status-success">Installed</span>
                                            )}
                                        </div>
                                    </div>
                                ))}
                            </div>
                        )}

                        {activeTab === 'paths' && (
                            <div className="space-y-4">
                                <div>
                                    <label className="block text-sm font-medium mb-2">Projects Directory</label>
                                    <div className="flex gap-2">
                                        <input
                                            type="text"
                                            className="input flex-1"
                                            defaultValue="~/AI Studio/Projects"
                                        />
                                        <button className="btn btn-secondary">Browse</button>
                                    </div>
                                </div>
                                <div>
                                    <label className="block text-sm font-medium mb-2">Models Directory</label>
                                    <div className="flex gap-2">
                                        <input
                                            type="text"
                                            className="input flex-1"
                                            defaultValue="~/AI Studio/Models"
                                        />
                                        <button className="btn btn-secondary">Browse</button>
                                    </div>
                                </div>
                                <div>
                                    <label className="block text-sm font-medium mb-2">Cache Directory</label>
                                    <div className="flex gap-2">
                                        <input
                                            type="text"
                                            className="input flex-1"
                                            defaultValue="~/AI Studio/Cache"
                                        />
                                        <button className="btn btn-secondary">Browse</button>
                                    </div>
                                </div>
                            </div>
                        )}

                        {activeTab === 'performance' && (
                            <div className="space-y-4">
                                <label className="flex items-center justify-between p-4 rounded-lg bg-[var(--bg-tertiary)] cursor-pointer">
                                    <div>
                                        <div className="font-medium">GPU Acceleration</div>
                                        <div className="text-sm text-[var(--text-muted)]">Use GPU for model inference</div>
                                    </div>
                                    <input type="checkbox" defaultChecked className="w-5 h-5 accent-[var(--accent-primary)]" />
                                </label>
                                <label className="flex items-center justify-between p-4 rounded-lg bg-[var(--bg-tertiary)] cursor-pointer">
                                    <div>
                                        <div className="font-medium">Memory Optimization</div>
                                        <div className="text-sm text-[var(--text-muted)]">Reduce memory usage (may affect speed)</div>
                                    </div>
                                    <input type="checkbox" className="w-5 h-5 accent-[var(--accent-primary)]" />
                                </label>
                                <label className="flex items-center justify-between p-4 rounded-lg bg-[var(--bg-tertiary)] cursor-pointer">
                                    <div>
                                        <div className="font-medium">Background Processing</div>
                                        <div className="text-sm text-[var(--text-muted)]">Continue processing when window is minimized</div>
                                    </div>
                                    <input type="checkbox" defaultChecked className="w-5 h-5 accent-[var(--accent-primary)]" />
                                </label>
                                <div>
                                    <label className="block text-sm font-medium mb-2">Max Concurrent Workers</label>
                                    <input
                                        type="range"
                                        min="1"
                                        max="16"
                                        defaultValue="4"
                                        className="w-full"
                                    />
                                    <div className="flex justify-between text-xs text-[var(--text-muted)] mt-1">
                                        <span>1</span>
                                        <span>4 (default)</span>
                                        <span>16</span>
                                    </div>
                                </div>
                            </div>
                        )}

                        {activeTab === 'hotkeys' && (
                            <div className="space-y-2">
                                <p className="text-sm text-[var(--text-secondary)] mb-4">
                                    Keyboard shortcuts for quick navigation and actions.
                                </p>
                                {hotkeys.map((hk, i) => (
                                    <div
                                        key={i}
                                        className="flex items-center justify-between p-3 rounded-lg bg-[var(--bg-tertiary)]"
                                    >
                                        <span className="text-sm">{hk.action}</span>
                                        <kbd className="px-2 py-1 rounded bg-[var(--bg-elevated)] text-sm font-mono">
                                            {hk.shortcut}
                                        </kbd>
                                    </div>
                                ))}
                            </div>
                        )}

                        {activeTab === 'appearance' && (
                            <div className="space-y-4">
                                <div>
                                    <label className="block text-sm font-medium mb-2">Theme</label>
                                    <select className="input">
                                        <option>Dark (Default)</option>
                                        <option>Light</option>
                                        <option>System</option>
                                    </select>
                                </div>
                                <div>
                                    <label className="block text-sm font-medium mb-2">Accent Color</label>
                                    <div className="flex gap-2">
                                        {['#8b5cf6', '#3b82f6', '#22c55e', '#f59e0b', '#ef4444', '#ec4899'].map((color) => (
                                            <button
                                                key={color}
                                                className="w-8 h-8 rounded-full border-2 border-transparent hover:border-white/30"
                                                style={{ background: color }}
                                            />
                                        ))}
                                    </div>
                                </div>
                                <label className="flex items-center justify-between p-4 rounded-lg bg-[var(--bg-tertiary)] cursor-pointer">
                                    <div>
                                        <div className="font-medium">Reduce Motion</div>
                                        <div className="text-sm text-[var(--text-muted)]">Minimize animations</div>
                                    </div>
                                    <input type="checkbox" className="w-5 h-5 accent-[var(--accent-primary)]" />
                                </label>
                            </div>
                        )}
                    </div>
                </div>
            </div>
        </div>
    );
}
