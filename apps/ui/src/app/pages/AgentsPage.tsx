import { useState } from 'react';
import { Plus, RefreshCw, Send, Bot } from 'lucide-react';
import { useAppStore } from '../../state/store';

/**
 * Agents Page
 * 
 * Features:
 * - Agent list with status pills
 * - Mock chat timeline
 * - Agent management
 */
export function AgentsPage() {
    const { agents } = useAppStore();
    const [selectedAgentId, setSelectedAgentId] = useState(agents[0]?.id);
    const [message, setMessage] = useState('');

    const selectedAgent = agents.find(a => a.id === selectedAgentId);

    // Mock chat messages
    const mockMessages = [
        { role: 'user', content: 'Analyze the dataset and provide insights', timestamp: '10:30 AM' },
        { role: 'agent', content: 'I\'ve analyzed the dataset. Here are the key findings:\n\n1. The data contains 10,000 samples\n2. Primary features show strong correlation\n3. Recommended preprocessing: normalization', timestamp: '10:31 AM' },
        { role: 'user', content: 'What\'s the recommended model architecture?', timestamp: '10:32 AM' },
        { role: 'agent', content: 'Based on the data characteristics, I recommend:\n\n• A transformer-based architecture\n• 6 attention layers\n• Embedding dimension: 512\n\nThis should achieve ~94% accuracy on the validation set.', timestamp: '10:33 AM' },
    ];

    const statusColors = {
        running: 'status-success',
        idle: 'status-info',
        error: 'status-error',
        offline: 'status-warning',
    };

    return (
        <div className="animate-fade-in h-full flex flex-col">
            {/* Page Header */}
            <div className="page-header">
                <div>
                    <h1 className="page-title">Agents</h1>
                    <p className="page-description">Manage and interact with AI agents</p>
                </div>
                <div className="flex items-center gap-2">
                    <button className="btn btn-secondary">
                        <RefreshCw className="w-4 h-4" />
                        Refresh
                    </button>
                    <button className="btn btn-primary">
                        <Plus className="w-4 h-4" />
                        New Agent
                    </button>
                </div>
            </div>

            {/* Main Content */}
            <div className="flex-1 flex gap-4 mt-4 overflow-hidden">
                {/* Agent List */}
                <div className="w-80 panel flex flex-col">
                    <div className="panel-header">
                        <span className="panel-title">Active Agents</span>
                        <span className="text-xs text-[var(--text-muted)]">{agents.length} total</span>
                    </div>
                    <div className="flex-1 overflow-y-auto p-2 space-y-2">
                        {agents.map((agent) => (
                            <div
                                key={agent.id}
                                className={`p-3 rounded-lg cursor-pointer transition-all ${selectedAgentId === agent.id
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
                                        <div className="text-xs text-[var(--text-muted)]">{agent.model}</div>
                                    </div>
                                    <span className={`status-pill ${statusColors[agent.status]}`}>
                                        <span className="status-dot" />
                                        {agent.status}
                                    </span>
                                </div>
                            </div>
                        ))}
                    </div>
                </div>

                {/* Chat Timeline */}
                <div className="flex-1 panel flex flex-col">
                    <div className="panel-header">
                        <div className="flex items-center gap-3">
                            <Bot className="w-5 h-5 text-[var(--accent-primary)]" />
                            <span className="panel-title">{selectedAgent?.name || 'Select an agent'}</span>
                        </div>
                        {selectedAgent && (
                            <span className={`status-pill ${statusColors[selectedAgent.status]}`}>
                                <span className="status-dot" />
                                {selectedAgent.status}
                            </span>
                        )}
                    </div>

                    {/* Messages */}
                    <div className="flex-1 overflow-y-auto p-4 space-y-4">
                        {mockMessages.map((msg, i) => (
                            <div
                                key={i}
                                className={`flex ${msg.role === 'user' ? 'justify-end' : 'justify-start'}`}
                            >
                                <div
                                    className={`max-w-[80%] p-3 rounded-lg ${msg.role === 'user'
                                            ? 'bg-[var(--accent-primary)] text-white'
                                            : 'bg-[var(--bg-tertiary)]'
                                        }`}
                                >
                                    <div className="text-sm whitespace-pre-wrap">{msg.content}</div>
                                    <div className={`text-xs mt-2 ${msg.role === 'user' ? 'text-white/70' : 'text-[var(--text-muted)]'
                                        }`}>
                                        {msg.timestamp}
                                    </div>
                                </div>
                            </div>
                        ))}
                    </div>

                    {/* Input */}
                    <div className="p-4 border-t border-[var(--border-subtle)]">
                        <div className="flex gap-2">
                            <input
                                type="text"
                                className="input flex-1"
                                placeholder="Send a message..."
                                value={message}
                                onChange={(e) => setMessage(e.target.value)}
                            />
                            <button className="btn btn-primary">
                                <Send className="w-4 h-4" />
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}
