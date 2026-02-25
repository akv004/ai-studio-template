// Muted Blender-inspired palette
export const nodeColors: Record<string, string> = {
    input: '#2d5a27',
    output: '#8a5a1e',
    llm: '#3a3a8a',
    tool: '#8a2a5a',
    router: '#1a6a6a',
    approval: '#8a7a1a',
    transform: '#5a3a8a',
    subworkflow: '#1a5a7a',
    http_request: '#2a6a4a',
    file_read: '#4a6a2a',
    file_glob: '#3a7a3a',
    file_write: '#6a4a2a',
    shell_exec: '#3a3a3a',
    validator: '#2a5a6a',
    iterator: '#6a3a6a',
    aggregator: '#4a2a6a',
    knowledge_base: '#2a4a7a',
    loop: '#4a2a6a',
    exit: '#4a2a6a',
};

export const PROVIDER_MODELS: Record<string, string[]> = {
    anthropic: ['claude-sonnet-4-5-20250929', 'claude-haiku-4-5-20251001', 'claude-opus-4-6'],
    google: ['gemini-2.0-flash', 'gemini-2.5-pro', 'gemini-2.5-flash'],
    azure_openai: ['gpt-4o', 'gpt-4o-mini', 'gpt-4.1'],
    ollama: ['llama3.2', 'llama3.1', 'mistral', 'codellama', 'qwen2.5'],
    local: ['qwen3-vl', 'qwen2.5-vl', 'llama3.2'],
};
