import { Search, Download, Loader2 } from 'lucide-react';
import { useState, useEffect } from 'react';
import { useAppStore } from '../../state/store';

// Color mapping for event types
const eventColors: Record<string, string> = {
    'session.started': 'var(--accent-primary)',
    'session.ended': 'var(--accent-primary)',
    'message.user': 'var(--accent-secondary)',
    'message.assistant': 'var(--accent-secondary)',
    'llm.request.started': 'var(--status-info)',
    'llm.response.completed': 'var(--status-info)',
    'llm.response.error': 'var(--status-error)',
    'tool.requested': 'var(--status-warning)',
    'tool.approved': 'var(--status-success)',
    'tool.completed': 'var(--status-success)',
    'tool.rejected': 'var(--status-error)',
};

function formatTimestamp(ts: string): string {
    try {
        return new Date(ts).toLocaleTimeString();
    } catch {
        return ts;
    }
}

function summarizePayload(type: string, payload: Record<string, unknown>): string {
    if (type === 'message.user' || type === 'message.assistant') {
        const content = (payload.content as string) || '';
        return content.length > 80 ? content.slice(0, 80) + '...' : content;
    }
    if (type.startsWith('llm.')) {
        const model = (payload.model as string) || '';
        const tokens = (payload.input_tokens || payload.output_tokens) as number;
        return tokens ? `${model} - ${tokens} tokens` : model;
    }
    if (type.startsWith('tool.')) {
        return (payload.tool as string) || (payload.command as string) || type;
    }
    return Object.keys(payload).slice(0, 3).join(', ');
}

/**
 * Inspector Page (Flagship Feature)
 *
 * Chrome DevTools for AI agents. Event timeline, detail panel, stats bar.
 * Wired to real events from SQLite via Tauri IPC.
 */
