import { useState, useEffect } from 'react';
import { Plus, RefreshCw, Bot, Trash2, Loader2, Shield, Server, Zap } from 'lucide-react';
import { useAppStore } from '../../state/store';
import type { ToolsMode, RoutingMode } from '@ai-studio/shared';

// TODO: Source from `get_model_capabilities` IPC to stay in sync with MODEL_CAPABILITIES in routing.rs
const MODELS_BY_PROVIDER: Record<string, string[]> = {
    anthropic: ['claude-sonnet-4-20250514', 'claude-opus-4-20250514', 'claude-haiku-4-5-20251001'],
    google: ['gemini-2.0-flash', 'gemini-2.0-flash-lite', 'gemini-1.5-pro', 'gemini-1.5-flash'],
    azure_openai: ['gpt-4o', 'gpt-4o-mini', 'gpt-4-turbo', 'gpt-35-turbo'],
    local: [],
};

/**
 * Agents Page
 *
 * Create and configure AI agents with different models, prompts, tools, and permissions.
 * Wired to real SQLite CRUD via Tauri IPC.
 */
export function AgentsPage() {
    const {
        agents, agentsLoading, fetchAgents, createAgent, deleteAgent, error,
        mcpServers, fetchMcpServers,
    } = useAppStore();
    const [selectedAgentId, setSelectedAgentId] = useState<string | undefined>();
    const [showCreate, setShowCreate] = useState(false);
    const [creating, setCreating] = useState(false);

    // Form state
    const [name, setName] = useState('');
    const [provider, setProvider] = useState('anthropic');
    const [model, setModel] = useState('claude-sonnet-4-20250514');
    const [customModel, setCustomModel] = useState(false);
    const [systemPrompt, setSystemPrompt] = useState('');
    const [toolsMode, setToolsMode] = useState<ToolsMode>('restricted');
    const [routingMode, setRoutingMode] = useState<RoutingMode>('single');
    const [selectedMcpServers, setSelectedMcpServers] = useState<string[]>([]);

    const availableModels = MODELS_BY_PROVIDER[provider] || [];

    const handleProviderChange = (newProvider: string) => {
        setProvider(newProvider);
        setCustomModel(false);
        const models = MODELS_BY_PROVIDER[newProvider] || [];
        setModel(models[0] || '');
    };

    useEffect(() => {
        fetchAgents();
        fetchMcpServers();
    }, [fetchAgents, fetchMcpServers]);

    useEffect(() => {
        if (agents.length > 0 && !selectedAgentId) {
            setSelectedAgentId(agents[0].id);
        }
    }, [agents, selectedAgentId]);

    const selectedAgent = agents.find(a => a.id === selectedAgentId);

    const handleCreate = async () => {
        if (!name.trim()) return;
        setCreating(true);
        try {
            const agent = await createAgent({
                name, provider, model, systemPrompt, toolsMode,
                routingMode,
                mcpServers: selectedMcpServers,
            });
            setSelectedAgentId(agent.id);
            setShowCreate(false);
            setName('');
            setSystemPrompt('');
            setToolsMode('restricted');
            setRoutingMode('single');
            setSelectedMcpServers([]);
        } catch { /* error handled by store */ }
        setCreating(false);
    };

    const handleDelete = async (id: string) => {
        try {
            await deleteAgent(id);
            if (selectedAgentId === id) {
                setSelectedAgentId(agents.find(a => a.id !== id)?.id);
            }
        } catch { /* error handled by store */ }
    };

    return (
        <div className="animate-fade-in h-full flex flex-col">
            <div className="page-header">
                <div>
                    <h1 className="page-title">Agents</h1>
                    <p className="page-description">Create and configure AI agents</p>
                </div>
                <div className="flex items-center gap-2">
                    <button className="btn btn-secondary" onClick={fetchAgents} disabled={agentsLoading}>
                        <RefreshCw className={`w-4 h-4 ${agentsLoading ? 'animate-spin' : ''}`} />
                        Refresh
                    </button>
                    <button className="btn btn-primary" onClick={() => setShowCreate(true)}>
                        <Plus className="w-4 h-4" />
                        New Agent
                    </button>
                </div>
            </div>

            {error && (
                <div className="mt-2 p-3 rounded-lg bg-red-500/10 border border-red-500/30 text-red-400 text-sm">
                    {error}
                </div>
            )}

            <div className="flex-1 flex gap-4 mt-4 overflow-hidden">
                {/* Agent List */}
                <div className="w-80 panel flex flex-col">
                    <div className="panel-header">
                        <span className="panel-title">Agents</span>
                        <span className="text-xs text-[var(--text-muted)]">{agents.length} total</span>
                    </div>
                    <div className="flex-1 overflow-y-auto p-2 space-y-2">
                        {agentsLoading && agents.length === 0 && (
                            <div className="flex items-center justify-center p-8 text-[var(--text-muted)]">
                                <Loader2 className="w-5 h-5 animate-spin mr-2" /> Loading...
                            </div>
                        )}
                        {agents.map((agent) => (
                            <div
                                key={agent.id}
                                className={`p-3 rounded-lg cursor-pointer transition-all group ${
                                    selectedAgentId === agent.id
                                        ? 'bg-[var(--accent-glow)] border border-[var(--accent-primary)]'
                                        : 'bg-[var(--bg-tertiary)] hover:bg-[var(--bg-hover)]'
                                }`}
                                onClick={() => setSelectedAgentId(agent.id)}
                            >
                                <div className="flex items-center gap-3">
                                    <div className="w-10 h-10 rounded-full bg-[var(--bg-hover)] flex items-center justify-center">
                                        <Bot className="w-5 h-5 text-[var(--accent-primary)]" />
                                    </div>
                                    <div className="flex-1 min-w-0">
                                        <div className="font-medium text-sm truncate">{agent.name}</div>
                                        <div className="text-xs text-[var(--text-muted)]">{agent.provider} / {agent.model}</div>
                                    </div>
                                    <button
                                        className="opacity-0 group-hover:opacity-100 p-1 hover:text-red-400 transition-all"
                                        onClick={(e) => { e.stopPropagation(); handleDelete(agent.id); }}
                                        title="Delete agent"
                                    >
                                        <Trash2 className="w-4 h-4" />
                                    </button>
                                </div>
                            </div>
                        ))}
                        {!agentsLoading && agents.length === 0 && (
                            <div className="text-center text-[var(--text-muted)] p-8 text-sm">
                                No agents yet. Create one to get started.
                            </div>
                        )}
                    </div>
                </div>

                {/* Agent Detail / Create Form */}
                <div className="flex-1 panel flex flex-col">
                    {showCreate ? (
                        <>
                            <div className="panel-header">
                                <span className="panel-title">New Agent</span>
                                <button className="btn btn-secondary btn-sm" onClick={() => setShowCreate(false)}>Cancel</button>
                            </div>
                            <div className="panel-content space-y-4">
                                <div>
                                    <label className="block text-xs font-medium text-[var(--text-muted)] mb-1">Name</label>
                                    <input className="input w-full" value={name} onChange={e => setName(e.target.value)} placeholder="e.g. Code Reviewer" />
                                </div>
                                <div className="grid grid-cols-2 gap-4">
                                    <div>
                                        <label className="block text-xs font-medium text-[var(--text-muted)] mb-1">Provider</label>
                                        <select className="input w-full" value={provider} onChange={e => handleProviderChange(e.target.value)}>
                                            <option value="anthropic">Anthropic</option>
                                            <option value="google">Google</option>
                                            <option value="azure_openai">Azure OpenAI</option>
                                            <option value="local">Local / OpenAI-Compatible</option>
                                        </select>
                                    </div>
                                    <div>
                                        <label className="block text-xs font-medium text-[var(--text-muted)] mb-1">Model</label>
                                        {customModel || availableModels.length === 0 ? (
                                            <div className="flex gap-2">
                                                <input
                                                    className="input flex-1"
                                                    value={model}
                                                    onChange={e => setModel(e.target.value)}
                                                    placeholder="e.g. my-custom-model"
                                                />
                                                {availableModels.length > 0 && (
                                                    <button
                                                        type="button"
                                                        className="btn btn-secondary btn-sm text-xs"
                                                        onClick={() => { setCustomModel(false); setModel(availableModels[0]); }}
                                                    >
                                                        List
                                                    </button>
                                                )}
                                            </div>
                                        ) : (
                                            <select
                                                className="input w-full"
                                                value={model}
                                                onChange={e => {
                                                    if (e.target.value === '__custom__') {
                                                        setCustomModel(true);
                                                        setModel('');
                                                    } else {
                                                        setModel(e.target.value);
                                                    }
                                                }}
                                            >
                                                {availableModels.map(m => (
                                                    <option key={m} value={m}>{m}</option>
                                                ))}
                                                <option value="__custom__">Custom...</option>
                                            </select>
                                        )}
                                    </div>
                                </div>
                                <div>
                                    <label className="block text-xs font-medium text-[var(--text-muted)] mb-1">System Prompt</label>
                                    <textarea className="input w-full h-32 resize-none" value={systemPrompt} onChange={e => setSystemPrompt(e.target.value)} placeholder="You are a helpful AI assistant..." />
                                </div>
                                <div>
                                    <label className="block text-xs font-medium text-[var(--text-muted)] mb-1">Tools Mode</label>
                                    <select className="input w-full" value={toolsMode} onChange={e => setToolsMode(e.target.value as ToolsMode)}>
                                        <option value="sandboxed">Sandboxed — no tool access</option>
                                        <option value="restricted">Restricted — approved tools only (default)</option>
                                        <option value="full">Full — all available tools</option>
                                    </select>
                                </div>
                                <div>
                                    <label className="block text-xs font-medium text-[var(--text-muted)] mb-1">Routing Mode</label>
                                    <div className="grid grid-cols-3 gap-2">
                                        {([
                                            { value: 'single' as const, label: 'Single', desc: 'One model for everything' },
                                            { value: 'hybrid_auto' as const, label: 'Auto', desc: 'Smart model routing' },
                                            { value: 'hybrid_manual' as const, label: 'Manual', desc: 'Custom rules' },
                                        ]).map(opt => (
                                            <button
                                                key={opt.value}
                                                type="button"
                                                className={`p-2 rounded-lg border text-left transition-all ${
                                                    routingMode === opt.value
                                                        ? 'border-[var(--accent-primary)] bg-[var(--accent-glow)]'
                                                        : 'border-[var(--border-primary)] bg-[var(--bg-tertiary)] hover:bg-[var(--bg-hover)]'
                                                }`}
                                                onClick={() => setRoutingMode(opt.value)}
                                            >
                                                <div className="text-xs font-medium">{opt.label}</div>
                                                <div className="text-[10px] text-[var(--text-muted)]">{opt.desc}</div>
                                            </button>
                                        ))}
                                    </div>
                                    {routingMode !== 'single' && (
                                        <div className="mt-2 p-2 rounded bg-[var(--bg-tertiary)] text-xs text-[var(--text-muted)]">
                                            <Zap className="w-3 h-3 inline mr-1 text-amber-400" />
                                            {routingMode === 'hybrid_auto'
                                                ? 'AI Studio will automatically pick the best model for each message based on task type, budget, and available providers.'
                                                : 'Define custom rules in agent settings after creation. Rules match conditions (vision, code, budget) to specific models.'}
                                        </div>
                                    )}
                                </div>
                                {mcpServers.length > 0 && (
                                    <div>
                                        <label className="block text-xs font-medium text-[var(--text-muted)] mb-1">MCP Servers</label>
                                        <div className="space-y-2">
                                            {mcpServers.filter(s => s.enabled).map(server => (
                                                <label key={server.id} className="flex items-center gap-2 text-sm cursor-pointer">
                                                    <input
                                                        type="checkbox"
                                                        checked={selectedMcpServers.includes(server.id)}
                                                        onChange={(e) => {
                                                            if (e.target.checked) {
                                                                setSelectedMcpServers(prev => [...prev, server.id]);
                                                            } else {
                                                                setSelectedMcpServers(prev => prev.filter(id => id !== server.id));
                                                            }
                                                        }}
                                                        className="rounded border-[var(--border-primary)]"
                                                    />
                                                    <Server className="w-3.5 h-3.5 text-[var(--text-muted)]" />
                                                    {server.name}
                                                </label>
                                            ))}
                                        </div>
                                    </div>
                                )}
                                <button className="btn btn-primary w-full" onClick={handleCreate} disabled={creating || !name.trim()}>
                                    {creating ? <><Loader2 className="w-4 h-4 animate-spin" /> Creating...</> : 'Create Agent'}
                                </button>
                            </div>
                        </>
                    ) : selectedAgent ? (
                        <>
                            <div className="panel-header">
                                <div className="flex items-center gap-3">
                                    <Bot className="w-5 h-5 text-[var(--accent-primary)]" />
                                    <span className="panel-title">{selectedAgent.name}</span>
                                </div>
                            </div>
                            <div className="panel-content space-y-6">
                                <div>
                                    <label className="block text-xs font-medium text-[var(--text-muted)] mb-1">Model</label>
                                    <div className="text-sm">{selectedAgent.provider} / {selectedAgent.model}</div>
                                </div>
                                {selectedAgent.description && (
                                    <div>
                                        <label className="block text-xs font-medium text-[var(--text-muted)] mb-1">Description</label>
                                        <div className="text-sm">{selectedAgent.description}</div>
                                    </div>
                                )}
                                <div>
                                    <label className="block text-xs font-medium text-[var(--text-muted)] mb-1">System Prompt</label>
                                    <div className="text-sm bg-[var(--bg-tertiary)] p-3 rounded-lg whitespace-pre-wrap">
                                        {selectedAgent.systemPrompt || '(none)'}
                                    </div>
                                </div>
                                <div>
                                    <label className="block text-xs font-medium text-[var(--text-muted)] mb-1">Tools Mode</label>
                                    <span className={`inline-flex items-center gap-1.5 px-2.5 py-1 text-xs rounded-full font-medium ${
                                        selectedAgent.toolsMode === 'full' ? 'bg-green-500/15 text-green-400' :
                                        selectedAgent.toolsMode === 'restricted' ? 'bg-yellow-500/15 text-yellow-400' :
                                        'bg-red-500/15 text-red-400'
                                    }`}>
                                        <Shield className="w-3 h-3" />
                                        {selectedAgent.toolsMode}
                                    </span>
                                </div>
                                <div>
                                    <label className="block text-xs font-medium text-[var(--text-muted)] mb-1">Routing Mode</label>
                                    <span className={`inline-flex items-center gap-1.5 px-2.5 py-1 text-xs rounded-full font-medium ${
                                        selectedAgent.routingMode === 'hybrid_auto' ? 'bg-amber-500/15 text-amber-400' :
                                        selectedAgent.routingMode === 'hybrid_manual' ? 'bg-purple-500/15 text-purple-400' :
                                        'bg-[var(--bg-tertiary)] text-[var(--text-muted)]'
                                    }`}>
                                        <Zap className="w-3 h-3" />
                                        {selectedAgent.routingMode === 'hybrid_auto' ? 'Hybrid Auto' :
                                         selectedAgent.routingMode === 'hybrid_manual' ? 'Hybrid Manual' : 'Single Model'}
                                    </span>
                                </div>
                                {selectedAgent.mcpServers.length > 0 && (
                                    <div>
                                        <label className="block text-xs font-medium text-[var(--text-muted)] mb-1">MCP Servers</label>
                                        <div className="flex flex-wrap gap-2">
                                            {selectedAgent.mcpServers.map((serverId: string) => {
                                                const server = mcpServers.find(s => s.id === serverId);
                                                return (
                                                    <span key={serverId} className="inline-flex items-center gap-1.5 px-2 py-1 text-xs rounded bg-[var(--bg-tertiary)] font-mono">
                                                        <Server className="w-3 h-3" />
                                                        {server?.name || serverId}
                                                    </span>
                                                );
                                            })}
                                        </div>
                                    </div>
                                )}
                                {selectedAgent.tools.length > 0 && (
                                    <div>
                                        <label className="block text-xs font-medium text-[var(--text-muted)] mb-1">Tools (legacy)</label>
                                        <div className="flex flex-wrap gap-2">
                                            {selectedAgent.tools.map((tool: string) => (
                                                <span key={tool} className="px-2 py-1 text-xs rounded bg-[var(--bg-tertiary)] font-mono">
                                                    {tool}
                                                </span>
                                            ))}
                                        </div>
                                    </div>
                                )}
                                <div className="flex gap-6 text-sm">
                                    <div>
                                        <span className="text-[var(--text-muted)]">Temperature: </span>
                                        <span className="font-medium">{selectedAgent.temperature}</span>
                                    </div>
                                    <div>
                                        <span className="text-[var(--text-muted)]">Max Tokens: </span>
                                        <span className="font-medium">{selectedAgent.maxTokens.toLocaleString()}</span>
                                    </div>
                                </div>
                                <div className="flex gap-4 text-xs text-[var(--text-muted)]">
                                    <span>Created: {new Date(selectedAgent.createdAt).toLocaleDateString()}</span>
                                    <span>Updated: {new Date(selectedAgent.updatedAt).toLocaleDateString()}</span>
                                </div>
                            </div>
                        </>
                    ) : (
                        <div className="flex-1 flex items-center justify-center text-[var(--text-muted)]">
                            Select an agent to view details
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
}
