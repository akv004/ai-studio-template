import { Handle, Position } from '@xyflow/react';
import { Webhook, Copy, Circle } from 'lucide-react';
import { NodeShell, OutputPreview } from './NodeShell';
import { useNodeData } from '../hooks/useNodeData';

const AUTH_LABELS: Record<string, string> = {
    none: '',
    token: 'Bearer',
    hmac: 'HMAC',
};

export function WebhookTriggerNode({ id, data, selected }: { id: string; data: Record<string, unknown>; selected?: boolean }) {
    const { updateField } = useNodeData(id);

    const path = (data.path as string) || '';
    const methods = (data.methods as string[]) || ['POST'];
    const authMode = (data.authMode as string) || 'none';
    const armed = (data._armed as boolean) || false;
    const webhookPort = (data._webhookPort as number) || 9876;

    const webhookUrl = path ? `http://localhost:${webhookPort}/hook/${path.replace(/^\//, '')}` : '';

    const handleCopyUrl = async (e: React.MouseEvent) => {
        e.stopPropagation();
        if (!webhookUrl) return;
        try {
            await navigator.clipboard.writeText(webhookUrl);
        } catch {
            // ignore
        }
    };

    return (
        <NodeShell id={id} type="webhook_trigger" label="WEBHOOK" icon={Webhook} selected={selected}
            collapsed={data.collapsed as boolean} customLabel={(data.label as string) || ''}>

            <div className="flex flex-col gap-1" onClick={e => e.stopPropagation()}>
                {/* Status + path */}
                <div className="flex items-center gap-1.5">
                    <Circle
                        size={8}
                        fill={armed ? '#22c55e' : '#666'}
                        stroke={armed ? '#22c55e' : '#666'}
                    />
                    <span className="text-[10px] text-[var(--text-muted)]">
                        {armed ? 'Armed' : 'Disarmed'}
                    </span>
                </div>

                {/* Path input */}
                <input
                    className="node-inline-input"
                    value={path}
                    placeholder="/my-webhook"
                    onChange={e => updateField('path', e.target.value)}
                    onMouseDown={e => e.stopPropagation()}
                />

                {/* Method badges */}
                <div className="flex items-center gap-1 flex-wrap">
                    {methods.map(m => (
                        <span key={m} className="px-1 py-0 rounded text-[9px] font-mono font-medium bg-[#3a3a3a] text-[#ccc]">
                            {m}
                        </span>
                    ))}
                    {AUTH_LABELS[authMode] && (
                        <span className="px-1 py-0 rounded text-[9px] font-mono bg-yellow-900/50 text-yellow-300">
                            {AUTH_LABELS[authMode]}
                        </span>
                    )}
                </div>

                {/* Webhook URL (if path set) */}
                {webhookUrl && (
                    <div
                        className="flex items-center gap-1 mt-0.5 px-1 py-0.5 rounded bg-[#1e1e1e] border border-[#3a3a3a] cursor-pointer hover:border-[#555] group"
                        onClick={handleCopyUrl}
                        title="Click to copy URL"
                    >
                        <span className="text-[9px] font-mono text-[#888] truncate flex-1 max-w-[160px]">
                            {webhookUrl}
                        </span>
                        <Copy size={9} className="text-[#666] group-hover:text-[#aaa] shrink-0" />
                    </div>
                )}
            </div>

            <OutputPreview nodeId={id} />

            {/* Source handles only — webhook is an entry point */}
            <div className="handle-row output">
                <span className="handle-label">body</span>
                <Handle type="source" position={Position.Right} id="body"
                    className="custom-handle handle-json" title="json" />
            </div>
            <div className="handle-row output">
                <span className="handle-label">headers</span>
                <Handle type="source" position={Position.Right} id="headers"
                    className="custom-handle handle-json" title="json" />
            </div>
            <div className="handle-row output">
                <span className="handle-label">query</span>
                <Handle type="source" position={Position.Right} id="query"
                    className="custom-handle handle-json" title="json" />
            </div>
            <div className="handle-row output">
                <span className="handle-label">method</span>
                <Handle type="source" position={Position.Right} id="method"
                    className="custom-handle handle-text" title="text" />
            </div>
        </NodeShell>
    );
}
