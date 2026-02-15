import {
    Search, Download, Loader2, MessageSquare, Brain, Wrench,
    AlertCircle, Clock, Zap, DollarSign, Hash, Cpu, Copy, Check,
    GitBranch, Play, FileText, ChevronRight, ChevronDown,
} from 'lucide-react';
import { useState, useEffect, useCallback, useRef } from 'react';
import { useAppStore, type StudioEvent } from '../../state/store';

// ============================================
// COLOR SYSTEM (per agent-inspector.md spec)
// ============================================

type EventCategory = 'message' | 'llm' | 'tool' | 'session' | 'error';

function getEventCategory(type: string): EventCategory {
    if (type.startsWith('message.')) return 'message';
    if (type === 'llm.response.error') return 'error';
    if (type.startsWith('llm.')) return 'llm';
    if (type.startsWith('tool.')) return 'tool';
    if (type.startsWith('session.')) return 'session';
    return 'session';
}

const categoryIcons: Record<EventCategory, typeof MessageSquare> = {
    message: MessageSquare,
    llm: Brain,
    tool: Wrench,
    session: Zap,
    error: AlertCircle,
};

// Finer-grained colors for specific event types
function getEventColor(type: string): string {
    switch (type) {
        case 'message.user': return '#3B82F6';
        case 'message.assistant': return '#22C55E';
        case 'llm.request.started': return '#A855F7';
        case 'llm.response.completed': return '#A855F7';
        case 'llm.response.error': return '#EF4444';
        case 'tool.requested': return '#EAB308';
        case 'tool.approved': return '#22C55E';
        case 'tool.completed': return '#22C55E';
        case 'tool.rejected': return '#EF4444';
        case 'tool.denied': return '#EF4444';
        case 'session.started': return '#6B7280';
        case 'session.ended': return '#6B7280';
        default: return '#6B7280';
    }
}

// ============================================
// FILTERS
// ============================================

type FilterId = 'all' | 'messages' | 'llm' | 'tools' | 'errors';

const FILTERS: { id: FilterId; label: string; match: (type: string) => boolean }[] = [
    { id: 'all', label: 'All', match: () => true },
    { id: 'messages', label: 'Messages', match: (t) => t.startsWith('message.') },
    { id: 'llm', label: 'LLM', match: (t) => t.startsWith('llm.') },
    { id: 'tools', label: 'Tools', match: (t) => t.startsWith('tool.') },
    { id: 'errors', label: 'Errors', match: (t) => t.includes('error') || t.includes('denied') || t.includes('rejected') },
];

// ============================================
// HELPERS
// ============================================

function formatTimestamp(ts: string): string {
    try {
        return new Date(ts).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' });
    } catch {
        return ts;
    }
}

function formatDuration(ms: number): string {
    if (ms < 1000) return `${ms}ms`;
    return `${(ms / 1000).toFixed(1)}s`;
}

function formatTokens(n: number): string {
    if (n >= 1000) return `${(n / 1000).toFixed(1)}k`;
    return String(n);
}

function eventLabel(type: string): string {
    switch (type) {
        case 'message.user': return 'User Message';
        case 'message.assistant': return 'Assistant Response';
        case 'llm.request.started': return 'LLM Request';
        case 'llm.response.completed': return 'LLM Response';
        case 'llm.response.error': return 'LLM Error';
        case 'tool.requested': return 'Tool Requested';
        case 'tool.approved': return 'Tool Approved';
        case 'tool.completed': return 'Tool Completed';
        case 'tool.rejected': return 'Tool Denied';
        case 'session.started': return 'Session Started';
        case 'session.ended': return 'Session Ended';
        default: return type;
    }
}

function eventSummary(event: StudioEvent): string {
    const p = event.payload;
    switch (event.type) {
        case 'message.user':
        case 'message.assistant': {
            const content = (p.content as string) || '';
            return content.length > 100 ? content.slice(0, 100) + '...' : content;
        }
        case 'llm.request.started':
            return `${p.model || 'unknown'} via ${p.provider || 'unknown'}`;
        case 'llm.response.completed': {
            const inT = (p.input_tokens as number) || 0;
            const outT = (p.output_tokens as number) || 0;
            const dur = (p.duration_ms as number) || 0;
            return `${p.model || ''} — ${formatTokens(inT)} in / ${formatTokens(outT)} out — ${formatDuration(dur)}`;
        }
        case 'llm.response.error':
            return (p.error as string) || 'Unknown error';
        case 'tool.requested':
        case 'tool.approved':
        case 'tool.completed':
            return (p.tool as string) || (p.command as string) || '';
        default:
            return Object.keys(p).slice(0, 3).join(', ') || '—';
    }
}

// ============================================
// EVENT GROUPING
// ============================================

interface EventGroup {
    id: string;
    kind: 'single' | 'tool-group' | 'llm-group';
    events: StudioEvent[];
    label: string;
}

