import { useState, useEffect, useCallback } from 'react';
import { Cpu, Keyboard, Palette, FolderOpen, Zap, Cloud, Check, AlertCircle, Loader2, Eye, EyeOff, Save, Plug, Plus, Trash2, Power, PowerOff, AlertTriangle } from 'lucide-react';
import { useAppStore } from '../../state/store';
import { fetchApi } from '../../services/api';

type SettingsTab = 'providers' | 'mcp' | 'models' | 'paths' | 'performance' | 'hotkeys' | 'appearance';

interface HotkeyConfig {
    action: string;
    shortcut: string;
}

interface ProviderField {
    key: string;
    label: string;
    placeholder: string;
    secret?: boolean;
}

interface ProviderConfig {
    id: string;
    name: string;
    icon: string;
    description: string;
    fields: ProviderField[];
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
    const { settings, fetchSettings, saveSetting, mcpServers, mcpServersLoading, fetchMcpServers, addMcpServer, updateMcpServer, removeMcpServer, wipeDatabase } = useAppStore();
    const [activeTab, setActiveTab] = useState<SettingsTab>('providers');
    const [providerStatus, setProviderStatus] = useState<Record<string, 'idle' | 'testing' | 'success' | 'error'>>({});
    const [showApiKeys, setShowApiKeys] = useState<Record<string, boolean>>({});
    const [toolDemoOutput, setToolDemoOutput] = useState<string>('');
    const [toolDemoStatus, setToolDemoStatus] = useState<'idle' | 'running' | 'done' | 'error'>('idle');
    // MCP form state
    const [mcpForm, setMcpForm] = useState({ name: '', command: '', args: '' });
    const [mcpAdding, setMcpAdding] = useState(false);
    const [wipeConfirm, setWipeConfirm] = useState(false);
    const [wiping, setWiping] = useState(false);
    // Provider form state — keyed by "provider_id.field_key"
    const [formValues, setFormValues] = useState<Record<string, string>>({});
    const [saving, setSaving] = useState(false);
    const [saveSuccess, setSaveSuccess] = useState(false);

    // Load settings + MCP servers from SQLite on mount
    useEffect(() => { fetchSettings(); fetchMcpServers(); }, [fetchSettings, fetchMcpServers]);

    // Sync stored settings → form state (only when settings load)
    useEffect(() => {
        const vals: Record<string, string> = {};
        for (const [key, value] of Object.entries(settings)) {
            if (key.startsWith('provider.')) {
                // key format: "provider.{id}.{field}" → form key: "{id}.{field}"
                const rest = key.slice('provider.'.length);
                vals[rest] = value;
            }
        }
        setFormValues(prev => {
            // Only update if there are new keys from DB (don't overwrite user edits)
            const merged = { ...prev };
            for (const [k, v] of Object.entries(vals)) {
                if (!(k in merged)) merged[k] = v;
            }
            return Object.keys(merged).length === Object.keys(prev).length ? prev : merged;
        });
    }, [settings]);

    const getFieldValue = useCallback((providerId: string, fieldKey: string) => {
        return formValues[`${providerId}.${fieldKey}`] || '';
    }, [formValues]);

    const setFieldValue = useCallback((providerId: string, fieldKey: string, value: string) => {
        setFormValues(prev => ({ ...prev, [`${providerId}.${fieldKey}`]: value }));
        setSaveSuccess(false);
    }, []);

    const handleSaveProviders = async () => {
        setSaving(true);
        try {
            for (const [formKey, value] of Object.entries(formValues)) {
                if (value.trim()) {
                    await saveSetting(`provider.${formKey}`, value);
                }
            }
            setSaveSuccess(true);
            setTimeout(() => setSaveSuccess(false), 3000);
        } catch { /* error in store */ }
        setSaving(false);
    };

    const tabs: { id: SettingsTab; label: string; icon: React.ElementType }[] = [
        { id: 'providers', label: 'AI Providers', icon: Cloud },
        { id: 'mcp', label: 'MCP Servers', icon: Plug },
        { id: 'models', label: 'Local Models', icon: Cpu },
        { id: 'paths', label: 'Paths', icon: FolderOpen },
        { id: 'performance', label: 'Performance', icon: Zap },
        { id: 'hotkeys', label: 'Hotkeys', icon: Keyboard },
        { id: 'appearance', label: 'Appearance', icon: Palette },
    ];

