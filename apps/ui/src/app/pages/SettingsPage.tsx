import { useState } from 'react';
import { Cpu, Keyboard, Palette, FolderOpen, Zap, Cloud, Check, AlertCircle, Loader2, Eye, EyeOff } from 'lucide-react';

type SettingsTab = 'providers' | 'models' | 'paths' | 'performance' | 'hotkeys' | 'appearance';

interface HotkeyConfig {
    action: string;
    shortcut: string;
}

interface ProviderConfig {
    id: string;
    name: string;
    icon: string;
    description: string;
    apiKeyEnv: string;
    models: string[];
    defaultModel: string;
    docsUrl: string;
}

/**
 * Settings Page
 * 
 * Features:
 * - AI Providers configuration (NEW)
 * - Models configuration
 * - Paths settings
 * - Performance toggles
 * - Hotkeys customization
 */
export function SettingsPage() {
    const [activeTab, setActiveTab] = useState<SettingsTab>('providers');
    const [providerStatus, setProviderStatus] = useState<Record<string, 'idle' | 'testing' | 'success' | 'error'>>({});
    const [showApiKeys, setShowApiKeys] = useState<Record<string, boolean>>({});

    const tabs: { id: SettingsTab; label: string; icon: React.ElementType }[] = [
        { id: 'providers', label: 'AI Providers', icon: Cloud },
        { id: 'models', label: 'Local Models', icon: Cpu },
        { id: 'paths', label: 'Paths', icon: FolderOpen },
        { id: 'performance', label: 'Performance', icon: Zap },
        { id: 'hotkeys', label: 'Hotkeys', icon: Keyboard },
        { id: 'appearance', label: 'Appearance', icon: Palette },
    ];

    const providers: ProviderConfig[] = [
        {
            id: 'ollama',
            name: 'Ollama (Local)',
            icon: '🦙',
            description: 'Run LLMs locally on your machine',
            apiKeyEnv: 'OLLAMA_HOST',
            models: ['llama3.2', 'llama3.1:70b', 'mistral', 'codellama', 'qwen2.5'],
            defaultModel: 'llama3.2',
            docsUrl: 'https://ollama.ai',
        },
        {
            id: 'anthropic',
            name: 'Anthropic',
            icon: '🔮',
            description: 'Claude models - Best for coding and analysis',
            apiKeyEnv: 'ANTHROPIC_API_KEY',
            models: ['claude-sonnet-4-20250514', 'claude-opus-4-20250514', 'claude-3-haiku-20240307'],
            defaultModel: 'claude-sonnet-4-20250514',
            docsUrl: 'https://console.anthropic.com',
        },
        {
            id: 'openai',
            name: 'OpenAI',
            icon: '🤖',
            description: 'GPT models - Versatile general-purpose AI',
            apiKeyEnv: 'OPENAI_API_KEY',
            models: ['gpt-4o', 'gpt-4o-mini', 'o1-preview', 'o1-mini'],
            defaultModel: 'gpt-4o',
            docsUrl: 'https://platform.openai.com/api-keys',
        },
        {
            id: 'google',
            name: 'Google AI Studio',
            icon: '✨',
            description: 'Gemini models - Google\'s multimodal AI',
            apiKeyEnv: 'GOOGLE_API_KEY',
            models: ['gemini-2.0-flash', 'gemini-2.0-flash-lite', 'gemini-1.5-pro', 'gemini-1.5-flash'],
            defaultModel: 'gemini-2.0-flash',
            docsUrl: 'https://aistudio.google.com/apikey',
        },
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

    const testConnection = async (providerId: string) => {
        setProviderStatus(prev => ({ ...prev, [providerId]: 'testing' }));

        try {
            const response = await fetch('http://localhost:8765/providers');
            const data = await response.json();

            if (data.providers?.includes(providerId)) {
                setProviderStatus(prev => ({ ...prev, [providerId]: 'success' }));
            } else {
                setProviderStatus(prev => ({ ...prev, [providerId]: 'error' }));
            }
        } catch {
            setProviderStatus(prev => ({ ...prev, [providerId]: 'error' }));
        }

        setTimeout(() => {
            setProviderStatus(prev => ({ ...prev, [providerId]: 'idle' }));
        }, 3000);
    };

    const toggleApiKeyVisibility = (providerId: string) => {
        setShowApiKeys(prev => ({ ...prev, [providerId]: !prev[providerId] }));
    };

    const renderProviderStatus = (providerId: string) => {
        const status = providerStatus[providerId];
        switch (status) {
            case 'testing':
                return <Loader2 className="w-4 h-4 animate-spin text-[var(--accent-primary)]" />;
            case 'success':
                return <Check className="w-4 h-4 text-green-400" />;
            case 'error':
                return <AlertCircle className="w-4 h-4 text-red-400" />;
            default:
                return null;
        }
    };

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
                        {activeTab === 'providers' && (
                            <div className="space-y-6">
                                <p className="text-sm text-[var(--text-secondary)]">
                                    Configure AI providers for chat, code generation, and agent tasks.
                                    API keys are stored locally and never sent to external servers.
                                </p>

                                {providers.map((provider) => (
                                    <div
                                        key={provider.id}
                                        className="p-4 rounded-lg bg-[var(--bg-tertiary)] border border-[var(--border-subtle)]"
                                    >
                                        {/* Provider Header */}
                                        <div className="flex items-center justify-between mb-4">
                                            <div className="flex items-center gap-3">
                                                <span className="text-2xl">{provider.icon}</span>
                                                <div>
                                                    <div className="font-medium">{provider.name}</div>
                                                    <div className="text-xs text-[var(--text-muted)]">
                                                        {provider.description}
                                                    </div>
                                                </div>
                                            </div>
                                            <div className="flex items-center gap-2">
                                                {renderProviderStatus(provider.id)}
                                                <button
                                                    className="btn btn-secondary btn-sm"
                                                    onClick={() => testConnection(provider.id)}
                                                    disabled={providerStatus[provider.id] === 'testing'}
                                                >
                                                    Test
                                                </button>
                                                <a
                                                    href={provider.docsUrl}
                                                    target="_blank"
                                                    rel="noopener noreferrer"
                                                    className="btn btn-secondary btn-sm"
                                                >
                                                    Get Key
                                                </a>
                                            </div>
                                        </div>

                                        {/* API Key Input */}
                                        <div className="grid grid-cols-2 gap-4">
                                            <div>
                                                <label className="block text-xs font-medium text-[var(--text-muted)] mb-1">
                                                    {provider.id === 'ollama' ? 'Host URL' : 'API Key'}
                                                </label>
                                                <div className="relative">
                                                    <input
                                                        type={showApiKeys[provider.id] ? 'text' : 'password'}
                                                        className="input w-full pr-10"
                                                        placeholder={provider.id === 'ollama' ? 'http://localhost:11434' : `Enter ${provider.apiKeyEnv}`}
                                                        defaultValue={provider.id === 'ollama' ? 'http://localhost:11434' : ''}
                                                    />
                                                    <button
                                                        type="button"
                                                        className="absolute right-2 top-1/2 -translate-y-1/2 p-1 text-[var(--text-muted)] hover:text-[var(--text-primary)]"
                                                        onClick={() => toggleApiKeyVisibility(provider.id)}
                                                    >
                                                        {showApiKeys[provider.id] ? (
                                                            <EyeOff className="w-4 h-4" />
                                                        ) : (
                                                            <Eye className="w-4 h-4" />
                                                        )}
                                                    </button>
                                                </div>
                                            </div>
                                            <div>
                                                <label className="block text-xs font-medium text-[var(--text-muted)] mb-1">
                                                    Default Model
                                                </label>
                                                <select className="input w-full" defaultValue={provider.defaultModel}>
                                                    {provider.models.map((model) => (
                                                        <option key={model} value={model}>
                                                            {model}
                                                        </option>
                                                    ))}
                                                </select>
                                            </div>
                                        </div>
                                    </div>
                                ))}

                                {/* Save Button */}
                                <div className="flex justify-end pt-4 border-t border-[var(--border-subtle)]">
                                    <button className="btn btn-primary">
                                        Save Provider Settings
                                    </button>
                                </div>
                            </div>
                        )}

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

