import { useState, useCallback, useMemo } from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import {
    Copy, Check, Maximize2, Minimize2,
    ChevronRight, ChevronDown,
    ArrowUpDown, ArrowUp, ArrowDown,
    Table as TableIcon, Braces, FileText,
} from 'lucide-react';

// ---------------------------------------------------------------------------
// Content type detection
// ---------------------------------------------------------------------------

type ContentType = 'json-array' | 'json-object' | 'markdown' | 'text';

function detectContentType(content: string): ContentType {
    const trimmed = content.trim();

    // JSON array of objects → table
    if (trimmed.startsWith('[')) {
        try {
            const parsed = JSON.parse(trimmed);
            if (Array.isArray(parsed) && parsed.length > 0 && typeof parsed[0] === 'object' && parsed[0] !== null) {
                return 'json-array';
            }
        } catch { /* not valid JSON */ }
    }

    // JSON object → tree view
    if (trimmed.startsWith('{')) {
        try {
            JSON.parse(trimmed);
            return 'json-object';
        } catch { /* not valid JSON */ }
    }

    // Markdown indicators: headers, tables, bold, code fences, lists
    if (/^#{1,6}\s/m.test(trimmed) ||
        /\|.*\|.*\n\|[\s:|-]+\|/m.test(trimmed) ||
        /\*\*.*\*\*/m.test(trimmed) ||
        /```[\s\S]*?```/m.test(trimmed) ||
        /^[-*+]\s/m.test(trimmed) ||
        /^\d+\.\s/m.test(trimmed)) {
        return 'markdown';
    }

    return 'text';
}

// ---------------------------------------------------------------------------
// Clipboard helper
// ---------------------------------------------------------------------------

async function copyText(text: string): Promise<void> {
    try {
        await navigator.clipboard.writeText(text);
    } catch {
        const ta = document.createElement('textarea');
        ta.value = text;
        document.body.appendChild(ta);
        ta.select();
        document.execCommand('copy');
        document.body.removeChild(ta);
    }
}

function CopyButton({ text, size = 12 }: { text: string; size?: number }) {
    const [copied, setCopied] = useState(false);
    const handleCopy = useCallback(async () => {
        await copyText(text);
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
    }, [text]);
    return (
        <button
            className="p-0.5 rounded bg-[#2a2a2a] hover:bg-[#3a3a3a] text-[var(--text-muted)] hover:text-[var(--text-primary)]"
            onClick={handleCopy}
            title="Copy to clipboard"
        >
            {copied ? <Check size={size} className="text-green-400" /> : <Copy size={size} />}
        </button>
    );
}

// ---------------------------------------------------------------------------
// Code block with language label + copy
// ---------------------------------------------------------------------------

function CodeBlock({ className, children }: { className?: string; children: React.ReactNode }) {
    const text = String(children).replace(/\n$/, '');
    const lang = className?.replace(/^language-/, '') || '';

    return (
        <div className="relative group/code my-1.5">
            {lang && (
                <span className="absolute top-1 left-2 text-[9px] uppercase tracking-wider text-[#666] font-mono">
                    {lang}
                </span>
            )}
            <div className="absolute top-1 right-1 opacity-0 group-hover/code:opacity-100 transition-opacity">
                <CopyButton text={text} size={11} />
            </div>
            <pre className="bg-[#1a1a1a] p-2 pt-5 rounded overflow-x-auto font-mono text-[0.9em] leading-relaxed">
                <code>{text}</code>
            </pre>
        </div>
    );
}

// ---------------------------------------------------------------------------
// JSON Tree View
// ---------------------------------------------------------------------------