function groupEvents(events: StudioEvent[]): EventGroup[] {
    const groups: EventGroup[] = [];
    let i = 0;

    while (i < events.length) {
        const ev = events[i];

        // Tool group: consecutive tool.* events with the same tool name
        if (ev.type.startsWith('tool.')) {
            const toolName = String(ev.payload.tool || ev.payload.command || 'tool');
            const batch = [ev];
            let j = i + 1;
            while (j < events.length && events[j].type.startsWith('tool.')) {
                const next = events[j];
                const nextTool = String(next.payload.tool || next.payload.command || 'tool');
                if (nextTool === toolName) {
                    batch.push(next);
                    j++;
                } else {
                    break;
                }
            }
            if (batch.length > 1) {
                const seqRange = `seq ${batch[0].seq}-${batch[batch.length - 1].seq}`;
                groups.push({
                    id: `tg-${batch[0].eventId}`,
                    kind: 'tool-group',
                    events: batch,
                    label: `Tool: ${toolName} (${seqRange})`,
                });
                i = j;
                continue;
            }
        }

        // LLM group: llm.request.started followed by llm.response.*
        if (ev.type === 'llm.request.started') {
            const batch = [ev];
            let j = i + 1;
            while (j < events.length && events[j].type.startsWith('llm.')) {
                batch.push(events[j]);
                if (events[j].type === 'llm.response.completed' || events[j].type === 'llm.response.error') {
                    j++;
                    break;
                }
                j++;
            }
            if (batch.length > 1) {
                const model = String(ev.payload.model || 'LLM');
                const seqRange = `seq ${batch[0].seq}-${batch[batch.length - 1].seq}`;
                groups.push({
                    id: `lg-${batch[0].eventId}`,
                    kind: 'llm-group',
                    events: batch,
                    label: `LLM: ${model} (${seqRange})`,
                });
                i = j;
                continue;
            }
        }

        // Single event
        groups.push({ id: ev.eventId, kind: 'single', events: [ev], label: '' });
        i++;
    }
    return groups;
}

// ============================================
// COPY BUTTON
// ============================================

function CopyButton({ text }: { text: string }) {
    const [copied, setCopied] = useState(false);

    const handleCopy = useCallback(() => {
        navigator.clipboard.writeText(text);
        setCopied(true);
        setTimeout(() => setCopied(false), 1500);
    }, [text]);

    return (
        <button
            className="btn btn-ghost btn-sm"
            onClick={handleCopy}
            title="Copy to clipboard"
        >
            {copied ? <Check className="w-3 h-3 text-green-400" /> : <Copy className="w-3 h-3" />}
            {copied ? 'Copied' : 'Copy'}
        </button>
    );
}

// ============================================
// DETAIL PANEL — type-specific views
// ============================================

function EventDetail({ event, sessionId, onBranch }: {
    event: StudioEvent;
    sessionId?: string;
    onBranch?: (seq: number) => void;
}) {
    const p = event.payload;

    return (
        <div className="panel-content space-y-4 overflow-y-auto">
            {/* Header */}
            <div className="flex items-center gap-3">
                <div
                    className="w-3 h-3 rounded-full"
                    style={{ background: getEventColor(event.type) }}
                />
                <span className="font-semibold text-[var(--text-primary)]">
                    {eventLabel(event.type)}
                </span>
                <span className="text-xs font-mono text-[var(--text-muted)] ml-auto">
                    seq={event.seq}
                </span>
            </div>

            {/* Meta row */}
            <div className="flex flex-wrap gap-4 text-xs text-[var(--text-muted)]">
                <div className="flex items-center gap-1">
                    <Clock className="w-3 h-3" />
                    {formatTimestamp(event.ts)}
                </div>
                <div className="flex items-center gap-1">
                    <Hash className="w-3 h-3" />
                    {event.source}
                </div>
                {event.costUsd != null && event.costUsd > 0 && (
                    <div className="flex items-center gap-1">
                        <DollarSign className="w-3 h-3" />
                        <span className="text-green-400">${event.costUsd.toFixed(4)}</span>
                    </div>
                )}
            </div>

            <div className="border-t border-[var(--border-subtle)]" />

            {/* Type-specific content */}
            {event.type === 'message.user' && (
                <MessageDetail content={p.content as string} role="user" />
            )}
            {event.type === 'message.assistant' && (
                <MessageDetail
                    content={p.content as string}
                    role="assistant"
                    model={p.model as string}
                />
            )}
            {event.type === 'llm.request.started' && (
                <LlmRequestDetail payload={p} />
            )}
            {event.type === 'llm.response.completed' && (
                <LlmResponseDetail payload={p} />
            )}
            {event.type === 'llm.response.error' && (
                <ErrorDetail payload={p} />
            )}
            {event.type.startsWith('tool.') && (
                <ToolDetail payload={p} type={event.type} />
            )}
            {/* Fallback: raw payload for unknown types */}
            {!['message.user', 'message.assistant', 'llm.request.started',
              'llm.response.completed', 'llm.response.error'].includes(event.type) &&
              !event.type.startsWith('tool.') && (
                <RawPayload payload={p} />
            )}

            {/* Actions */}
            <div className="flex flex-wrap gap-2 pt-2 border-t border-[var(--border-subtle)]">
                <CopyButton text={JSON.stringify(p, null, 2)} />
                {sessionId && onBranch && (
                    <button
                        className="btn btn-ghost btn-sm"
                        onClick={() => onBranch(event.seq)}
                        title="Branch conversation from this point"
                    >
                        <GitBranch className="w-3 h-3" />
                        Branch from here
                    </button>
                )}
                {sessionId && onBranch && event.type === 'llm.response.error' && (
                    <button
                        className="btn btn-ghost btn-sm"
                        onClick={() => onBranch(Math.max(1, event.seq - 1))}
                        title="Retry from the previous point"
                    >
                        <Play className="w-3 h-3" />
                        Retry from here
                    </button>
                )}
            </div>
        </div>
    );
}

