import { Plus, MessageSquare, Send, Loader2, Trash2, Search, GitBranch } from 'lucide-react';
import { useState, useEffect, useRef } from 'react';
import { useAppStore } from '../../state/store';

/**
 * Sessions Page
 *
 * Interactive chat with AI agents. Real messages via Tauri IPC.
 */
export function SessionsPage() {
    const {
        sessions, sessionsLoading, fetchSessions, createSession, deleteSession, branchSession,
        agents, fetchAgents,
        messages, messagesLoading, fetchMessages, sendMessage, sending,
        error, openInspector,
    } = useAppStore();
    const [selectedSessionId, setSelectedSessionId] = useState<string | undefined>();
    const [input, setInput] = useState('');
    const [showNewSession, setShowNewSession] = useState(false);
    const [selectedAgentId, setSelectedAgentId] = useState('');
    const chatEndRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        fetchSessions();
        fetchAgents();
    }, [fetchSessions, fetchAgents]);

    useEffect(() => {
        if (sessions.length > 0 && !selectedSessionId) {
            setSelectedSessionId(sessions[0].id);
        }
    }, [sessions, selectedSessionId]);

    // Load messages when session changes
    useEffect(() => {
        if (selectedSessionId) {
            fetchMessages(selectedSessionId);
        }
    }, [selectedSessionId, fetchMessages]);

    // Auto-scroll to bottom on new messages
    useEffect(() => {
        chatEndRef.current?.scrollIntoView({ behavior: 'smooth' });
    }, [messages]);

    const selectedSession = sessions.find(s => s.id === selectedSessionId);

    const handleSend = async () => {
        if (!input.trim() || !selectedSessionId || sending) return;
        const content = input;
        setInput('');
        try {
            await sendMessage(selectedSessionId, content);
        } catch { /* error handled by store */ }
    };

    const handleKeyDown = (e: React.KeyboardEvent) => {
        if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
            handleSend();
        }
    };

    const handleCreateSession = async () => {
        if (!selectedAgentId) return;
        try {
            const session = await createSession(selectedAgentId);
            setSelectedSessionId(session.id);
            setShowNewSession(false);
        } catch { /* error handled by store */ }
    };

    const handleDeleteSession = async (id: string) => {
        try {
            await deleteSession(id);
            if (selectedSessionId === id) {
                setSelectedSessionId(sessions.find(s => s.id !== id)?.id);
            }
        } catch { /* error handled by store */ }
    };

    const handleBranch = async (msgSeq: number) => {
        if (!selectedSessionId) return;
        try {
            const branch = await branchSession(selectedSessionId, msgSeq);
            setSelectedSessionId(branch.id);
        } catch { /* error handled by store */ }
    };

    return (
        <div className="animate-fade-in h-full flex flex-col">
            <div className="page-header">
                <div>
                    <h1 className="page-title">Sessions</h1>
                    <p className="page-description">Interactive conversations with AI agents</p>
                </div>
                <button className="btn btn-primary" onClick={() => setShowNewSession(true)}>
                    <Plus className="w-4 h-4" />
                    New Session
                </button>
            </div>

            {error && (
                <div className="mt-2 p-3 rounded-lg bg-red-500/10 border border-red-500/30 text-red-400 text-sm">
                    {error}
                </div>
            )}

            {/* New Session Dialog */}
            {showNewSession && (
                <div className="mt-2 p-4 panel space-y-3">
                    <div className="text-sm font-medium">Start a new session</div>
                    <div className="flex gap-3">
                        <select
                            className="input flex-1"
                            value={selectedAgentId}
                            onChange={e => setSelectedAgentId(e.target.value)}
                        >
                            <option value="">Select an agent...</option>
                            {agents.map(a => (
                                <option key={a.id} value={a.id}>{a.name} ({a.provider}/{a.model})</option>
                            ))}
                        </select>
                        <button className="btn btn-primary" onClick={handleCreateSession} disabled={!selectedAgentId}>
                            Create
                        </button>
                        <button className="btn btn-secondary" onClick={() => setShowNewSession(false)}>
                            Cancel
                        </button>
                    </div>
                </div>
            )}

            <div className="flex-1 flex gap-4 mt-4 overflow-hidden">
                {/* Session List */}
                <div className="w-80 panel flex flex-col">
                    <div className="panel-header">
                        <span className="panel-title">Recent Sessions</span>
                        <span className="text-xs text-[var(--text-muted)]">{sessions.length}</span>
                    </div>
                    <div className="flex-1 overflow-y-auto p-2 space-y-2">
                        {sessionsLoading && sessions.length === 0 && (
                            <div className="flex items-center justify-center p-8 text-[var(--text-muted)]">
                                <Loader2 className="w-5 h-5 animate-spin mr-2" /> Loading...
                            </div>
                        )}
                        {sessions.map((session) => (
                            <div
                                key={session.id}
                                className={`p-3 rounded-lg cursor-pointer transition-all group ${
                                    selectedSessionId === session.id
                                        ? 'bg-[var(--accent-glow)] border border-[var(--accent-primary)]'
                                        : 'bg-[var(--bg-tertiary)] hover:bg-[var(--bg-hover)]'
                                }`}
                                onClick={() => setSelectedSessionId(session.id)}
                            >
                                <div className="flex items-center justify-between">
                                    <div className="font-medium text-sm truncate flex-1 flex items-center gap-1">
                                        {session.parentSessionId && (
                                            <span title="Branched session">
                                                <GitBranch className="w-3 h-3 text-[var(--text-muted)] flex-shrink-0" />
                                            </span>
                                        )}
                                        {session.title}
                                    </div>
                                    <div className="opacity-0 group-hover:opacity-100 flex items-center gap-1 transition-all">
                                        <button
                                            className="p-1 hover:text-[var(--accent-primary)] transition-colors"
                                            onClick={(e) => { e.stopPropagation(); openInspector(session.id); }}
                                            title="Inspect session"
                                        >
                                            <Search className="w-3 h-3" />
                                        </button>
                                        <button
                                            className="p-1 hover:text-red-400 transition-colors"
                                            onClick={(e) => { e.stopPropagation(); handleDeleteSession(session.id); }}
                                            title="Delete session"
                                        >
                                            <Trash2 className="w-3 h-3" />
                                        </button>
                                    </div>
                                </div>
                                <div className="text-xs text-[var(--text-muted)] mt-1">
                                    {session.agentName} &middot; {session.messageCount} messages
                                </div>
                            </div>
                        ))}
                        {!sessionsLoading && sessions.length === 0 && (
                            <div className="text-center text-[var(--text-muted)] p-8 text-sm">
                                No sessions yet. Start a new one.
                            </div>
                        )}
                    </div>
                </div>

                {/* Chat Area */}
                <div className="flex-1 panel flex flex-col">
                    <div className="panel-header">
                        <div className="flex items-center gap-3">
                            <MessageSquare className="w-5 h-5 text-[var(--accent-primary)]" />
                            <span className="panel-title">{selectedSession?.title || 'Select a session'}</span>
                        </div>
                        {selectedSession?.agentName && (
                            <span className="text-xs text-[var(--text-muted)]">
                                {selectedSession.agentName} &middot; {selectedSession.agentModel}
                            </span>
                        )}
                    </div>

                    <div className="flex-1 overflow-y-auto p-4 space-y-4">
                        {messagesLoading && (
                            <div className="flex items-center justify-center p-8 text-[var(--text-muted)]">
                                <Loader2 className="w-5 h-5 animate-spin mr-2" /> Loading messages...
                            </div>
                        )}
                        {!messagesLoading && messages.length === 0 && selectedSession && (
                            <div className="text-center text-[var(--text-muted)] p-8 text-sm">
                                Send a message to start the conversation.
                            </div>
                        )}
                        {messages.map((msg) => (
                            <div key={msg.id} className={`group/msg flex ${msg.role === 'user' ? 'justify-end' : 'justify-start'}`}>
                                <div className={`max-w-[80%] p-3 rounded-lg relative ${
                                    msg.role === 'user'
                                        ? 'bg-[var(--accent-primary)] text-white'
                                        : 'bg-[var(--bg-tertiary)]'
                                }`}>
                                    <div className="text-sm whitespace-pre-wrap">{msg.content}</div>
                                    <div className={`text-xs mt-2 flex items-center gap-2 ${
                                        msg.role === 'user' ? 'text-white/70' : 'text-[var(--text-muted)]'
                                    }`}>
                                        <span>{new Date(msg.createdAt).toLocaleTimeString()}</span>
                                        {msg.model && <span>&middot; {msg.model}</span>}
                                        {msg.durationMs != null && <span>&middot; {(msg.durationMs / 1000).toFixed(1)}s</span>}
                                        <button
                                            className="opacity-0 group-hover/msg:opacity-100 ml-auto p-0.5 rounded hover:bg-white/10 transition-all"
                                            onClick={() => handleBranch(msg.seq)}
                                            title="Branch from here"
                                        >
                                            <GitBranch className="w-3 h-3" />
                                        </button>
                                    </div>
                                </div>
                            </div>
                        ))}
                        {sending && (
                            <div className="flex justify-start">
                                <div className="p-3 rounded-lg bg-[var(--bg-tertiary)]">
                                    <Loader2 className="w-4 h-4 animate-spin text-[var(--accent-primary)]" />
                                </div>
                            </div>
                        )}
                        <div ref={chatEndRef} />
                    </div>

                    <div className="p-4 border-t border-[var(--border-subtle)]">
                        <div className="flex gap-2">
                            <input
                                type="text"
                                className="input flex-1"
                                placeholder={selectedSession ? 'Send a message... (Ctrl+Enter)' : 'Select a session first'}
                                value={input}
                                onChange={(e) => setInput(e.target.value)}
                                onKeyDown={handleKeyDown}
                                disabled={!selectedSession || sending}
                            />
                            <button
                                className="btn btn-primary"
                                onClick={handleSend}
                                disabled={!selectedSession || !input.trim() || sending}
                            >
                                {sending ? <Loader2 className="w-4 h-4 animate-spin" /> : <Send className="w-4 h-4" />}
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}
