import { ChevronDown, ChevronRight, Loader2, Check, X, Clock } from 'lucide-react';
import { useAppStore } from '../../../../state/store';
import { nodeColors } from '../nodeColors';
import type { NodeExecutionStatus } from '@ai-studio/shared';

const execBadgeConfig: Record<NodeExecutionStatus, { icon: React.ElementType | null; label: string }> = {
    idle: { icon: null, label: '' },
    running: { icon: Loader2, label: 'Running' },
    completed: { icon: Check, label: 'Done' },
    error: { icon: X, label: 'Error' },
    waiting: { icon: Clock, label: 'Waiting' },
    skipped: { icon: null, label: 'Skipped' },
};

export function ExecutionBadge({ nodeId }: { nodeId: string }) {
    const state = useAppStore((s) => s.workflowNodeStates[nodeId]);
    if (!state || state.status === 'idle') return null;
    const cfg = execBadgeConfig[state.status];
    const Icon = cfg.icon;
    return (
        <div className="absolute -top-2 -right-2 flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] font-medium bg-[#1e1e1e] border border-[#3a3a3a] z-10">
            {Icon && <Icon size={10} className={state.status === 'running' ? 'animate-spin' : ''} />}
            {cfg.label}
        </div>
    );
}

export function OutputPreview({ nodeId }: { nodeId: string }) {
    const state = useAppStore((s) => s.workflowNodeStates[nodeId]);
    if (!state || state.status !== 'completed' || !state.output) return null;
    return (
        <div
            className="mt-1 text-[10px] text-[#999] max-w-[180px] max-h-[80px] overflow-y-auto font-mono leading-tight whitespace-pre-wrap break-words cursor-default"
            title={state.output.slice(0, 1000)}
            onMouseDown={e => e.stopPropagation()}
        >
            {state.output}
        </div>
    );
}

export function useExecClass(nodeId: string): string {
    const state = useAppStore((s) => s.workflowNodeStates[nodeId]);
    if (!state || state.status === 'idle') return '';
    return `exec-${state.status}`;
}

export function NodeShell({ id, type, label, icon: Icon, selected, collapsed, onToggleCollapse, children }: {
    id: string; type: string; label: string; icon: React.ElementType;
    selected?: boolean; collapsed?: boolean; onToggleCollapse?: () => void;
    children: React.ReactNode;
}) {
    const execClass = useExecClass(id);
    return (
        <div className={`custom-node ${selected ? 'selected' : ''} ${collapsed ? 'collapsed' : ''} ${execClass} relative`}>
            <ExecutionBadge nodeId={id} />
            <div className="custom-node-header" style={{ background: nodeColors[type] }}
                onClick={onToggleCollapse}>
                <span className="collapse-chevron">
                    {collapsed ? <ChevronRight size={8} /> : <ChevronDown size={8} />}
                </span>
                <Icon size={12} />
                {label}
            </div>
            <div className="custom-node-body">
                {children}
            </div>
        </div>
    );
}