function MessageDetail({ content, role, model }: { content: string; role: string; model?: string }) {
    return (
        <div className="space-y-3">
            <div className={`p-4 rounded-lg text-sm whitespace-pre-wrap leading-relaxed ${
                role === 'user'
                    ? 'bg-blue-500/10 border border-blue-500/20'
                    : 'bg-green-500/10 border border-green-500/20'
            }`}>
                {content}
            </div>
            {model && (
                <div className="flex items-center gap-2 text-xs text-[var(--text-muted)]">
                    <Cpu className="w-3 h-3" />
                    Model: <span className="font-mono text-[var(--text-secondary)]">{model}</span>
                </div>
            )}
        </div>
    );
}

function LlmRequestDetail({ payload }: { payload: Record<string, unknown> }) {
    return (
        <div className="space-y-3">
            <div className="grid grid-cols-2 gap-3">
                <MetricCard label="Model" value={String(payload.model || '—')} />
                <MetricCard label="Provider" value={String(payload.provider || '—')} />
            </div>
            <div className="text-xs text-[var(--text-muted)] flex items-center gap-2">
                <Loader2 className="w-3 h-3 animate-spin" />
                Waiting for LLM response...
            </div>
        </div>
    );
}

function LlmResponseDetail({ payload }: { payload: Record<string, unknown> }) {
    const inputTokens = (payload.input_tokens as number) || 0;
    const outputTokens = (payload.output_tokens as number) || 0;
    const durationMs = (payload.duration_ms as number) || 0;

    return (
        <div className="space-y-3">
            <div className="grid grid-cols-2 gap-3">
                <MetricCard label="Model" value={String(payload.model || '—')} />
                <MetricCard label="Provider" value={String(payload.provider || '—')} />
            </div>
            <div className="grid grid-cols-3 gap-3">
                <MetricCard label="Input Tokens" value={formatTokens(inputTokens)} accent />
                <MetricCard label="Output Tokens" value={formatTokens(outputTokens)} accent />
                <MetricCard label="Duration" value={formatDuration(durationMs)} accent />
            </div>
        </div>
    );
}

function ErrorDetail({ payload }: { payload: Record<string, unknown> }) {
    return (
        <div className="space-y-3">
            <div className="p-4 rounded-lg bg-red-500/10 border border-red-500/20 text-red-400 text-sm">
                <div className="font-medium mb-1">Error</div>
                <div className="font-mono text-xs">{String(payload.error || 'Unknown error')}</div>
            </div>
            {payload.model != null && (
                <div className="text-xs text-[var(--text-muted)]">
                    Model: <span className="font-mono">{String(payload.model)}</span>
                </div>
            )}
        </div>
    );
}

function ToolDetail({ payload, type }: { payload: Record<string, unknown>; type: string }) {
    return (
        <div className="space-y-3">
            {payload.tool != null && (
                <MetricCard label="Tool" value={String(payload.tool)} />
            )}
            {payload.command != null && (
                <div>
                    <label className="block text-xs font-medium text-[var(--text-muted)] mb-1">Command</label>
                    <pre className="text-xs bg-[var(--bg-tertiary)] p-3 rounded-lg font-mono overflow-x-auto">
                        {String(payload.command)}
                    </pre>
                </div>
            )}
            {payload.input != null && (
                <CollapsibleSection title="Input">
                    <pre className="text-xs font-mono whitespace-pre-wrap">
                        {typeof payload.input === 'string' ? payload.input : JSON.stringify(payload.input, null, 2)}
                    </pre>
                </CollapsibleSection>
            )}
            {payload.output != null && (
                <CollapsibleSection title="Output">
                    <pre className="text-xs font-mono whitespace-pre-wrap">
                        {typeof payload.output === 'string' ? payload.output : JSON.stringify(payload.output, null, 2)}
                    </pre>
                </CollapsibleSection>
            )}
            {payload.duration_ms != null && (
                <div className="text-xs text-[var(--text-muted)]">
                    Duration: <span className="font-mono">{formatDuration(payload.duration_ms as number)}</span>
                </div>
            )}
            {(type === 'tool.rejected' || type === 'tool.denied') && (
                <div className="text-xs text-red-400">Tool execution was denied.</div>
            )}
        </div>
    );
}