export function InspectorPage() {
    const {
        sessions, fetchSessions,
        events, eventsLoading, fetchEvents,
        sessionStats, fetchSessionStats,
        error,
    } = useAppStore();

    const [selectedSessionId, setSelectedSessionId] = useState<string | undefined>();
    const [selectedEventId, setSelectedEventId] = useState<string | undefined>();
    const [filter, setFilter] = useState('');

    useEffect(() => {
        fetchSessions();
    }, [fetchSessions]);

    // Load events when session changes
    useEffect(() => {
        if (selectedSessionId) {
            fetchEvents(selectedSessionId);
            fetchSessionStats(selectedSessionId);
        }
    }, [selectedSessionId, fetchEvents, fetchSessionStats]);

    const selectedEvent = events.find(e => e.eventId === selectedEventId);

    const filteredEvents = filter
        ? events.filter(e =>
            e.type.includes(filter) ||
            JSON.stringify(e.payload).toLowerCase().includes(filter.toLowerCase())
        )
        : events;

    const handleExport = () => {
        const data = JSON.stringify({ events, stats: sessionStats }, null, 2);
        const blob = new Blob([data], { type: 'application/json' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `inspector-${selectedSessionId || 'events'}.json`;
        a.click();
        URL.revokeObjectURL(url);
    };

    return (
        <div className="animate-fade-in h-full flex flex-col">
            <div className="page-header">
                <div>
                    <h1 className="page-title">Inspector</h1>
                    <p className="page-description">Every event, every token, every dollar</p>
                </div>
                <div className="flex items-center gap-2">
                    <select
                        className="input"
                        value={selectedSessionId || ''}
                        onChange={e => {
                            setSelectedSessionId(e.target.value || undefined);
                            setSelectedEventId(undefined);
                        }}
                    >
                        <option value="">Select session...</option>
                        {sessions.map(s => (
                            <option key={s.id} value={s.id}>
                                {s.title} ({s.messageCount} msgs)
                            </option>
                        ))}
                    </select>
                    <button className="btn btn-secondary" onClick={handleExport} disabled={events.length === 0}>
                        <Download className="w-4 h-4" />
                        Export
                    </button>
                </div>
            </div>

            {error && (
                <div className="mt-2 p-3 rounded-lg bg-red-500/10 border border-red-500/30 text-red-400 text-sm">
                    {error}
                </div>
            )}

            <div className="flex-1 flex gap-4 mt-4 overflow-hidden">
                {/* Event Timeline */}
                <div className="w-96 panel flex flex-col">
                    <div className="panel-header">
                        <span className="panel-title">Event Timeline</span>
                        <span className="text-xs text-[var(--text-muted)]">{filteredEvents.length} events</span>
                    </div>
                    <div className="p-2">
                        <div className="relative">
                            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-[var(--text-muted)]" />
                            <input
                                type="text"
                                className="input w-full pl-9"
                                placeholder="Filter events..."
                                value={filter}
                                onChange={e => setFilter(e.target.value)}
                            />
                        </div>
                    </div>
                    <div className="flex-1 overflow-y-auto p-2 space-y-1">
                        {!selectedSessionId && (
                            <div className="text-center text-[var(--text-muted)] p-8 text-sm">
                                Select a session to view its events.
                            </div>
                        )}
                        {eventsLoading && (
                            <div className="flex items-center justify-center p-8 text-[var(--text-muted)]">
                                <Loader2 className="w-5 h-5 animate-spin mr-2" /> Loading...
                            </div>
                        )}
                        {filteredEvents.map((event) => (
                            <div
                                key={event.eventId}
                                className={`p-3 rounded-lg cursor-pointer transition-all ${
                                    selectedEventId === event.eventId
                                        ? 'bg-[var(--accent-glow)] border border-[var(--accent-primary)]'
                                        : 'bg-[var(--bg-tertiary)] hover:bg-[var(--bg-hover)]'
                                }`}
                                onClick={() => setSelectedEventId(event.eventId)}
                            >
                                <div className="flex items-center gap-2 mb-1">
                                    <div
                                        className="w-2 h-2 rounded-full"
                                        style={{ background: eventColors[event.type] || 'var(--text-muted)' }}
                                    />
                                    <span className="text-xs font-mono text-[var(--text-muted)]">{formatTimestamp(event.ts)}</span>
                                    <span className="text-xs font-mono font-medium text-[var(--accent-primary)]">{event.type}</span>
                                </div>
                                <div className="text-sm text-[var(--text-secondary)] pl-4 truncate">
                                    {summarizePayload(event.type, event.payload)}
                                </div>
                            </div>
                        ))}
                        {selectedSessionId && !eventsLoading && events.length === 0 && (
                            <div className="text-center text-[var(--text-muted)] p-8 text-sm">
                                No events recorded for this session yet.
                            </div>
                        )}
                    </div>
                </div>

                {/* Detail Panel */}
                <div className="flex-1 panel flex flex-col">
                    <div className="panel-header">
                        <span className="panel-title">Event Detail</span>
                        {selectedEvent && (
                            <span className="text-xs font-mono text-[var(--accent-primary)]">{selectedEvent.type}</span>
                        )}
                    </div>
                    {selectedEvent ? (
                        <div className="panel-content space-y-4 overflow-y-auto">
                            <div className="grid grid-cols-3 gap-4 text-sm">
                                <div>
                                    <span className="text-[var(--text-muted)]">Seq: </span>
                                    <span className="font-mono">{selectedEvent.seq}</span>
                                </div>
                                <div>
                                    <span className="text-[var(--text-muted)]">Source: </span>
                                    <span className="font-mono">{selectedEvent.source}</span>
                                </div>
                                <div>
                                    <span className="text-[var(--text-muted)]">Time: </span>
                                    <span className="font-mono">{formatTimestamp(selectedEvent.ts)}</span>
                                </div>
                            </div>
                            {selectedEvent.costUsd != null && selectedEvent.costUsd > 0 && (
                                <div className="text-sm">
                                    <span className="text-[var(--text-muted)]">Cost: </span>
                                    <span className="font-medium text-green-400">${selectedEvent.costUsd.toFixed(4)}</span>
                                </div>
                            )}
                            <div>
                                <label className="block text-xs font-medium text-[var(--text-muted)] mb-1">Payload</label>
                                <pre className="text-xs bg-[var(--bg-tertiary)] p-4 rounded-lg overflow-x-auto whitespace-pre-wrap font-mono">
                                    {JSON.stringify(selectedEvent.payload, null, 2)}
                                </pre>
                            </div>
                        </div>
                    ) : (
                        <div className="flex-1 flex items-center justify-center text-[var(--text-muted)]">
                            <div className="text-center">
                                <Search className="w-8 h-8 mx-auto mb-2 opacity-50" />
                                <p>Select an event from the timeline</p>
                                <p className="text-xs mt-1">See input, output, tokens, cost, and timing</p>
                            </div>
                        </div>
                    )}
                </div>
            </div>

            {/* Stats Bar */}
            {sessionStats && (
                <div className="mt-4 p-3 panel flex items-center gap-6 text-sm">
                    <div><span className="text-[var(--text-muted)]">Events: </span><span className="font-medium">{sessionStats.totalEvents}</span></div>
                    <div><span className="text-[var(--text-muted)]">Messages: </span><span className="font-medium">{sessionStats.totalMessages}</span></div>
                    <div>
                        <span className="text-[var(--text-muted)]">Tokens: </span>
                        <span className="font-medium">{(sessionStats.totalInputTokens + sessionStats.totalOutputTokens).toLocaleString()}</span>
                    </div>
                    <div>
                        <span className="text-[var(--text-muted)]">Cost: </span>
                        <span className="font-medium text-green-400">${sessionStats.totalCostUsd.toFixed(4)}</span>
                    </div>
                    {sessionStats.modelsUsed.length > 0 && (
                        <div>
                            <span className="text-[var(--text-muted)]">Models: </span>
                            <span className="font-medium">{sessionStats.modelsUsed.join(', ')}</span>
                        </div>
                    )}
                </div>
            )}
        </div>
    );
}