function JsonValue({ value, depth = 0 }: { value: unknown; depth?: number }) {
    const [open, setOpen] = useState(depth < 2);

    if (value === null) return <span className="text-[#888]">null</span>;
    if (value === undefined) return <span className="text-[#888]">undefined</span>;
    if (typeof value === 'boolean') return <span className="text-[#c586c0]">{String(value)}</span>;
    if (typeof value === 'number') return <span className="text-[#b5cea8]">{String(value)}</span>;
    if (typeof value === 'string') {
        if (value.length > 200) {
            return <span className="text-[#ce9178]">"{value.slice(0, 200)}..."</span>;
        }
        return <span className="text-[#ce9178]">"{value}"</span>;
    }

    if (Array.isArray(value)) {
        if (value.length === 0) return <span className="text-[#888]">[]</span>;
        return (
            <span>
                <button
                    className="inline-flex items-center text-[#888] hover:text-white"
                    onClick={() => setOpen(!open)}
                >
                    {open ? <ChevronDown size={10} /> : <ChevronRight size={10} />}
                    <span className="text-[#888] ml-0.5">[{value.length}]</span>
                </button>
                {open && (
                    <div className="ml-3 border-l border-[#333] pl-2">
                        {value.map((item, i) => (
                            <div key={i} className="flex items-start gap-1">
                                <span className="text-[#888] shrink-0 select-none">{i}:</span>
                                <JsonValue value={item} depth={depth + 1} />
                            </div>
                        ))}
                    </div>
                )}
            </span>
        );
    }

    if (typeof value === 'object') {
        const entries = Object.entries(value as Record<string, unknown>);
        if (entries.length === 0) return <span className="text-[#888]">{'{}'}</span>;
        return (
            <span>
                <button
                    className="inline-flex items-center text-[#888] hover:text-white"
                    onClick={() => setOpen(!open)}
                >
                    {open ? <ChevronDown size={10} /> : <ChevronRight size={10} />}
                    <span className="text-[#888] ml-0.5">{'{'}...{'}'}</span>
                </button>
                {open && (
                    <div className="ml-3 border-l border-[#333] pl-2">
                        {entries.map(([key, val]) => (
                            <div key={key} className="flex items-start gap-1">
                                <span className="text-[#9cdcfe] shrink-0">"{key}":</span>
                                <JsonValue value={val} depth={depth + 1} />
                            </div>
                        ))}
                    </div>
                )}
            </span>
        );
    }

    return <span>{String(value)}</span>;
}

function JsonTreeView({ content }: { content: string }) {
    const parsed = useMemo(() => {
        try { return JSON.parse(content); } catch { return null; }
    }, [content]);

    if (parsed === null) return <pre className="font-mono text-xs whitespace-pre-wrap">{content}</pre>;

    return (
        <div className="font-mono text-[11px] leading-relaxed">
            <JsonValue value={parsed} />
        </div>
    );
}

// ---------------------------------------------------------------------------
// JSON Array Table
// ---------------------------------------------------------------------------

type SortDir = 'asc' | 'desc' | null;

function JsonArrayTable({ content }: { content: string }) {
    const data = useMemo<Record<string, unknown>[]>(() => {
        try { return JSON.parse(content); } catch { return []; }
    }, [content]);

    const columns = useMemo(() => {
        if (data.length === 0) return [];
        const keys = new Set<string>();
        for (const row of data) {
            for (const key of Object.keys(row)) keys.add(key);
        }
        return Array.from(keys);
    }, [data]);

    const [sortCol, setSortCol] = useState<string | null>(null);
    const [sortDir, setSortDir] = useState<SortDir>(null);

    const sorted = useMemo(() => {
        if (!sortCol || !sortDir) return data;
        return [...data].sort((a, b) => {
            const va = a[sortCol];
            const vb = b[sortCol];
            if (va == null && vb == null) return 0;
            if (va == null) return 1;
            if (vb == null) return -1;
            if (typeof va === 'number' && typeof vb === 'number') {
                return sortDir === 'asc' ? va - vb : vb - va;
            }
            const sa = String(va);
            const sb = String(vb);
            return sortDir === 'asc' ? sa.localeCompare(sb) : sb.localeCompare(sa);
        });
    }, [data, sortCol, sortDir]);

    const handleSort = (col: string) => {
        if (sortCol === col) {
            setSortDir(sortDir === 'asc' ? 'desc' : sortDir === 'desc' ? null : 'asc');
            if (sortDir === 'desc') setSortCol(null);
        } else {
            setSortCol(col);
            setSortDir('asc');
        }
    };

    const handleCopyTsv = useCallback(async () => {
        const header = columns.join('\t');
        const rows = sorted.map(row => columns.map(c => {
            const v = row[c];
            return v == null ? '' : typeof v === 'object' ? JSON.stringify(v) : String(v);
        }).join('\t'));
        await copyText([header, ...rows].join('\n'));
    }, [columns, sorted]);

    if (data.length === 0 || columns.length === 0) return null;

    const SortIcon = ({ col }: { col: string }) => {
        if (sortCol !== col) return <ArrowUpDown size={10} className="opacity-30" />;
        if (sortDir === 'asc') return <ArrowUp size={10} className="text-blue-400" />;
        return <ArrowDown size={10} className="text-blue-400" />;
    };

    const handleCopyCsv = useCallback(async () => {
        const escCsv = (v: string) => v.includes(',') || v.includes('"') || v.includes('\n') ? `"${v.replace(/"/g, '""')}"` : v;
        const header = columns.map(escCsv).join(',');
        const rows = sorted.map(row => columns.map(c => {
            const v = row[c];
            return v == null ? '' : escCsv(typeof v === 'object' ? JSON.stringify(v) : String(v));
        }).join(','));
        await copyText([header, ...rows].join('\n'));
    }, [columns, sorted]);

    return (
        <div>
            <div className="flex items-center justify-between mb-1">
                <span className="text-[10px] text-[var(--text-muted)]">{data.length} rows</span>
                <div className="flex items-center gap-1">
                    <button
                        className="text-[10px] text-[var(--text-muted)] hover:text-[var(--text-primary)] flex items-center gap-0.5"
                        onClick={handleCopyCsv}
                        title="Copy as CSV"
                    >
                        <Copy size={10} /> CSV
                    </button>
                    <button
                        className="text-[10px] text-[var(--text-muted)] hover:text-[var(--text-primary)] flex items-center gap-0.5"
                        onClick={handleCopyTsv}
                        title="Copy as TSV"
                    >
                        <Copy size={10} /> TSV
                    </button>
                </div>
            </div>
            <div className="overflow-x-auto rounded border border-[#333]">
                <table className="w-full text-[11px] font-mono">
                    <thead>
                        <tr className="bg-[#1e1e1e]">
                            {columns.map(col => (
                                <th
                                    key={col}
                                    className="px-2 py-1 text-left text-[var(--text-muted)] font-medium cursor-pointer hover:text-[var(--text-primary)] select-none whitespace-nowrap border-b border-[#333]"
                                    onClick={() => handleSort(col)}
                                >
                                    <span className="flex items-center gap-1">
                                        {col} <SortIcon col={col} />
                                    </span>
                                </th>
                            ))}
                        </tr>
                    </thead>
                    <tbody>
                        {sorted.map((row, i) => (
                            <tr key={i} className="border-b border-[#2a2a2a] hover:bg-[#1e1e1e]">
                                {columns.map(col => {
                                    const val = row[col];
                                    const display = val == null ? '' : typeof val === 'object' ? JSON.stringify(val) : String(val);
                                    return (
                                        <td key={col} className="px-2 py-0.5 whitespace-nowrap max-w-[200px] truncate" title={display}>
                                            {display}
                                        </td>
                                    );
                                })}
                            </tr>
                        ))}
                    </tbody>
                </table>
            </div>
        </div>
    );
}