    const providers: ProviderConfig[] = [
        {
            id: 'anthropic',
            name: 'Anthropic',
            icon: '🔮',
            description: 'Claude models - Best for coding and analysis',
            fields: [
                { key: 'api_key', label: 'API Key', placeholder: 'sk-ant-...', secret: true },
            ],
            models: ['claude-sonnet-4-20250514', 'claude-opus-4-20250514', 'claude-haiku-4-5-20251001'],
            defaultModel: 'claude-sonnet-4-20250514',
            docsUrl: 'https://console.anthropic.com',
        },
        {
            id: 'google',
            name: 'Google AI Studio',
            icon: '✨',
            description: 'Gemini models - Google\'s multimodal AI',
            fields: [
                { key: 'api_key', label: 'API Key', placeholder: 'AIza...', secret: true },
            ],
            models: ['gemini-2.0-flash', 'gemini-2.0-flash-lite', 'gemini-1.5-pro', 'gemini-1.5-flash'],
            defaultModel: 'gemini-2.0-flash',
            docsUrl: 'https://aistudio.google.com/apikey',
        },
        {
            id: 'azure_openai',
            name: 'Azure OpenAI',
            icon: '☁️',
            description: 'OpenAI models via Azure - Enterprise grade',
            fields: [
                { key: 'endpoint', label: 'Endpoint', placeholder: 'https://myorg-openai-eastus2.openai.azure.com' },
                { key: 'api_key', label: 'API Key', placeholder: 'AZURE_OPENAI_API_KEY', secret: true },
                { key: 'deployment', label: 'Deployment Name', placeholder: 'gpt-4o-mini' },
                { key: 'api_version', label: 'API Version', placeholder: '2024-08-01-preview' },
            ],
            models: ['gpt-4o', 'gpt-4o-mini', 'gpt-4-turbo', 'gpt-35-turbo'],
            defaultModel: 'gpt-4o',
            docsUrl: 'https://portal.azure.com/#view/Microsoft_Azure_ProjectOxford/CognitiveServicesHub',
        },
        {
            id: 'local',
            name: 'Local / OpenAI-Compatible',
            icon: '🖥️',
            description: 'Ollama, vLLM, LM Studio, or any OpenAI-compatible server',
            fields: [
                { key: 'base_url', label: 'Base URL', placeholder: 'http://localhost:11434/v1' },
                { key: 'api_key', label: 'API Key (optional)', placeholder: 'leave empty if not required', secret: true },
                { key: 'model_name', label: 'Model Name', placeholder: 'llama3.2 or qwen3-vl-8b' },
            ],
            models: [],
            defaultModel: '',
            docsUrl: 'https://ollama.ai',
        },
    ];

    const models = [
        { id: 'yolov8', name: 'YOLOv8', path: '/models/yolov8.pt', size: '6.3 MB' },
        { id: 'whisper', name: 'Whisper Base', path: '/models/whisper-base.pt', size: '142 MB' },
        { id: 'llama', name: 'LLaMA 7B', path: '', size: '13 GB', missing: true },
    ];

    const hotkeys: HotkeyConfig[] = [
        { action: 'Open Command Palette', shortcut: '⌘K' },
        { action: 'Navigate to Agents', shortcut: '⌘1' },
        { action: 'Navigate to Sessions', shortcut: '⌘2' },
        { action: 'Navigate to Runs', shortcut: '⌘3' },
        { action: 'Navigate to Inspector', shortcut: '⌘4' },
        { action: 'Open Settings', shortcut: '⌘,' },
        { action: 'New Agent', shortcut: '⌘N' },
        { action: 'New Session', shortcut: '⌘⇧N' },
    ];

