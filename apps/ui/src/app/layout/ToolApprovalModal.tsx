type ToolApprovalRequest = {
  id: string;
  method: string;
  path: string;
  body?: unknown;
};

interface ToolApprovalModalProps {
  request: ToolApprovalRequest;
  onApprove: () => void;
  onDeny: () => void;
}

export function ToolApprovalModal({ request, onApprove, onDeny }: ToolApprovalModalProps) {
  return (
    <div className="modal-overlay" role="dialog" aria-modal="true">
      <div className="modal">
        <div className="modal-header">
          <div className="modal-title">Approve tool request</div>
          <div className="text-xs text-[var(--text-muted)]">
            {request.method} {request.path}
          </div>
        </div>

        <div className="modal-body">
          <div className="text-sm text-[var(--text-secondary)]">
            The agent is requesting permission to use a local tool.
          </div>

          {typeof request.body !== 'undefined' && (
            <pre className="modal-pre">
              {typeof request.body === 'string'
                ? request.body
                : JSON.stringify(request.body, null, 2)}
            </pre>
          )}
        </div>

        <div className="modal-footer">
          <button className="btn btn-secondary" onClick={onDeny}>
            Deny
          </button>
          <button className="btn btn-primary" onClick={onApprove}>
            Approve
          </button>
        </div>
      </div>
    </div>
  );
}