// ---------------------------------------------------------------------------
// Mode switcher tabs
// ---------------------------------------------------------------------------

const modeIcons: Record<string, React.ElementType> = {
    'markdown': FileText,
    'table': TableIcon,
    'json': Braces,
    'text': FileText,
};

function ModeSwitcher({ mode, available, onSwitch }: {
    mode: string;
    available: string[];
    onSwitch: (m: string) => void;
}) {
    if (available.length <= 1) return null;
    return (
        <div className="flex items-center gap-0.5 mb-1">
            {available.map(m => {
                const Icon = modeIcons[m] || FileText;
                return (
                    <button
                        key={m}
                        className={`px-1.5 py-0.5 rounded text-[10px] flex items-center gap-0.5 ${
                            m === mode
                                ? 'bg-[#333] text-[var(--text-primary)]'
                                : 'text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:bg-[#2a2a2a]'
                        }`}
                        onClick={() => onSwitch(m)}
                    >
                        <Icon size={10} />
                        {m.charAt(0).toUpperCase() + m.slice(1)}
                    </button>
                );
            })}
        </div>
    );
}

// ---------------------------------------------------------------------------
// Main RichOutput component
// ---------------------------------------------------------------------------

interface RichOutputProps {
    content: string;
    maxHeight?: number;
    className?: string;
    compact?: boolean;
    forceMode?: string;
}

