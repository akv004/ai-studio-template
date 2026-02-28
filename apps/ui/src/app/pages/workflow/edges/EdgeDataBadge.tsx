import { useState, useRef, useEffect } from 'react';
import { Copy, Check } from 'lucide-react';
import { useAppStore } from '../../../../state/store';
import { resolveHandleValue, formatPreview, formatFullPreview, formatDetailPreview, getDataTypeLabel } from './edgeDataUtils';

interface EdgeDataBadgeProps {
    sourceNodeId: string;
    sourceHandle: string;
    labelX: number;
    labelY: number;
}

export function EdgeDataBadge({ sourceNodeId, sourceHandle, labelX, labelY }: EdgeDataBadgeProps) {
    const outputs = useAppStore(s => s.lastRunNodeOutputs);
    const xray = useAppStore(s => s.xrayEnabled);
    const [showPopover, setShowPopover] = useState(false);
    const [copied, setCopied] = useState(false);
    const popoverRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        if (!showPopover) return;
        const handler = (e: MouseEvent) => {
            if (popoverRef.current && !popoverRef.current.contains(e.target as HTMLElement)) {
                setShowPopover(false);
            }
        };
        document.addEventListener('mousedown', handler);
        return () => document.removeEventListener('mousedown', handler);
    }, [showPopover]);

    if (!xray || !outputs[sourceNodeId]) return null;

    const nodeOutput = outputs[sourceNodeId];
    const value = resolveHandleValue(nodeOutput, sourceHandle);
    if (value === undefined) return null;

    const preview = formatPreview(value, 40);
    const tooltip = formatFullPreview(value, 500);
    const isEmpty = value == null || (typeof value === 'string' && value.length === 0);

    const handleCopy = async () => {
        const text = formatDetailPreview(value, 50000);
        try {
            await navigator.clipboard.writeText(text);
            setCopied(true);
            setTimeout(() => setCopied(false), 1500);
        } catch {
            // Fallback
        }
    };

    return (
        <div
            style={{
                position: 'absolute',
                transform: `translate(-50%, -50%) translate(${labelX}px, ${labelY}px)`,
                pointerEvents: 'all',
            }}
            className="nopan nodrag"
        >
            <div
                className={`
                    bg-[var(--bg-secondary)]/90 border border-[var(--border-subtle)] rounded-full
                    px-2 py-0.5 max-w-[200px] truncate cursor-pointer
                    text-[10px] font-mono select-none
                    hover:bg-[var(--bg-tertiary)] hover:border-[var(--text-muted)] transition-colors
                    ${isEmpty ? 'text-[var(--text-muted)]/50 italic' : 'text-[var(--text-muted)]'}
                `}
                title={tooltip}
                onClick={(e) => {
                    e.stopPropagation();
                    setShowPopover(!showPopover);
                }}
            >
                {preview}
            </div>

            {showPopover && (
                <div
                    ref={popoverRef}
                    className="absolute left-1/2 -translate-x-1/2 top-full mt-1 z-50
                        bg-[var(--bg-primary)] border border-[var(--border-subtle)] rounded-lg shadow-lg
                        w-[360px] max-h-[300px] overflow-hidden"
                    onClick={(e) => e.stopPropagation()}
                >
                    <div className="flex items-center justify-between px-3 py-1.5 border-b border-[var(--border-subtle)]">
                        <span className="text-[10px] text-[var(--text-muted)]">
                            {getDataTypeLabel(value)}
                        </span>
                        <button
                            className="p-1 rounded hover:bg-[var(--bg-tertiary)] text-[var(--text-muted)] hover:text-[var(--text-primary)] transition-colors"
                            onClick={handleCopy}
                            title="Copy to clipboard"
                        >
                            {copied ? <Check size={12} className="text-green-400" /> : <Copy size={12} />}
                        </button>
                    </div>
                    <pre className="px-3 py-2 text-[11px] font-mono text-[var(--text-secondary)] overflow-auto max-h-[250px] whitespace-pre-wrap break-words">
                        {formatDetailPreview(value, 5000)}
                    </pre>
                </div>
            )}
        </div>
    );
}
