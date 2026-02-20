import { useRef, useEffect, useState, useCallback } from 'react';
import { ChevronDown, ChevronUp, Trash2, ArrowDown } from 'lucide-react';
import { useAppStore } from '../../../state/store';

export function LiveFeedPanel() {
    const { liveFeedItems, liveMode, liveStartedAt, clearLiveFeed, liveSessionId, openInspector } = useAppStore();
    const [collapsed, setCollapsed] = useState(false);
    const [autoScroll, setAutoScroll] = useState(true);
    const scrollRef = useRef<HTMLDivElement>(null);

    // Filter to iteration items only for display
    const iterations = liveFeedItems.filter(
        (i) => i.type === 'live.iteration.completed' || i.type === 'live.iteration.error'
    );

    // Compute totals
    const totalIterations = iterations.length;
    const totalTokens = iterations.reduce((sum, i) => sum + (i.tokens ?? 0), 0);
    const totalCost = iterations.reduce((sum, i) => sum + (i.costUsd ?? 0), 0);
    const elapsed = liveStartedAt ? Math.floor((Date.now() - liveStartedAt) / 1000) : 0;
    const elapsedStr = elapsed >= 3600
        ? `${Math.floor(elapsed / 3600)}h ${Math.floor((elapsed % 3600) / 60)}m`
        : elapsed >= 60
            ? `${Math.floor(elapsed / 60)}m ${elapsed % 60}s`
            : `${elapsed}s`;

    // Auto-scroll
    useEffect(() => {
        if (autoScroll && scrollRef.current) {
            scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
        }
    }, [iterations.length, autoScroll]);

    const handleScroll = useCallback(() => {
        if (!scrollRef.current) return;
        const { scrollTop, scrollHeight, clientHeight } = scrollRef.current;
        // If user scrolled more than 50px from bottom, pause auto-scroll
        setAutoScroll(scrollHeight - scrollTop - clientHeight < 50);
    }, []);

    // Update elapsed time every second
    const [, setTick] = useState(0);
    useEffect(() => {
        if (!liveMode) return;
        const interval = setInterval(() => setTick((t) => t + 1), 1000);
        return () => clearInterval(interval);
    }, [liveMode]);

    if (iterations.length === 0 && !liveMode) return null;

    return (
        <div className="border-t border-[var(--border-subtle)] bg-[var(--bg-secondary)]">
            {/* Header */}
            <div
                className="flex items-center justify-between px-4 py-1.5 cursor-pointer hover:bg-[var(--bg-tertiary)]"
                onClick={() => setCollapsed(!collapsed)}
            >
                <div className="flex items-center gap-2 text-sm">
                    {collapsed ? <ChevronUp size={14} /> : <ChevronDown size={14} />}
                    {liveMode && (
                        <span className="w-2 h-2 rounded-full bg-green-500 animate-pulse" />
                    )}
                    <span className="font-medium">Live Feed</span>
                    <span className="text-xs text-[var(--text-muted)]">
                        #{totalIterations} iterations
                    </span>
                    <span className="text-xs text-[var(--text-muted)]">
                        {totalTokens.toLocaleString()} tokens
                    </span>
                    <span className="text-xs text-[var(--text-muted)]">
                        ${totalCost.toFixed(4)}
                    </span>
                    {liveMode && (
                        <span className="text-xs text-[var(--text-muted)]">
                            {elapsedStr}
                        </span>
                    )}
                </div>
                <div className="flex items-center gap-1">
                    {liveSessionId && (
                        <button
                            className="btn-icon-sm"
                            title="Open in Inspector"
                            onClick={(e) => { e.stopPropagation(); openInspector(liveSessionId); }}
                        >
                            Inspector
                        </button>
                    )}
                    <button
                        className="btn-icon-sm"
                        title="Clear feed"
                        onClick={(e) => { e.stopPropagation(); clearLiveFeed(); }}
                    >
                        <Trash2 size={12} />
                    </button>
                </div>
            </div>

            {/* Feed body */}
            {!collapsed && (
                <div
                    ref={scrollRef}
                    onScroll={handleScroll}
                    className="max-h-52 overflow-y-auto text-xs font-mono"
                >
                    {iterations.length === 0 ? (
                        <div className="px-4 py-3 text-[var(--text-muted)] text-center">
                            {liveMode ? 'Waiting for first iteration...' : 'No feed items'}
                        </div>
                    ) : (
                        <table className="w-full">
                            <tbody>
                                {iterations.map((item, idx) => {
                                    const isError = item.type === 'live.iteration.error';
                                    const ts = item.timestamp
                                        ? new Date(item.timestamp).toLocaleTimeString()
                                        : '';
                                    return (
                                        <tr
                                            key={idx}
                                            className={`border-b border-[var(--border-subtle)] hover:bg-[var(--bg-tertiary)] ${isError ? 'bg-red-950/20' : ''}`}
                                        >
                                            <td className="px-2 py-1 text-[var(--text-muted)] whitespace-nowrap w-16">
                                                {ts}
                                            </td>
                                            <td className="px-2 py-1 text-[var(--text-muted)] whitespace-nowrap w-10">
                                                #{item.iteration}
                                            </td>
                                            <td className={`px-2 py-1 truncate max-w-[400px] ${isError ? 'text-red-400' : 'text-[var(--text-primary)]'}`}>
                                                {isError
                                                    ? `Error: ${item.error}`
                                                    : item.outputSummary || '(no output)'}
                                            </td>
                                            <td className="px-2 py-1 text-[var(--text-muted)] whitespace-nowrap w-14 text-right">
                                                {isError ? '---' : `${((item.durationMs ?? 0) / 1000).toFixed(1)}s`}
                                            </td>
                                            <td className="px-2 py-1 text-[var(--text-muted)] whitespace-nowrap w-16 text-right">
                                                {isError ? '---' : `${item.tokens ?? 0} tok`}
                                            </td>
                                            <td className="px-2 py-1 text-[var(--text-muted)] whitespace-nowrap w-16 text-right">
                                                {isError ? '---' : `$${(item.costUsd ?? 0).toFixed(4)}`}
                                            </td>
                                        </tr>
                                    );
                                })}
                            </tbody>
                        </table>
                    )}

                    {/* Jump to latest button */}
                    {!autoScroll && iterations.length > 5 && (
                        <div className="sticky bottom-2 flex justify-center">
                            <button
                                className="px-2 py-1 rounded bg-blue-600/90 text-white text-xs flex items-center gap-1"
                                onClick={() => {
                                    setAutoScroll(true);
                                    if (scrollRef.current) {
                                        scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
                                    }
                                }}
                            >
                                <ArrowDown size={12} /> Jump to latest
                            </button>
                        </div>
                    )}
                </div>
            )}
        </div>
    );
}
