import { useState } from 'react';
import { Cloud, Bot, MessageSquare, ChevronRight, Check, Loader2, Eye, EyeOff, Sparkles } from 'lucide-react';
import { useAppStore } from '../../state/store';
import { fetchApi } from '../../services/api';

interface ProviderOption {
    id: string;
    name: string;
    icon: string;
    placeholder: string;
    docsUrl: string;
    defaultModel: string;
}

const PROVIDERS: ProviderOption[] = [
    { id: 'google', name: 'Google AI (Gemini)', icon: '✨', placeholder: 'AIza...', docsUrl: 'https://aistudio.google.com/apikey', defaultModel: 'gemini-2.0-flash' },
    { id: 'anthropic', name: 'Anthropic (Claude)', icon: '🔮', placeholder: 'sk-ant-...', docsUrl: 'https://console.anthropic.com', defaultModel: 'claude-sonnet-4-20250514' },
    { id: 'azure_openai', name: 'Azure OpenAI', icon: '☁️', placeholder: 'endpoint|key|deployment', docsUrl: 'https://portal.azure.com', defaultModel: 'gpt-4o' },
    { id: 'local', name: 'Local (Ollama)', icon: '🖥️', placeholder: 'http://localhost:11434', docsUrl: 'https://ollama.ai', defaultModel: 'llama3.2' },
];

interface AgentTemplate {
    name: string;
    description: string;
    systemPrompt: string;
    icon: string;
}

const TEMPLATES: AgentTemplate[] = [
    {
        name: 'Code Helper',
        description: 'Writes, explains, and debugs code',
        systemPrompt: 'You are an expert software engineer. Help the user write, debug, and explain code. Be concise and provide working code examples.',
        icon: '🤖',
    },
    {
        name: 'Code Reviewer',
        description: 'Reviews code for bugs and improvements',
        systemPrompt: 'You are a senior code reviewer. Analyze code for bugs, security issues, performance problems, and style. Be constructive and specific.',
        icon: '🔍',
    },
    {
        name: 'Data Analyst',
        description: 'Analyzes data and generates insights',
        systemPrompt: 'You are a data analyst. Help the user analyze datasets, write SQL queries, create visualizations, and derive insights. Be precise with numbers.',
        icon: '📊',
    },
    {
        name: 'General Assistant',
        description: 'Start with a blank slate',
        systemPrompt: 'You are a helpful AI assistant.',
        icon: '✨',
    },
];

interface WelcomePageProps {
    onComplete: () => void;
}

