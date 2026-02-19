import { useState, useCallback } from 'react';
import ReactMarkdown from 'react-markdown';
import { Copy, Check, Maximize2, Minimize2 } from 'lucide-react';

interface RichOutputProps {
    content: string;
    maxHeight?: number;
    className?: string;
    compact?: boolean;
}

export function RichOutput({ content, maxHeight, className = '', compact = false }: RichOutputProps) {
    const [copied, setCopied] = useState(false);
    const [expanded, setExpanded] = useState(false);

    const handleCopy = useCallback(async () => {
        try {
            await navigator.clipboard.writeText(content);
            setCopied(true);
            setTimeout(() => setCopied(false), 2000);
        } catch {
            // Fallback for non-secure contexts
            const ta = document.createElement('textarea');
            ta.value = content;
            document.body.appendChild(ta);
            ta.select();
            document.execCommand('copy');
            document.body.removeChild(ta);
            setCopied(true);
            setTimeout(() => setCopied(false), 2000);
        }
    }, [content]);

    const isLong = content.length > 300;
    const effectiveMaxHeight = expanded ? undefined : maxHeight;

    return (
        <div className={`rich-output-container relative group ${className}`}>
            {/* Toolbar */}
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
                <button
                    className="p-0.5 rounded bg-[#2a2a2a] hover:bg-[#3a3a3a] text-[var(--text-muted)] hover:text-[var(--text-primary)]"
                    onClick={handleCopy}
                    title="Copy to clipboard"
                >
                    {copied ? <Check size={12} className="text-green-400" /> : <Copy size={12} />}
                </button>
            </div>

            {/* Content */}
            <div
                className={`rich-output-content bg-[var(--bg-primary)] rounded p-2 overflow-y-auto ${compact ? 'text-[10px]' : 'text-xs'}`}
                style={{ maxHeight: effectiveMaxHeight ? `${effectiveMaxHeight}px` : undefined }}
            >
                <ReactMarkdown
                    components={{
                        p: ({ children }) => <p className="mb-1.5 last:mb-0">{children}</p>,
                        ul: ({ children }) => <ul className="list-disc pl-4 mb-1.5">{children}</ul>,
                        ol: ({ children }) => <ol className="list-decimal pl-4 mb-1.5">{children}</ol>,
                        li: ({ children }) => <li className="mb-0.5">{children}</li>,
                        strong: ({ children }) => <strong className="font-bold text-[var(--text-primary)]">{children}</strong>,
                        em: ({ children }) => <em className="italic">{children}</em>,
                        code: ({ children }) => (
                            <code className="bg-[#2a2a2a] px-1 py-0.5 rounded text-[#e8c84a] font-mono text-[0.9em]">{children}</code>
                        ),
                        pre: ({ children }) => (
                            <pre className="bg-[#1a1a1a] p-2 rounded my-1.5 overflow-x-auto font-mono text-[0.9em]">{children}</pre>
                        ),
                        h1: ({ children }) => <h1 className="text-base font-bold mb-1 text-[var(--text-primary)]">{children}</h1>,
                        h2: ({ children }) => <h2 className="text-sm font-bold mb-1 text-[var(--text-primary)]">{children}</h2>,
                        h3: ({ children }) => <h3 className="text-xs font-bold mb-1 text-[var(--text-primary)]">{children}</h3>,
                        blockquote: ({ children }) => (
                            <blockquote className="border-l-2 border-[#555] pl-2 my-1.5 text-[var(--text-muted)] italic">{children}</blockquote>
                        ),
                        hr: () => <hr className="border-[#333] my-2" />,
                    }}
                >
                    {content}
                </ReactMarkdown>
            </div>
        </div>
    );
}