    const testConnection = async (providerId: string) => {
        setProviderStatus(prev => ({ ...prev, [providerId]: 'testing' }));

        try {
            // Build config from current form values
            const apiKey = getFieldValue(providerId, 'api_key');
            const baseUrl = getFieldValue(providerId, 'base_url') || getFieldValue(providerId, 'endpoint');
            const extraConfig: Record<string, string> = {};
            const providerCfg = providers.find(p => p.id === providerId);
            if (providerCfg) {
                for (const field of providerCfg.fields) {
                    if (field.key !== 'api_key' && field.key !== 'base_url' && field.key !== 'endpoint') {
                        const val = getFieldValue(providerId, field.key);
                        if (val) extraConfig[field.key] = val;
                    }
                }
            }

            const result = await fetchApi<{ success: boolean; message: string }>('/providers/test', {
                method: 'POST',
                body: JSON.stringify({
                    provider: providerId,
                    api_key: apiKey || undefined,
                    base_url: baseUrl || undefined,
                    extra_config: Object.keys(extraConfig).length > 0 ? extraConfig : undefined,
                }),
            });

            setProviderStatus(prev => ({
                ...prev,
                [providerId]: result.success ? 'success' : 'error',
            }));
        } catch (err) {
            console.error(`Test connection failed for ${providerId}:`, err);
            setProviderStatus(prev => ({ ...prev, [providerId]: 'error' }));
        }

        setTimeout(() => {
            setProviderStatus(prev => ({ ...prev, [providerId]: 'idle' }));
        }, 3000);
    };

    const runToolApprovalDemo = async () => {
        setToolDemoStatus('running');
        setToolDemoOutput('');
        try {
            const result = await fetchApi('/tools/shell', {
                method: 'POST',
                body: JSON.stringify({ command: 'pwd' }),
            });
            setToolDemoOutput(JSON.stringify(result, null, 2));
            setToolDemoStatus('done');
        } catch (e) {
            setToolDemoOutput(e instanceof Error ? e.message : String(e));
            setToolDemoStatus('error');
        }
    };

    const toggleApiKeyVisibility = (providerId: string) => {
        setShowApiKeys(prev => ({ ...prev, [providerId]: !prev[providerId] }));
    };

    const handleAddMcpServer = async () => {
        if (!mcpForm.name.trim() || !mcpForm.command.trim()) return;
        setMcpAdding(true);
        try {
            const args = mcpForm.args.trim()
                ? mcpForm.args.split(/\s+/)
                : [];
            await addMcpServer({
                name: mcpForm.name.trim(),
                command: mcpForm.command.trim(),
                args,
            });
            setMcpForm({ name: '', command: '', args: '' });
        } catch { /* error in store */ }
        setMcpAdding(false);
    };