function RawPayload({ payload }: { payload: Record<string, unknown> }) {
    return (
        <div>
            <label className="block text-xs font-medium text-[var(--text-muted)] mb-1">Payload</label>
            <pre className="text-xs bg-[var(--bg-tertiary)] p-4 rounded-lg overflow-x-auto whitespace-pre-wrap font-mono">
                {JSON.stringify(payload, null, 2)}
            </pre>
        </div>
    );
}

// ============================================
// SHARED COMPONENTS
// ============================================

function MetricCard({ label, value, accent }: { label: string; value: string; accent?: boolean }) {
    return (
        <div className="p-3 rounded-lg bg-[var(--bg-tertiary)]">
            <div className="text-xs text-[var(--text-muted)] mb-1">{label}</div>
            <div className={`text-sm font-mono font-medium ${accent ? 'text-[var(--accent-secondary)]' : 'text-[var(--text-primary)]'}`}>
                {value}
            </div>
        </div>
    );
}

function CollapsibleSection({ title, children }: { title: string; children: React.ReactNode }) {
    const [open, setOpen] = useState(false);
    return (
        <div>
            <button
                className="flex items-center gap-1 text-xs font-medium text-[var(--text-muted)] hover:text-[var(--text-secondary)] transition-colors"
                onClick={() => setOpen(!open)}
            >
                <span>{open ? '▾' : '▸'}</span> {title}
            </button>
            {open && (
                <div className="mt-1 p-3 rounded-lg bg-[var(--bg-tertiary)] overflow-x-auto">
                    {children}
                </div>
            )}
        </div>
    );
}

// ============================================
// TIMELINE EVENT (reusable for flat + grouped rendering)
// ============================================

function TimelineEvent({ event, isSelected, onSelect }: {
    event: StudioEvent;
    isSelected: boolean;
    onSelect: (id: string) => void;
}) {
    const category = getEventCategory(event.type);
    const color = getEventColor(event.type);
    const Icon = categoryIcons[category];

    return (
        <div
            className={`relative flex gap-3 py-2 px-2 rounded-lg cursor-pointer transition-all ${
                isSelected
                    ? 'bg-[var(--accent-glow)]'
                    : 'hover:bg-[var(--bg-hover)]'
            }`}
            onClick={() => onSelect(event.eventId)}
        >
            {/* Dot with icon */}
            <div
                className="relative z-10 w-5 h-5 rounded-full flex items-center justify-center flex-shrink-0 mt-0.5"
                style={{
                    background: isSelected ? color : `${color}33`,
                    border: `2px solid ${color}`,
                }}
            >
                <Icon className="w-2.5 h-2.5" style={{ color: isSelected ? '#fff' : color }} />
            </div>

            {/* Content */}
            <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2">
                    <span className="text-xs font-mono font-medium" style={{ color }}>
                        {event.type}
                    </span>
                    <span className="text-[10px] text-[var(--text-muted)] ml-auto flex-shrink-0">
                        {formatTimestamp(event.ts)}
                    </span>
                </div>
                <div className="text-xs text-[var(--text-muted)] truncate mt-0.5">
                    {eventSummary(event)}
                </div>
            </div>
        </div>
    );
}

// ============================================
// MAIN INSPECTOR PAGE
// ============================================

