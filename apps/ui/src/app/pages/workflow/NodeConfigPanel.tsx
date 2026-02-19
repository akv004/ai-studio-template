import { Plus, Trash2, Check, X } from 'lucide-react';
import type { Node } from '@xyflow/react';
import { useAppStore } from '../../../state/store';
import { PROVIDER_MODELS } from './nodeColors';

export function NodeConfigPanel({ node, onChange, onDelete }: {
    node: Node;
    onChange: (data: Record<string, unknown>) => void;
    onDelete: () => void;
}) {
    const nodeState = useAppStore((s) => s.workflowNodeStates[node.id]);
    const data = node.data as Record<string, unknown>;
    const type = node.type || 'input';

    const update = (field: string, value: unknown) => {
        onChange({ ...data, [field]: value });
    };

    // Auto-correct model if it doesn't belong to the selected provider
    const provider = (data.provider as string) || '';
    const currentModel = (data.model as string) || '';
    const validModels = PROVIDER_MODELS[provider] || [];
    if (type === 'llm' && provider && currentModel && validModels.length > 0 && !validModels.includes(currentModel)) {
        onChange({ ...data, model: validModels[0] });
    }

    return (
        <div className="p-4 space-y-3">
            <h3 className="text-sm font-semibold uppercase text-[var(--text-muted)]">
                {type} Configuration
            </h3>

            {/* Common: name field for input/output */}
            {(type === 'input' || type === 'output') && (
                <label className="block">
                    <span className="text-xs text-[var(--text-muted)]">Name</span>
                    <input
                        className="config-input"
                        value={(data.name as string) || ''}
                        onChange={(e) => update('name', e.target.value)}
                    />
                </label>
            )}

            {type === 'input' && (
                <label className="block">
                    <span className="text-xs text-[var(--text-muted)]">Data Type</span>
                    <select className="config-input" value={(data.dataType as string) || 'text'}
                        onChange={(e) => update('dataType', e.target.value)}>
                        <option value="text">Text</option>
                        <option value="json">JSON</option>
                        <option value="boolean">Boolean</option>
                        <option value="file">File</option>
                    </select>
                </label>
            )}

            {type === 'output' && (
                <label className="block">
                    <span className="text-xs text-[var(--text-muted)]">Format</span>
                    <select className="config-input" value={(data.format as string) || 'text'}
                        onChange={(e) => update('format', e.target.value)}>
                        <option value="text">Text</option>
                        <option value="markdown">Markdown</option>
                        <option value="json">JSON</option>
                    </select>
                </label>
            )}

            {type === 'llm' && (
                <>
                    <label className="block">
                        <span className="text-xs text-[var(--text-muted)]">Provider</span>
                        <select className="config-input" value={(data.provider as string) || ''}
                            onChange={(e) => {
                                const p = e.target.value;
                                const models = PROVIDER_MODELS[p] || [];
                                onChange({ ...data, provider: p, model: models[0] || '' });
                            }}>
                            <option value="" disabled>Select provider...</option>
                            <option value="anthropic">Anthropic</option>
                            <option value="google">Google</option>
                            <option value="azure_openai">Azure OpenAI</option>
                            <option value="ollama">Ollama (local)</option>
                            <option value="local">Local (OpenAI-Compatible)</option>
                        </select>
                    </label>
                    <label className="block">
                        <span className="text-xs text-[var(--text-muted)]">Model</span>
                        <select className="config-input" value={(data.model as string) || ''}
                            onChange={(e) => update('model', e.target.value)}>
                            <option value="" disabled>Select model...</option>
                            {validModels.map((m) => (
                                <option key={m} value={m}>{m}</option>
                            ))}
                        </select>
                    </label>
                    <label className="block">
                        <span className="text-xs text-[var(--text-muted)]">System Prompt</span>
                        <textarea className="config-input min-h-[80px]" value={(data.systemPrompt as string) || ''}
                            onChange={(e) => update('systemPrompt', e.target.value)} />
                    </label>
                    <label className="block">
                        <span className="text-xs text-[var(--text-muted)]">Temperature</span>
                        <input type="number" step="0.1" min="0" max="2" className="config-input"
                            value={(data.temperature as number) ?? 0.7}
                            onChange={(e) => update('temperature', parseFloat(e.target.value))} />
                    </label>
                </>
            )}

            {type === 'tool' && (
                <>
                    <label className="block">
                        <span className="text-xs text-[var(--text-muted)]">Tool Name</span>
                        <input className="config-input" value={(data.toolName as string) || ''}
                            onChange={(e) => update('toolName', e.target.value)}
                            placeholder="e.g. builtin__shell" />
                    </label>
                    <label className="block">
                        <span className="text-xs text-[var(--text-muted)]">Approval</span>
                        <select className="config-input" value={(data.approval as string) || 'auto'}
                            onChange={(e) => update('approval', e.target.value)}>
                            <option value="auto">Auto</option>
                            <option value="ask">Ask</option>
                            <option value="deny">Deny</option>
                        </select>
                    </label>
                </>
            )}

            {type === 'router' && (
                <>
                    <label className="block">
                        <span className="text-xs text-[var(--text-muted)]">Mode</span>
                        <select className="config-input" value={(data.mode as string) || 'pattern'}
                            onChange={(e) => update('mode', e.target.value)}>
                            <option value="pattern">Pattern Match</option>
                            <option value="llm">LLM Classify</option>
                        </select>
                    </label>
                    <label className="block">
                        <span className="text-xs text-[var(--text-muted)]">Branches (comma-separated)</span>
                        <input className="config-input"
                            value={((data.branches as string[]) || ['true', 'false']).join(', ')}
                            onChange={(e) => update('branches', e.target.value.split(',').map(s => s.trim()).filter(Boolean))} />
                    </label>
                </>
            )}

            {type === 'approval' && (
                <label className="block">
                    <span className="text-xs text-[var(--text-muted)]">Message</span>
                    <textarea className="config-input min-h-[60px]" value={(data.message as string) || ''}
                        onChange={(e) => update('message', e.target.value)} />
                </label>
            )}

            {type === 'transform' && (
                <>
                    <label className="block">
                        <span className="text-xs text-[var(--text-muted)]">Mode</span>
                        <select className="config-input" value={(data.mode as string) || 'template'}
                            onChange={(e) => update('mode', e.target.value)}>
                            <option value="template">Template</option>
                            <option value="jsonpath">JSONPath</option>
                            <option value="script">Script</option>
                        </select>
                    </label>

                    <div className="block">
                        <span className="text-xs text-[var(--text-muted)]">Inputs</span>
                        <div className="space-y-2 mt-1">
                            {((data.inputs as string[]) || ['input']).map((input, idx) => (
                                <div key={idx} className="flex gap-2">
                                    <input className="config-input text-xs py-1"
                                        value={input}
                                        onChange={(e) => {
                                            const newInputs = [...((data.inputs as string[]) || ['input'])];
                                            newInputs[idx] = e.target.value;
                                            update('inputs', newInputs);
                                        }}
                                    />
                                    <button className="btn-icon text-[var(--text-muted)] hover:text-red-400"
                                        onClick={() => {
                                            const newInputs = [...((data.inputs as string[]) || ['input'])];
                                            newInputs.splice(idx, 1);
                                            update('inputs', newInputs);
                                        }}>
                                        <X size={12} />
                                    </button>
                                </div>
                            ))}
                            <button className="text-xs text-blue-400 hover:text-blue-300 flex items-center gap-1"
                                onClick={() => {
                                    const newInputs = [...((data.inputs as string[]) || ['input'])];
                                    newInputs.push(`input_${newInputs.length + 1}`);
                                    update('inputs', newInputs);
                                }}>
                                <Plus size={10} /> Add Input
                            </button>
                        </div>
                    </div>

                    <label className="block">
                        <span className="text-xs text-[var(--text-muted)]">Template / Expression</span>
                        <textarea className="config-input min-h-[60px] font-mono text-xs"
                            value={(data.template as string) || ''}
                            onChange={(e) => update('template', e.target.value)} />
                    </label>
                </>
            )}

            {type === 'subworkflow' && (
                <label className="block">
                    <span className="text-xs text-[var(--text-muted)]">Workflow Name</span>
                    <input className="config-input" value={(data.workflowName as string) || ''}
                        onChange={(e) => update('workflowName', e.target.value)}
                        placeholder="workflow name" />
                </label>
            )}

            {type === 'http_request' && (
                <>
                    <label className="block">
                        <span className="text-xs text-[var(--text-muted)]">URL</span>
                        <input className="config-input" value={(data.url as string) || ''}
                            onChange={(e) => update('url', e.target.value)}
                            placeholder="https://api.example.com" />
                    </label>
                    <label className="block">
                        <span className="text-xs text-[var(--text-muted)]">Method</span>
                        <select className="config-input" value={(data.method as string) || 'GET'}
                            onChange={(e) => update('method', e.target.value)}>
                            <option value="GET">GET</option>
                            <option value="POST">POST</option>
                            <option value="PUT">PUT</option>
                            <option value="PATCH">PATCH</option>
                            <option value="DELETE">DELETE</option>
                            <option value="HEAD">HEAD</option>
                        </select>
                    </label>
                    <label className="block">
                        <span className="text-xs text-[var(--text-muted)]">Timeout (seconds)</span>
                        <input type="number" className="config-input" value={(data.timeout as number) ?? 30}
                            onChange={(e) => update('timeout', parseInt(e.target.value) || 30)} />
                    </label>
                    <label className="block">
                        <span className="text-xs text-[var(--text-muted)]">Auth</span>
                        <select className="config-input" value={(data.auth as string) || 'none'}
                            onChange={(e) => update('auth', e.target.value)}>
                            <option value="none">None</option>
                            <option value="bearer">Bearer Token</option>
                            <option value="basic">Basic Auth</option>
                            <option value="api_key">API Key</option>
                        </select>
                    </label>
                    {(data.auth as string) !== 'none' && (data.auth as string) && (
                        <label className="block">
                            <span className="text-xs text-[var(--text-muted)]">Auth Token Settings Key</span>
                            <input className="config-input" value={(data.authTokenSettingsKey as string) || ''}
                                onChange={(e) => update('authTokenSettingsKey', e.target.value)}
                                placeholder="provider.github.api_key" />
                        </label>
                    )}
                    <label className="block">
                        <span className="text-xs text-[var(--text-muted)]">Request Body</span>
                        <textarea className="config-input min-h-[60px] font-mono text-xs"
                            value={(data.body as string) || ''}
                            onChange={(e) => update('body', e.target.value)}
                            placeholder='{"key": "value"}' />
                    </label>
                </>
            )}

            {type === 'file_read' && (
                <>
                    <label className="block">
                        <span className="text-xs text-[var(--text-muted)]">File Path</span>
                        <input className="config-input" value={(data.path as string) || ''}
                            onChange={(e) => update('path', e.target.value)}
                            placeholder="/path/to/file.txt" />
                    </label>
                    <label className="block">
                        <span className="text-xs text-[var(--text-muted)]">Mode</span>
                        <select className="config-input" value={(data.mode as string) || 'text'}
                            onChange={(e) => update('mode', e.target.value)}>
                            <option value="text">Text</option>
                            <option value="json">JSON</option>
                            <option value="csv">CSV</option>
                            <option value="binary">Binary</option>
                        </select>
                    </label>
                    <label className="block">
                        <span className="text-xs text-[var(--text-muted)]">Max Size (MB)</span>
                        <input type="number" className="config-input" value={(data.maxSize as number) ?? 10}
                            onChange={(e) => update('maxSize', parseFloat(e.target.value) || 10)} />
                    </label>
                    {(data.mode === 'csv') && (
                        <>
                            <label className="block">
                                <span className="text-xs text-[var(--text-muted)]">CSV Delimiter</span>
                                <input className="config-input" value={(data.csvDelimiter as string) || ','}
                                    onChange={(e) => update('csvDelimiter', e.target.value)} maxLength={1} />
                            </label>
                            <label className="flex items-center gap-2 text-xs text-[var(--text-secondary)]">
                                <input type="checkbox" checked={(data.csvHasHeader as boolean) ?? true}
                                    onChange={(e) => update('csvHasHeader', e.target.checked)} />
                                First row is header
                            </label>
                        </>
                    )}
                </>
            )}

            {type === 'file_write' && (
                <>
                    <label className="block">
                        <span className="text-xs text-[var(--text-muted)]">File Path</span>
                        <input className="config-input" value={(data.path as string) || ''}
                            onChange={(e) => update('path', e.target.value)}
                            placeholder="/path/to/output.json" />
                    </label>
                    <label className="block">
                        <span className="text-xs text-[var(--text-muted)]">Mode</span>
                        <select className="config-input" value={(data.mode as string) || 'text'}
                            onChange={(e) => update('mode', e.target.value)}>
                            <option value="text">Text</option>
                            <option value="json">JSON</option>
                            <option value="csv">CSV</option>
                        </select>
                    </label>
                    <label className="block">
                        <span className="text-xs text-[var(--text-muted)]">Write Mode</span>
                        <select className="config-input" value={(data.writeMode as string) || 'overwrite'}
                            onChange={(e) => update('writeMode', e.target.value)}>
                            <option value="overwrite">Overwrite</option>
                            <option value="append">Append</option>
                        </select>
                    </label>
                    <label className="flex items-center gap-2 text-xs text-[var(--text-secondary)]">
                        <input type="checkbox" checked={(data.createDirs as boolean) ?? true}
                            onChange={(e) => update('createDirs', e.target.checked)} />
                        Create parent directories
                    </label>
                </>
            )}

            {type === 'shell_exec' && (
                <>
                    <label className="block">
                        <span className="text-xs text-[var(--text-muted)]">Command</span>
                        <input className="config-input font-mono" value={(data.command as string) || ''}
                            onChange={(e) => update('command', e.target.value)}
                            placeholder="echo hello" />
                    </label>
                    <label className="block">
                        <span className="text-xs text-[var(--text-muted)]">Shell</span>
                        <select className="config-input" value={(data.shell as string) || 'bash'}
                            onChange={(e) => update('shell', e.target.value)}>
                            <option value="bash">bash</option>
                            <option value="sh">sh</option>
                            <option value="zsh">zsh</option>
                        </select>
                    </label>
                    <label className="block">
                        <span className="text-xs text-[var(--text-muted)]">Working Directory</span>
                        <input className="config-input" value={(data.workingDir as string) || ''}
                            onChange={(e) => update('workingDir', e.target.value)}
                            placeholder="/home/user" />
                    </label>
                    <label className="block">
                        <span className="text-xs text-[var(--text-muted)]">Timeout (seconds)</span>
                        <input type="number" className="config-input" value={(data.timeout as number) ?? 30}
                            onChange={(e) => update('timeout', parseInt(e.target.value) || 30)} />
                    </label>
                </>
            )}

            {type === 'validator' && (
                <>
                    <label className="block">
                        <span className="text-xs text-[var(--text-muted)]">JSON Schema</span>
                        <textarea className="config-input min-h-[100px] font-mono text-xs"
                            value={(data.schema as string) || '{}'}
                            onChange={(e) => update('schema', e.target.value)}
                            placeholder='{"type":"object","required":["name"]}' />
                    </label>
                    <label className="flex items-center gap-2 text-xs text-[var(--text-secondary)]">
                        <input type="checkbox" checked={(data.failOnError as boolean) ?? false}
                            onChange={(e) => update('failOnError', e.target.checked)} />
                        Fail on validation error
                    </label>
                </>
            )}

            {/* Last run output */}
            {nodeState && nodeState.status === 'completed' && nodeState.output && (
                <div className="pt-2 border-t border-[var(--border-subtle)]">
                    <div className="flex items-center gap-1 mb-1">
                        <Check size={12} className="text-green-400" />
                        <span className="text-xs font-medium text-green-400">Output</span>
                        {nodeState.durationMs != null && (
                            <span className="text-[10px] text-[var(--text-muted)] ml-auto">
                                {(nodeState.durationMs / 1000).toFixed(1)}s
                            </span>
                        )}
                    </div>
                    <pre className="text-xs text-[var(--text-secondary)] bg-[var(--bg-primary)] p-2 rounded max-h-[200px] overflow-y-auto whitespace-pre-wrap font-mono">
                        {nodeState.output}
                    </pre>
                </div>
            )}
            {nodeState && nodeState.status === 'error' && nodeState.error && (
                <div className="pt-2 border-t border-[var(--border-subtle)]">
                    <div className="flex items-center gap-1 mb-1">
                        <X size={12} className="text-red-400" />
                        <span className="text-xs font-medium text-red-400">Error</span>
                    </div>
                    <pre className="text-xs text-red-300 bg-red-500/10 p-2 rounded max-h-[120px] overflow-y-auto whitespace-pre-wrap font-mono">
                        {nodeState.error}
                    </pre>
                </div>
            )}

            <div className="pt-2 flex items-center justify-between">
                <span className="text-xs text-[var(--text-muted)]">ID: {node.id}</span>
                <button className="btn-icon text-red-400 hover:text-red-300" title="Delete node" onClick={onDelete}>
                    <Trash2 size={14} />
                </button>
            </div>
        </div>
    );
}