    const handleToggleMcpServer = async (id: string, enabled: boolean) => {
        await updateMcpServer(id, { enabled });
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

                                        {/* Config Fields */}
                                        <div className={`grid gap-4 ${provider.fields.length > 2 ? 'grid-cols-2' : provider.fields.length === 1 ? 'grid-cols-2' : 'grid-cols-1'}`}>
                                            {provider.fields.map((field) => (
                                                <div key={field.key}>
                                                    <label className="block text-xs font-medium text-[var(--text-muted)] mb-1">
                                                        {field.label}
                                                    </label>
                                                    <div className="relative">
                                                        <input
                                                            type={field.secret && !showApiKeys[`${provider.id}.${field.key}`] ? 'password' : 'text'}
                                                            className="input w-full pr-10"
                                                            placeholder={field.placeholder}
                                                            value={getFieldValue(provider.id, field.key)}
                                                            onChange={e => setFieldValue(provider.id, field.key, e.target.value)}
                                                        />
                                                        {field.secret && (
                                                            <button
                                                                type="button"
                                                                className="absolute right-2 top-1/2 -translate-y-1/2 p-1 text-[var(--text-muted)] hover:text-[var(--text-primary)]"
                                                                onClick={() => toggleApiKeyVisibility(`${provider.id}.${field.key}`)}
                                                            >
                                                                {showApiKeys[`${provider.id}.${field.key}`] ? (
                                                                    <EyeOff className="w-4 h-4" />
                                                                ) : (
                                                                    <Eye className="w-4 h-4" />
                                                                )}
                                                            </button>
                                                        )}
                                                    </div>
                                                </div>
                                            ))}
                                            {provider.models.length > 0 && (
                                                <div>
                                                    <label className="block text-xs font-medium text-[var(--text-muted)] mb-1">
                                                        Default Model
                                                    </label>
                                                    <select
                                                        className="input w-full"
                                                        value={getFieldValue(provider.id, 'default_model') || provider.defaultModel}
                                                        onChange={e => setFieldValue(provider.id, 'default_model', e.target.value)}
                                                    >
                                                        {provider.models.map((model) => (
                                                            <option key={model} value={model}>
                                                                {model}
                                                            </option>
                                                        ))}
                                                    </select>
                                                </div>
                                            )}
                                        </div>
                                    </div>
                                ))}

                                {/* Save Button */}
                                <div className="flex items-center justify-end gap-3 pt-4 border-t border-[var(--border-subtle)]">
                                    {saveSuccess && (
                                        <span className="flex items-center gap-1 text-sm text-green-400">
                                            <Check className="w-4 h-4" /> Saved
                                        </span>
                                    )}
                                    <button
                                        className="btn btn-primary"
                                        onClick={handleSaveProviders}
                                        disabled={saving}
                                    >
                                        {saving ? (
                                            <><Loader2 className="w-4 h-4 animate-spin" /> Saving...</>
                                        ) : (
                                            <><Save className="w-4 h-4" /> Save Provider Settings</>
                                        )}
                                    </button>
                                </div>

                                {/* Tool approval demo (Desktop) */}
                                <div className="pt-4 border-t border-[var(--border-subtle)] space-y-2">
                                    <div className="flex items-center justify-between gap-3">
                                        <div>
                                            <div className="font-medium text-sm">Tool approval demo</div>
                                            <div className="text-xs text-[var(--text-muted)]">
                                                Runs a safe shell command via the sidecar. Desktop should prompt for approval.
                                            </div>
                                        </div>
                                        <button
                                            className="btn btn-secondary btn-sm"
                                            onClick={runToolApprovalDemo}
                                            disabled={toolDemoStatus === 'running'}
                                        >
                                            {toolDemoStatus === 'running' ? 'Running…' : 'Run "pwd"'}
                                        </button>
                                    </div>

                                    {toolDemoOutput && (
                                        <pre className="modal-pre">{toolDemoOutput}</pre>
                                    )}
                                </div>
                            </div>
                        )}

                        {activeTab === 'mcp' && (
                            <div className="space-y-6">
                                <p className="text-sm text-[var(--text-secondary)]">
                                    Add external MCP (Model Context Protocol) servers to give agents access to tools like databases, APIs, and file systems.
                                </p>

                                {/* Add new server form */}
                                <div className="p-4 rounded-lg bg-[var(--bg-tertiary)] border border-[var(--border-subtle)] space-y-3">
                                    <div className="font-medium text-sm">Add MCP Server</div>
                                    <div className="grid grid-cols-3 gap-3">
                                        <div>
                                            <label className="block text-xs font-medium text-[var(--text-muted)] mb-1">Name</label>
                                            <input
                                                className="input w-full"
                                                placeholder="e.g. filesystem"
                                                value={mcpForm.name}
                                                onChange={e => setMcpForm(f => ({ ...f, name: e.target.value }))}
                                            />
                                        </div>
                                        <div>
                                            <label className="block text-xs font-medium text-[var(--text-muted)] mb-1">Command</label>
                                            <input
                                                className="input w-full"
                                                placeholder="e.g. npx or python"
                                                value={mcpForm.command}
                                                onChange={e => setMcpForm(f => ({ ...f, command: e.target.value }))}
                                            />
                                        </div>
                                        <div>
                                            <label className="block text-xs font-medium text-[var(--text-muted)] mb-1">Args (space-separated)</label>
                                            <input
                                                className="input w-full"
                                                placeholder="e.g. @modelcontextprotocol/server-filesystem /tmp"
                                                value={mcpForm.args}
                                                onChange={e => setMcpForm(f => ({ ...f, args: e.target.value }))}
                                            />
                                        </div>
                                    </div>
                                    <div className="flex justify-end">
                                        <button
                                            className="btn btn-primary btn-sm"
                                            onClick={handleAddMcpServer}
                                            disabled={mcpAdding || !mcpForm.name.trim() || !mcpForm.command.trim()}
                                        >
                                            {mcpAdding ? <Loader2 className="w-4 h-4 animate-spin" /> : <Plus className="w-4 h-4" />}
                                            Add Server
                                        </button>
                                    </div>
                                </div>

                                {/* Server list */}
                                {mcpServersLoading ? (
                                    <div className="flex items-center gap-2 text-sm text-[var(--text-muted)]">
                                        <Loader2 className="w-4 h-4 animate-spin" /> Loading servers...
                                    </div>
                                ) : mcpServers.length === 0 ? (
                                    <div className="text-center py-8 text-[var(--text-muted)]">
                                        <Plug className="w-8 h-8 mx-auto mb-2 opacity-50" />
                                        <p className="text-sm">No MCP servers configured yet</p>
                                        <p className="text-xs mt-1">Add a server above to give your agents access to external tools</p>
                                    </div>
                                ) : (
                                    <div className="space-y-2">
                                        {mcpServers.map((server) => (
                                            <div
                                                key={server.id}
                                                className="flex items-center justify-between p-4 rounded-lg bg-[var(--bg-tertiary)] border border-[var(--border-subtle)]"
                                            >
                                                <div className="flex items-center gap-3">
                                                    <div className={`w-2 h-2 rounded-full ${server.enabled ? 'bg-green-400' : 'bg-gray-500'}`} />
                                                    <div>
                                                        <div className="font-medium text-sm">{server.name}</div>
                                                        <div className="text-xs text-[var(--text-muted)] font-mono">
                                                            {server.command} {server.args.join(' ')}
                                                        </div>
                                                    </div>
                                                </div>
                                                <div className="flex items-center gap-2">
                                                    <span className="text-xs text-[var(--text-muted)]">{server.transport}</span>
                                                    <button
                                                        className="btn btn-secondary btn-sm"
                                                        onClick={() => handleToggleMcpServer(server.id, !server.enabled)}
                                                        title={server.enabled ? 'Disable' : 'Enable'}
                                                    >
                                                        {server.enabled ? <Power className="w-4 h-4 text-green-400" /> : <PowerOff className="w-4 h-4" />}
                                                    </button>
                                                    <button
                                                        className="btn btn-secondary btn-sm text-red-400 hover:text-red-300"
                                                        onClick={() => removeMcpServer(server.id)}
                                                        title="Remove"
                                                    >
                                                        <Trash2 className="w-4 h-4" />
                                                    </button>
                                                </div>
                                            </div>
                                        ))}
                                    </div>
                                )}

                                {/* Built-in tools info */}
                                <div className="p-3 rounded-lg bg-[var(--bg-elevated)] border border-[var(--border-subtle)]">
                                    <div className="text-xs text-[var(--text-muted)]">
                                        <strong>Built-in tools:</strong> shell, read_file, write_file, list_directory are always available when tool calling is enabled.
                                        External MCP servers provide additional capabilities.
                                    </div>
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

                    {/* Danger Zone */}
                    <div className="px-5 pb-5 pt-2">
                        <div className="p-4 rounded-lg border border-red-500/30 bg-red-500/5">
                            <div className="flex items-center gap-2 mb-2">
                                <AlertTriangle className="w-4 h-4 text-red-400" />
                                <span className="font-medium text-sm text-red-400">Danger Zone</span>
                            </div>
                            <div className="flex items-center justify-between">
                                <div>
                                    <div className="text-sm">Wipe Database</div>
                                    <div className="text-xs text-[var(--text-muted)]">Delete all agents, sessions, runs, events, and settings. Cannot be undone.</div>
                                </div>
                                {!wipeConfirm ? (
                                    <button
                                        className="btn btn-sm text-red-400 border-red-500/30"
                                        onClick={() => setWipeConfirm(true)}
                                    >
                                        <Trash2 className="w-3.5 h-3.5" />
                                        Wipe All Data
                                    </button>
                                ) : (
                                    <div className="flex items-center gap-2">
                                        <span className="text-xs text-red-400">Are you sure?</span>
                                        <button
                                            className="btn btn-sm"
                                            onClick={() => setWipeConfirm(false)}
                                        >
                                            Cancel
                                        </button>
                                        <button
                                            className="btn btn-sm bg-red-600 hover:bg-red-700 text-white"
                                            disabled={wiping}
                                            onClick={async () => {
                                                setWiping(true);
                                                try {
                                                    await wipeDatabase();
                                                    setWipeConfirm(false);
                                                } catch { /* error in store */ }
                                                setWiping(false);
                                            }}
                                        >
                                            {wiping ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <Trash2 className="w-3.5 h-3.5" />}
                                            Confirm Wipe
                                        </button>
                                    </div>
                                )}
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}