export function InspectorPage() {
    const {
        sessions, fetchSessions,
        events, eventsLoading, fetchEvents,
        sessionStats, fetchSessionStats,
        error,
        inspectorSessionId, clearInspectorSession,
        branchSession,
    } = useAppStore();

    const [selectedSessionId, setSelectedSessionId] = useState<string | undefined>();
    const [selectedEventId, setSelectedEventId] = useState<string | undefined>();
    const [activeFilter, setActiveFilter] = useState<FilterId>('all');
    const [searchQuery, setSearchQuery] = useState('');
    const [collapsedGroups, setCollapsedGroups] = useState<Set<string>>(new Set());
    const [showFilters, setShowFilters] = useState(true);
    const searchRef = useRef<HTMLInputElement>(null);
    const lastKeyRef = useRef<string>('');

    useEffect(() => {
        fetchSessions();
    }, [fetchSessions]);

    // Auto-select session when navigated from Sessions page "Inspect" button
    useEffect(() => {
        if (inspectorSessionId) {
            setSelectedSessionId(inspectorSessionId);
            clearInspectorSession();
        }
    }, [inspectorSessionId, clearInspectorSession]);

    // Load events when session changes
    useEffect(() => {
        if (selectedSessionId) {
            fetchEvents(selectedSessionId);
            fetchSessionStats(selectedSessionId);
            setSelectedEventId(undefined);
            setActiveFilter('all');
            setSearchQuery('');
            setCollapsedGroups(new Set());
        }
    }, [selectedSessionId, fetchEvents, fetchSessionStats]);

    const selectedSession = sessions.find(s => s.id === selectedSessionId);
    const selectedEvent = events.find(e => e.eventId === selectedEventId);

    // Apply filters and search
    const filteredEvents = events.filter(e => {
        const filterDef = FILTERS.find(f => f.id === activeFilter);
        if (filterDef && !filterDef.match(e.type)) return false;
        if (searchQuery) {
            const q = searchQuery.toLowerCase();
            return (
                e.type.toLowerCase().includes(q) ||
                JSON.stringify(e.payload).toLowerCase().includes(q)
            );
        }
        return true;
    });

    // Group filtered events
    const eventGroups = groupEvents(filteredEvents);

    // Flatten visible events (respecting collapsed groups) for keyboard nav
    const visibleEvents = eventGroups.flatMap(g =>
        g.kind === 'single' || !collapsedGroups.has(g.id)
            ? g.events
            : [g.events[0]] // show first event as group header when collapsed
    );

    // Count events per filter for badge numbers
    const filterCounts: Record<FilterId, number> = {
        all: events.length,
        messages: events.filter(e => e.type.startsWith('message.')).length,
        llm: events.filter(e => e.type.startsWith('llm.')).length,
        tools: events.filter(e => e.type.startsWith('tool.')).length,
        errors: events.filter(e => e.type.includes('error') || e.type.includes('denied')).length,
    };

    const toggleGroup = (groupId: string) => {
        setCollapsedGroups(prev => {
            const next = new Set(prev);
            if (next.has(groupId)) next.delete(groupId);
            else next.add(groupId);
            return next;
        });
    };

    const downloadFile = (content: string, filename: string, mime: string) => {
        const blob = new Blob([content], { type: mime });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = filename;
        a.click();
        URL.revokeObjectURL(url);
    };

    const fileSlug = selectedSession?.title?.replace(/\s+/g, '-') || selectedSessionId || 'export';

    const handleExportJSON = () => {
        const exportData = {
            export_version: 1,
            exported_at: new Date().toISOString(),
            session: selectedSession || null,
            events,
            stats: sessionStats,
        };
        downloadFile(JSON.stringify(exportData, null, 2), `inspector-${fileSlug}.json`, 'application/json');
    };

    const handleExportMarkdown = () => {
        const lines: string[] = [];
        lines.push(`# Session: ${selectedSession?.title || 'Untitled'}`);
        lines.push(`Agent: ${selectedSession?.agentName || '?'} (${selectedSession?.agentModel || '?'})`);
        lines.push(`Date: ${selectedSession ? new Date(selectedSession.createdAt).toLocaleString() : '?'}`);
        if (sessionStats) {
            lines.push(`Tokens: ${sessionStats.totalInputTokens} in / ${sessionStats.totalOutputTokens} out | Cost: $${sessionStats.totalCostUsd.toFixed(4)} | Events: ${sessionStats.totalEvents}`);
        }
        lines.push('', '---', '');

        for (const ev of events) {
            const p = ev.payload;
            switch (ev.type) {
                case 'message.user':
                    lines.push(`**User**: ${p.content || ''}`, '');
                    break;
                case 'message.assistant':
                    lines.push(`**Assistant**: ${p.content || ''}`, '');
                    break;
                case 'tool.requested':
                    lines.push(`> Tool requested: \`${p.tool || p.command || '?'}\``);
                    if (p.input) lines.push(`> Input: \`${typeof p.input === 'string' ? p.input : JSON.stringify(p.input)}\``);
                    lines.push('');
                    break;
                case 'tool.completed':
                    lines.push(`> Tool completed: \`${p.tool || p.command || '?'}\``);
                    if (p.output) {
                        const out = typeof p.output === 'string' ? p.output : JSON.stringify(p.output);
                        lines.push(`> Output: ${out.length > 200 ? out.slice(0, 200) + '...' : out}`);
                    }
                    if (p.duration_ms) lines.push(`> Duration: ${formatDuration(p.duration_ms as number)}`);
                    lines.push('');
                    break;
                case 'llm.response.completed':
                    lines.push(`> LLM: ${p.model || '?'} — ${formatTokens((p.input_tokens as number) || 0)} in / ${formatTokens((p.output_tokens as number) || 0)} out — ${formatDuration((p.duration_ms as number) || 0)}`, '');
                    break;
                case 'llm.response.error':
                    lines.push(`> **LLM Error**: ${p.error || 'Unknown'}`, '');
                    break;
            }
        }
        downloadFile(lines.join('\n'), `inspector-${fileSlug}.md`, 'text/markdown');
    };

    // Branch from inspector
    const handleBranch = useCallback(async (seq: number) => {
        if (!selectedSessionId) return;
        try {
            const newSession = await branchSession(selectedSessionId, seq);
            // Navigate to the new branch in inspector
            setSelectedSessionId(newSession.id);
        } catch {
            // Error handled by store (toast)
        }
    }, [selectedSessionId, branchSession]);

    // Keyboard navigation — extended shortcuts
    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            // Don't handle if typing in an input
            const tag = (e.target as HTMLElement)?.tagName;
            if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') {
                // Only handle Escape to blur search
                if (e.key === 'Escape') {
                    (e.target as HTMLElement).blur();
                    e.preventDefault();
                }
                return;
            }

            // Cmd+F: focus search
            if ((e.metaKey || e.ctrlKey) && e.key === 'f') {
                e.preventDefault();
                searchRef.current?.focus();
                return;
            }

            // Cmd+E: export JSON
            if ((e.metaKey || e.ctrlKey) && e.key === 'e') {
                e.preventDefault();
                if (events.length > 0) handleExportJSON();
                return;
            }

            // F: toggle filters
            if (e.key === 'f' && !e.metaKey && !e.ctrlKey) {
                e.preventDefault();
                setShowFilters(v => !v);
                return;
            }

            // G+G: jump to first event
            if (e.key === 'g' && !e.metaKey && !e.ctrlKey && !e.shiftKey) {
                if (lastKeyRef.current === 'g') {
                    e.preventDefault();
                    if (filteredEvents.length) setSelectedEventId(filteredEvents[0].eventId);
                    lastKeyRef.current = '';
                    return;
                }
                lastKeyRef.current = 'g';
                setTimeout(() => { lastKeyRef.current = ''; }, 500);
                return;
            }

            // Shift+G: jump to last event
            if (e.key === 'G' || (e.key === 'g' && e.shiftKey)) {
                e.preventDefault();
                if (filteredEvents.length) setSelectedEventId(filteredEvents[filteredEvents.length - 1].eventId);
                return;
            }

            // [ and ]: collapse/expand all groups
            if (e.key === '[') {
                e.preventDefault();
                setCollapsedGroups(new Set(eventGroups.filter(g => g.kind !== 'single').map(g => g.id)));
                return;
            }
            if (e.key === ']') {
                e.preventDefault();
                setCollapsedGroups(new Set());
                return;
            }

            if (!filteredEvents.length) return;
            const currentIndex = visibleEvents.findIndex(ev => ev.eventId === selectedEventId);

            if (e.key === 'ArrowDown' || e.key === 'j') {
                e.preventDefault();
                const next = Math.min(currentIndex + 1, visibleEvents.length - 1);
                setSelectedEventId(visibleEvents[next]?.eventId);
            } else if (e.key === 'ArrowUp' || e.key === 'k') {
                e.preventDefault();
                const prev = Math.max(currentIndex - 1, 0);
                setSelectedEventId(visibleEvents[prev]?.eventId);
            } else if (e.key === 'Enter') {
                // Enter: select current event (already selected, but ensure detail panel shows)
                e.preventDefault();
            }
        };
        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [filteredEvents, visibleEvents, selectedEventId, events.length, eventGroups]);

    return (
        <div className="animate-fade-in h-full flex flex-col">
            {/* Header */}
            <div className="page-header">
                <div>
                    <h1 className="page-title">Inspector</h1>
                    <p className="page-description">Every event, every token, every dollar</p>
                </div>
                <div className="flex items-center gap-3">
                    <select
                        className="input"
                        style={{ width: '280px' }}
                        value={selectedSessionId || ''}
                        onChange={e => setSelectedSessionId(e.target.value || undefined)}
                    >
                        <option value="">Select session...</option>
                        {sessions.map(s => (
                            <option key={s.id} value={s.id}>
                                {s.title} ({s.messageCount} msgs{s.agentModel ? ` · ${s.agentModel}` : ''})
                            </option>
                        ))}
                    </select>
                    <button
                        className="btn btn-secondary"
                        onClick={handleExportJSON}
                        disabled={events.length === 0}
                        title="Export session as JSON (Cmd+E)"
                    >
                        <Download className="w-4 h-4" />
                        JSON
                    </button>
                    <button
                        className="btn btn-secondary"
                        onClick={handleExportMarkdown}
                        disabled={events.length === 0}
                        title="Export session as Markdown"
                    >
                        <FileText className="w-4 h-4" />
                        Markdown
                    </button>
                </div>
            </div>

            {error && (
                <div className="p-3 rounded-lg bg-red-500/10 border border-red-500/30 text-red-400 text-sm">
                    {error}
                </div>
            )}

            {/* Session info bar */}
            {selectedSession && (
                <div className="flex items-center gap-4 px-3 py-2 rounded-lg bg-[var(--bg-secondary)] border border-[var(--border-subtle)] text-xs text-[var(--text-muted)] mb-2">
                    <span>Agent: <span className="text-[var(--text-secondary)] font-medium">{selectedSession.agentName}</span></span>
                    <span>Model: <span className="text-[var(--text-secondary)] font-mono">{selectedSession.agentModel}</span></span>
                    <span>Status: <span className={`font-medium ${selectedSession.status === 'active' ? 'text-green-400' : 'text-[var(--text-secondary)]'}`}>{selectedSession.status}</span></span>
                    <span className="ml-auto">{new Date(selectedSession.createdAt).toLocaleString()}</span>
                </div>
            )}

            {/* Main content */}
            <div className="flex-1 flex gap-3 overflow-hidden">
                {/* Event Timeline (left panel) */}
                <div className="w-[380px] panel flex flex-col">
                    <div className="panel-header">
                        <span className="panel-title">Timeline</span>
                        <span className="text-xs text-[var(--text-muted)]">
                            {filteredEvents.length}{filteredEvents.length !== events.length ? ` / ${events.length}` : ''} events
                        </span>
                    </div>

                    {/* Filter chips (toggle with F key) */}
                    {events.length > 0 && showFilters && (
                        <div className="flex gap-1 p-2 border-b border-[var(--border-subtle)]">
                            {FILTERS.map(f => (
                                <button
                                    key={f.id}
                                    className={`px-2 py-1 rounded text-xs font-medium transition-all ${
                                        activeFilter === f.id
                                            ? 'bg-[var(--accent-primary)] text-white'
                                            : 'bg-[var(--bg-tertiary)] text-[var(--text-muted)] hover:text-[var(--text-secondary)]'
                                    }`}
                                    onClick={() => setActiveFilter(f.id)}
                                >
                                    {f.label}
                                    {filterCounts[f.id] > 0 && (
                                        <span className="ml-1 opacity-70">{filterCounts[f.id]}</span>
                                    )}
                                </button>
                            ))}
                        </div>
                    )}

                    {/* Search */}
                    <div className="p-2 border-b border-[var(--border-subtle)]">
                        <div className="relative">
                            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-[var(--text-muted)]" />
                            <input
                                ref={searchRef}
                                type="text"
                                className="input w-full pl-8 text-xs"
                                placeholder="Search events... (Cmd+F)"
                                value={searchQuery}
                                onChange={e => setSearchQuery(e.target.value)}
                            />
                        </div>
                    </div>

                    {/* Event list */}
                    <div className="flex-1 overflow-y-auto">
                        {!selectedSessionId && (
                            <div className="flex flex-col items-center justify-center h-full text-[var(--text-muted)] p-8">
                                <Search className="w-8 h-8 mb-3 opacity-30" />
                                <p className="text-sm">Select a session to inspect</p>
                                <p className="text-xs mt-1">See every event, decision, and cost</p>
                            </div>
                        )}
                        {eventsLoading && (
                            <div className="flex items-center justify-center p-8 text-[var(--text-muted)]">
                                <Loader2 className="w-5 h-5 animate-spin mr-2" /> Loading events...
                            </div>
                        )}
                        {selectedSessionId && !eventsLoading && events.length === 0 && (
                            <div className="flex flex-col items-center justify-center h-full text-[var(--text-muted)] p-8">
                                <Zap className="w-8 h-8 mb-3 opacity-30" />
                                <p className="text-sm">No events yet</p>
                                <p className="text-xs mt-1">Send a message in this session to generate events</p>
                            </div>
                        )}

                        {/* Timeline entries with connecting line + grouping */}
                        <div className="relative px-3 py-2">
                            {/* Vertical connecting line */}
                            {filteredEvents.length > 1 && (
                                <div
                                    className="absolute left-[21px] top-6 bottom-6 w-[2px] bg-[var(--border-subtle)]"
                                />
                            )}

                            {eventGroups.map((group) => {
                                if (group.kind === 'single') {
                                    return <TimelineEvent
                                        key={group.id}
                                        event={group.events[0]}
                                        isSelected={selectedEventId === group.events[0].eventId}
                                        onSelect={setSelectedEventId}
                                    />;
                                }

                                // Group rendering
                                const isCollapsed = collapsedGroups.has(group.id);
                                const groupColor = group.kind === 'tool-group' ? '#EAB308' : '#A855F7';
                                const GroupIcon = group.kind === 'tool-group' ? Wrench : Brain;

                                return (
                                    <div key={group.id} className="relative">
                                        {/* Group header */}
                                        <div
                                            className="flex items-center gap-2 py-1.5 px-2 rounded-lg cursor-pointer hover:bg-[var(--bg-hover)] transition-all"
                                            onClick={() => toggleGroup(group.id)}
                                        >
                                            {isCollapsed
                                                ? <ChevronRight className="w-3 h-3 text-[var(--text-muted)]" />
                                                : <ChevronDown className="w-3 h-3 text-[var(--text-muted)]" />
                                            }
                                            <div
                                                className="w-4 h-4 rounded flex items-center justify-center flex-shrink-0"
                                                style={{ background: `${groupColor}22`, border: `1px solid ${groupColor}55` }}
                                            >
                                                <GroupIcon className="w-2.5 h-2.5" style={{ color: groupColor }} />
                                            </div>
                                            <span className="text-xs font-medium" style={{ color: groupColor }}>
                                                {group.label}
                                            </span>
                                            <span className="text-[10px] text-[var(--text-muted)] ml-auto">
                                                {group.events.length} events
                                            </span>
                                        </div>
                                        {/* Group children */}
                                        {!isCollapsed && (
                                            <div className="ml-3 pl-2 border-l-2" style={{ borderColor: `${groupColor}44` }}>
                                                {group.events.map(event => (
                                                    <TimelineEvent
                                                        key={event.eventId}
                                                        event={event}
                                                        isSelected={selectedEventId === event.eventId}
                                                        onSelect={setSelectedEventId}
                                                    />
                                                ))}
                                            </div>
                                        )}
                                    </div>
                                );
                            })}
                        </div>
                    </div>
                </div>

                {/* Detail Panel (right panel) */}
                <div className="flex-1 panel flex flex-col">
                    <div className="panel-header">
                        <span className="panel-title">Detail</span>
                        {selectedEvent && (
                            <span className="text-xs font-mono" style={{ color: getEventColor(selectedEvent.type) }}>
                                {selectedEvent.type}
                            </span>
                        )}
                    </div>
                    {selectedEvent ? (
                        <EventDetail event={selectedEvent} sessionId={selectedSessionId} onBranch={handleBranch} />
                    ) : (
                        <div className="flex-1 flex items-center justify-center text-[var(--text-muted)]">
                            <div className="text-center">
                                <Brain className="w-10 h-10 mx-auto mb-3 opacity-20" />
                                <p className="text-sm font-medium">Select an event</p>
                                <p className="text-xs mt-1 max-w-[200px]">
                                    Click any event in the timeline to see its full details
                                </p>
                                <div className="text-xs mt-3 text-[var(--text-muted)] space-y-1">
                                    <p><kbd className="px-1 py-0.5 rounded bg-[var(--bg-tertiary)] text-[10px]">j</kbd> <kbd className="px-1 py-0.5 rounded bg-[var(--bg-tertiary)] text-[10px]">k</kbd> navigate &nbsp; <kbd className="px-1 py-0.5 rounded bg-[var(--bg-tertiary)] text-[10px]">gg</kbd> first &nbsp; <kbd className="px-1 py-0.5 rounded bg-[var(--bg-tertiary)] text-[10px]">G</kbd> last</p>
                                    <p><kbd className="px-1 py-0.5 rounded bg-[var(--bg-tertiary)] text-[10px]">Cmd+F</kbd> search &nbsp; <kbd className="px-1 py-0.5 rounded bg-[var(--bg-tertiary)] text-[10px]">f</kbd> filters &nbsp; <kbd className="px-1 py-0.5 rounded bg-[var(--bg-tertiary)] text-[10px]">[</kbd> <kbd className="px-1 py-0.5 rounded bg-[var(--bg-tertiary)] text-[10px]">]</kbd> groups</p>
                                </div>
                            </div>
                        </div>
                    )}
                </div>
            </div>

            {/* Stats Bar (bottom) */}
            {sessionStats && (
                <div className="mt-3 p-3 panel">
                    <div className="flex items-center gap-6 text-sm">
                        <StatItem
                            icon={<Hash className="w-3.5 h-3.5" />}
                            label="Events"
                            value={String(sessionStats.totalEvents)}
                        />
                        <StatItem
                            icon={<MessageSquare className="w-3.5 h-3.5" />}
                            label="Messages"
                            value={String(sessionStats.totalMessages)}
                        />
                        <div className="h-4 w-px bg-[var(--border-subtle)]" />
                        <StatItem
                            icon={<Zap className="w-3.5 h-3.5" />}
                            label="Input"
                            value={formatTokens(sessionStats.totalInputTokens)}
                            suffix="tokens"
                        />
                        <StatItem
                            icon={<Zap className="w-3.5 h-3.5" />}
                            label="Output"
                            value={formatTokens(sessionStats.totalOutputTokens)}
                            suffix="tokens"
                        />
                        <div className="h-4 w-px bg-[var(--border-subtle)]" />
                        <StatItem
                            icon={<DollarSign className="w-3.5 h-3.5" />}
                            label="Cost"
                            value={`$${sessionStats.totalCostUsd.toFixed(4)}`}
                            accent
                        />
                        {sessionStats.modelsUsed.length > 0 && (
                            <>
                                <div className="h-4 w-px bg-[var(--border-subtle)]" />
                                <StatItem
                                    icon={<Cpu className="w-3.5 h-3.5" />}
                                    label="Models"
                                    value={sessionStats.modelsUsed.join(', ')}
                                />
                            </>
                        )}
                    </div>
                </div>
            )}
        </div>
    );
}

function StatItem({ icon, label, value, suffix, accent }: {
    icon: React.ReactNode; label: string; value: string; suffix?: string; accent?: boolean;
}) {
    return (
        <div className="flex items-center gap-2">
            <span className="text-[var(--text-muted)]">{icon}</span>
            <span className="text-xs text-[var(--text-muted)]">{label}:</span>
            <span className={`text-xs font-medium font-mono ${accent ? 'text-green-400' : 'text-[var(--text-primary)]'}`}>
                {value}
            </span>
            {suffix && <span className="text-[10px] text-[var(--text-muted)]">{suffix}</span>}
        </div>
    );
}
