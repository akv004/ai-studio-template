import { StickyNote } from 'lucide-react';
import { nodeColors } from '../nodeColors';

export function NoteNode({ data, selected }: { id: string; data: Record<string, unknown>; selected?: boolean }) {
    const content = (data.content as string) || '';
    const preview = content.slice(0, 200) || 'Double-click to add notes...';

    return (
        <div className={`custom-node note-node ${selected ? 'selected' : ''}`}
            style={{ minWidth: 220, maxWidth: 300 }}>
            <div className="custom-node-header" style={{ background: nodeColors['note'] || '#6a6a3a' }}>
                <StickyNote size={12} />
                NOTE
                {(data.label as string) && (
                    <span className="opacity-80 font-normal"> · {data.label as string}</span>
                )}
            </div>
            <div className="custom-node-body">
                <div
                    className="text-[11px] text-[var(--text-secondary)] leading-relaxed whitespace-pre-wrap break-words"
                    style={{ maxHeight: 120, overflow: 'hidden' }}
                >
                    {preview}
                    {content.length > 200 && <span className="text-[var(--text-muted)]">...</span>}
                </div>
            </div>
        </div>
    );
}