export function RichOutput({ content, maxHeight, className = '', compact = false, forceMode }: RichOutputProps) {
    const [expanded, setExpanded] = useState(false);

    const detected = useMemo(() => detectContentType(content), [content]);

    const availableModes = useMemo(() => {
        const modes: string[] = [];
        if (detected === 'json-array') {
            modes.push('table', 'json', 'text');
        } else if (detected === 'json-object') {
            modes.push('json', 'text');
        } else if (detected === 'markdown') {
            modes.push('markdown', 'text');
        } else {
            modes.push('text');
        }
        return modes;
    }, [detected]);

    const defaultMode = forceMode || availableModes[0];
    const [mode, setMode] = useState(defaultMode);

    // Reset mode if content type changes
    const activeMode = availableModes.includes(mode) ? mode : availableModes[0];

    const isLong = content.length > 300;
    const effectiveMaxHeight = expanded ? undefined : maxHeight;

    return (
        <div className={`rich-output-container relative group ${className}`}>
            {/* Toolbar */}
            {!compact && (
                <div className="flex items-center gap-1 mb-1 opacity-0 group-hover:opacity-100 transition-opacity absolute top-1 right-1 z-10">
                    {isLong && (
                        <button
                            className="p-0.5 rounded bg-[#2a2a2a] hover:bg-[#3a3a3a] text-[var(--text-muted)] hover:text-[var(--text-primary)]"
                            onClick={() => setExpanded(!expanded)}
                            title={expanded ? 'Collapse' : 'Expand'}
                        >
                            {expanded ? <Minimize2 size={12} /> : <Maximize2 size={12} />}
                        </button>
                    )}
                    <CopyButton text={content} />
                </div>
            )}

            {/* Mode tabs */}
            {!compact && <ModeSwitcher mode={activeMode} available={availableModes} onSwitch={setMode} />}

            {/* Content */}
            <div
                className={`rich-output-content bg-[var(--bg-primary)] rounded p-2 overflow-y-auto ${compact ? 'text-[10px]' : 'text-xs'}`}
                style={{ maxHeight: effectiveMaxHeight ? `${effectiveMaxHeight}px` : undefined }}
            >
                {activeMode === 'table' && <JsonArrayTable content={content} />}
                {activeMode === 'json' && <JsonTreeView content={content} />}
                {activeMode === 'markdown' && <MarkdownRenderer content={content} />}
                {activeMode === 'text' && (
                    <pre className="font-mono whitespace-pre-wrap break-words leading-relaxed">{content}</pre>
                )}
            </div>
        </div>
    );
}

// ---------------------------------------------------------------------------
// Markdown renderer (extracted for reuse)
// ---------------------------------------------------------------------------

function MarkdownRenderer({ content }: { content: string }) {
    return (
        <ReactMarkdown
            remarkPlugins={[remarkGfm]}
            components={{
                p: ({ children }) => <p className="mb-1.5 last:mb-0">{children}</p>,
                ul: ({ children }) => <ul className="list-disc pl-4 mb-1.5">{children}</ul>,
                ol: ({ children }) => <ol className="list-decimal pl-4 mb-1.5">{children}</ol>,
                li: ({ children }) => <li className="mb-0.5">{children}</li>,
                strong: ({ children }) => <strong className="font-bold text-[var(--text-primary)]">{children}</strong>,
                em: ({ children }) => <em className="italic">{children}</em>,
                del: ({ children }) => <del className="line-through text-[var(--text-muted)]">{children}</del>,
                a: ({ href, children }) => (
                    <a href={href} className="text-blue-400 hover:underline" target="_blank" rel="noopener noreferrer">
                        {children}
                    </a>
                ),
                // Inline code
                code: ({ className, children, ...props }) => {
                    const isBlock = className?.startsWith('language-');
                    if (isBlock) {
                        return <CodeBlock className={className}>{children}</CodeBlock>;
                    }
                    return (
                        <code className="bg-[#2a2a2a] px-1 py-0.5 rounded text-[#e8c84a] font-mono text-[0.9em]" {...props}>
                            {children}
                        </code>
                    );
                },
                pre: ({ children }) => <>{children}</>,
                h1: ({ children }) => <h1 className="text-base font-bold mb-1 text-[var(--text-primary)]">{children}</h1>,
                h2: ({ children }) => <h2 className="text-sm font-bold mb-1 text-[var(--text-primary)]">{children}</h2>,
                h3: ({ children }) => <h3 className="text-xs font-bold mb-1 text-[var(--text-primary)]">{children}</h3>,
                h4: ({ children }) => <h4 className="text-xs font-semibold mb-1 text-[var(--text-primary)]">{children}</h4>,
                blockquote: ({ children }) => (
                    <blockquote className="border-l-2 border-[#555] pl-2 my-1.5 text-[var(--text-muted)] italic">{children}</blockquote>
                ),
                hr: () => <hr className="border-[#333] my-2" />,
                // GFM table
                table: ({ children }) => (
                    <div className="overflow-x-auto my-1.5 rounded border border-[#333]">
                        <table className="w-full text-[11px]">{children}</table>
                    </div>
                ),
                thead: ({ children }) => <thead className="bg-[#1e1e1e]">{children}</thead>,
                th: ({ children }) => (
                    <th className="px-2 py-1 text-left text-[var(--text-muted)] font-medium border-b border-[#333] whitespace-nowrap">
                        {children}
                    </th>
                ),
                td: ({ children }) => (
                    <td className="px-2 py-0.5 border-b border-[#2a2a2a]">{children}</td>
                ),
                tr: ({ children }) => <tr className="hover:bg-[#1e1e1e]">{children}</tr>,
                // GFM task list
                input: ({ checked, ...props }) => (
                    <input
                        type="checkbox"
                        checked={checked}
                        readOnly
                        className="mr-1 accent-blue-500"
                        {...props}
                    />
                ),
            }}
        >
            {content}
        </ReactMarkdown>
    );
}