export function WelcomePage({ onComplete }: WelcomePageProps) {
    const { saveSetting, createAgent, addToast } = useAppStore();
    const [step, setStep] = useState(1);

    // Step 1 state
    const [selectedProvider, setSelectedProvider] = useState<string | null>(null);
    const [apiKey, setApiKey] = useState('');
    const [showKey, setShowKey] = useState(false);
    const [testing, setTesting] = useState(false);
    const [testResult, setTestResult] = useState<'idle' | 'success' | 'error'>('idle');

    // Step 2 state
    const [creating, setCreating] = useState(false);

    // Resolved provider + model for agent creation
    const [configuredProvider, setConfiguredProvider] = useState('');
    const [configuredModel, setConfiguredModel] = useState('');

    const provider = PROVIDERS.find(p => p.id === selectedProvider);

    const handleTestAndSave = async () => {
        if (!selectedProvider || !provider) return;

        setTesting(true);
        setTestResult('idle');

        try {
            // For local provider, just save the base_url
            if (selectedProvider === 'local') {
                const baseUrl = apiKey || 'http://localhost:11434';
                await saveSetting(`provider.local.base_url`, baseUrl);
                setConfiguredProvider('local');
                setConfiguredModel(provider.defaultModel);
                setTestResult('success');
                addToast('Local provider configured', 'success');
                setTesting(false);
                setTimeout(() => setStep(2), 800);
                return;
            }

            // Save API key first
            await saveSetting(`provider.${selectedProvider}.api_key`, apiKey);

            // Test the connection
            const result = await fetchApi<{ success: boolean; message: string }>('/providers/test', {
                method: 'POST',
                body: JSON.stringify({
                    provider: selectedProvider,
                    api_key: apiKey || undefined,
                }),
            });

            if (result.success) {
                setTestResult('success');
                setConfiguredProvider(selectedProvider);
                setConfiguredModel(provider.defaultModel);
                addToast(`${provider.name}: connected`, 'success');
                setTimeout(() => setStep(2), 800);
            } else {
                setTestResult('error');
                addToast(`${provider.name}: ${result.message || 'connection failed'}`, 'error');
            }
        } catch (err) {
            setTestResult('error');
            addToast(`Connection test failed: ${err instanceof Error ? err.message : 'unknown error'}`, 'error');
        }

        setTesting(false);
    };

    const handleSkipProvider = () => {
        // Default to local provider
        setConfiguredProvider('local');
        setConfiguredModel('llama3.2');
        setStep(2);
    };

    const handleCreateAgent = async (template: AgentTemplate) => {
        setCreating(true);
        try {
            await createAgent({
                name: template.name,
                provider: configuredProvider || 'local',
                model: configuredModel || 'llama3.2',
                systemPrompt: template.systemPrompt,
            });
            await saveSetting('onboarding.completed', 'true');
            setStep(3);
        } catch {
            // Error handled by store
        }
        setCreating(false);
    };

    const handleFinish = () => {
        onComplete();
    };

    return (
        <div className="animate-fade-in h-full flex items-center justify-center">
            <div className="w-full max-w-2xl">
                {/* Progress indicator */}
                <div className="flex items-center justify-center gap-2 mb-8">
                    {[1, 2, 3].map(s => (
                        <div key={s} className="flex items-center gap-2">
                            <div className={`w-8 h-8 rounded-full flex items-center justify-center text-sm font-medium transition-colors ${
                                s < step ? 'bg-green-500/20 text-green-400' :
                                s === step ? 'bg-[var(--accent-primary)] text-white' :
                                'bg-[var(--bg-tertiary)] text-[var(--text-muted)]'
                            }`}>
                                {s < step ? <Check className="w-4 h-4" /> : s}
                            </div>
                            {s < 3 && <ChevronRight className="w-4 h-4 text-[var(--text-muted)]" />}
                        </div>
                    ))}
                </div>

                {/* Step 1: Connect a provider */}
                {step === 1 && (
                    <div className="panel">
                        <div className="panel-header">
                            <div className="flex items-center gap-2">
                                <Cloud className="w-5 h-5 text-[var(--accent-primary)]" />
                                <span className="panel-title">Connect a Model Provider</span>
                            </div>
                            <button className="btn btn-secondary btn-sm" onClick={handleSkipProvider}>
                                Skip
                            </button>
                        </div>
                        <div className="panel-content space-y-3">
                            <p className="text-sm text-[var(--text-secondary)]">
                                Choose a provider and enter your API key. You can always change this later in Settings.
                            </p>

                            <div className="grid grid-cols-2 gap-3">
                                {PROVIDERS.map(p => (
                                    <button
                                        key={p.id}
                                        className={`p-3 rounded-lg text-left transition-all border ${
                                            selectedProvider === p.id
                                                ? 'bg-[var(--accent-glow)] border-[var(--accent-primary)]'
                                                : 'bg-[var(--bg-tertiary)] border-[var(--border-subtle)] hover:bg-[var(--bg-hover)]'
                                        }`}
                                        onClick={() => { setSelectedProvider(p.id); setApiKey(''); setTestResult('idle'); }}
                                    >
                                        <div className="flex items-center gap-2">
                                            <span className="text-lg">{p.icon}</span>
                                            <span className="font-medium text-sm">{p.name}</span>
                                        </div>
                                    </button>
                                ))}
                            </div>

                            {selectedProvider && provider && (
                                <div className="space-y-3 pt-2">
                                    <div>
                                        <label className="block text-xs font-medium text-[var(--text-muted)] mb-1">
                                            {selectedProvider === 'local' ? 'Base URL' : 'API Key'}
                                        </label>
                                        <div className="relative">
                                            <input
                                                type={showKey ? 'text' : 'password'}
                                                className="input w-full pr-10"
                                                placeholder={provider.placeholder}
                                                value={apiKey}
                                                onChange={e => { setApiKey(e.target.value); setTestResult('idle'); }}
                                                onKeyDown={e => { if (e.key === 'Enter') handleTestAndSave(); }}
                                            />
                                            <button
                                                type="button"
                                                className="absolute right-2 top-1/2 -translate-y-1/2 p-1 text-[var(--text-muted)] hover:text-[var(--text-primary)]"
                                                onClick={() => setShowKey(!showKey)}
                                            >
                                                {showKey ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                                            </button>
                                        </div>
                                    </div>
                                    <div className="flex items-center justify-between">
                                        <a
                                            href={provider.docsUrl}
                                            target="_blank"
                                            rel="noopener noreferrer"
                                            className="text-xs text-[var(--accent-primary)] hover:underline"
                                        >
                                            Get an API key
                                        </a>
                                        <button
                                            className="btn btn-primary"
                                            onClick={handleTestAndSave}
                                            disabled={testing || (!apiKey.trim() && selectedProvider !== 'local')}
                                        >
                                            {testing ? (
                                                <><Loader2 className="w-4 h-4 animate-spin" /> Testing...</>
                                            ) : testResult === 'success' ? (
                                                <><Check className="w-4 h-4" /> Connected</>
                                            ) : (
                                                <>Connect &amp; Continue</>
                                            )}
                                        </button>
                                    </div>
                                </div>
                            )}
                        </div>
                    </div>
                )}

                {/* Step 2: Pick a starter agent */}
                {step === 2 && (
                    <div className="panel">
                        <div className="panel-header">
                            <div className="flex items-center gap-2">
                                <Bot className="w-5 h-5 text-[var(--accent-primary)]" />
                                <span className="panel-title">Pick a Starter Agent</span>
                            </div>
                        </div>
                        <div className="panel-content space-y-3">
                            <p className="text-sm text-[var(--text-secondary)]">
                                Choose a template to create your first agent. You can customize it later.
                            </p>

                            <div className="grid grid-cols-2 gap-3">
                                {TEMPLATES.map(template => (
                                    <button
                                        key={template.name}
                                        className="p-4 rounded-lg bg-[var(--bg-tertiary)] border border-[var(--border-subtle)] hover:bg-[var(--bg-hover)] hover:border-[var(--accent-primary)] text-left transition-all"
                                        onClick={() => handleCreateAgent(template)}
                                        disabled={creating}
                                    >
                                        <div className="text-2xl mb-2">{template.icon}</div>
                                        <div className="font-medium text-sm">{template.name}</div>
                                        <div className="text-xs text-[var(--text-muted)] mt-1">{template.description}</div>
                                    </button>
                                ))}
                            </div>

                            {creating && (
                                <div className="flex items-center justify-center gap-2 text-sm text-[var(--text-muted)] pt-2">
                                    <Loader2 className="w-4 h-4 animate-spin" /> Creating agent...
                                </div>
                            )}

                            {configuredProvider && (
                                <div className="text-xs text-[var(--text-muted)] text-center pt-1">
                                    Using <span className="font-medium">{configuredProvider}</span> / <span className="font-mono">{configuredModel}</span>
                                </div>
                            )}
                        </div>
                    </div>
                )}

                {/* Step 3: Success */}
                {step === 3 && (
                    <div className="panel">
                        <div className="panel-content text-center py-8 space-y-4">
                            <div className="w-16 h-16 rounded-full bg-green-500/15 flex items-center justify-center mx-auto">
                                <Sparkles className="w-8 h-8 text-green-400" />
                            </div>
                            <div>
                                <h2 className="text-xl font-semibold">You're all set!</h2>
                                <p className="text-sm text-[var(--text-secondary)] mt-1">
                                    Your first agent is ready. Start a session to chat with it.
                                </p>
                            </div>
                            <button className="btn btn-primary" onClick={handleFinish}>
                                <MessageSquare className="w-4 h-4" />
                                Start Your First Session
                            </button>
                        </div>
                    </div>
                )}
            </div>
        </div>
    );
}
