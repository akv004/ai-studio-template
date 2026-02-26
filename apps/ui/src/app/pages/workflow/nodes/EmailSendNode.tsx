import { Handle, Position } from '@xyflow/react';
import { Mail } from 'lucide-react';
import { NodeShell, OutputPreview } from './NodeShell';

export function EmailSendNode({ id, data, selected }: { id: string; data: Record<string, unknown>; selected?: boolean }) {
    return (
        <NodeShell id={id} type="email_send" label="EMAIL SEND" icon={Mail} selected={selected}
            collapsed={data.collapsed as boolean} customLabel={(data.label as string) || ''}>
            <div className="handle-row input">
                <Handle type="target" position={Position.Left} id="to"
                    className="custom-handle handle-text" title="text" />
                <span className="handle-label">to</span>
            </div>
            <div className="handle-row input">
                <Handle type="target" position={Position.Left} id="subject"
                    className="custom-handle handle-text" title="text" />
                <span className="handle-label">subject</span>
            </div>
            <div className="handle-row input">
                <Handle type="target" position={Position.Left} id="body"
                    className="custom-handle handle-text" title="text" />
                <span className="handle-label">body</span>
            </div>
            <div className="handle-row input">
                <Handle type="target" position={Position.Left} id="cc"
                    className="custom-handle handle-text" title="text" />
                <span className="handle-label">cc</span>
            </div>
            <div className="handle-row input">
                <Handle type="target" position={Position.Left} id="bcc"
                    className="custom-handle handle-text" title="text" />
                <span className="handle-label">bcc</span>
            </div>
            <div className="handle-row input">
                <Handle type="target" position={Position.Left} id="replyTo"
                    className="custom-handle handle-text" title="text" />
                <span className="handle-label">replyTo</span>
            </div>

            {((data.smtpHost as string) || (data.fromAddress as string)) && (
                <div className="px-2 py-1 text-[10px] text-[var(--text-muted)] space-y-0.5">
                    {(data.smtpHost as string) && (
                        <div className="truncate">SMTP: {data.smtpHost as string}</div>
                    )}
                    {(data.fromAddress as string) && (
                        <div className="truncate">From: {data.fromAddress as string}</div>
                    )}
                </div>
            )}

            <OutputPreview nodeId={id} />

            <div className="handle-row output">
                <span className="handle-label">output</span>
                <Handle type="source" position={Position.Right} id="output"
                    className="custom-handle handle-json" title="json" />
            </div>
            <div className="handle-row output">
                <span className="handle-label">error</span>
                <Handle type="source" position={Position.Right} id="error"
                    className="custom-handle handle-text" title="text" />
            </div>
        </NodeShell>
    );
}
