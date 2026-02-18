export function defaultDataForType(type: string): Record<string, unknown> {
    switch (type) {
        case 'input': return { name: 'input', dataType: 'text', default: '' };
        case 'output': return { name: 'result', format: 'text' };
        case 'llm': return { provider: '', model: '', systemPrompt: '', temperature: 0.7, maxTokens: 4096 };
        case 'tool': return { toolName: '', serverName: '', approval: 'auto' };
        case 'router': return { mode: 'pattern', branches: ['true', 'false'] };
        case 'approval': return { message: 'Review before continuing', showData: true, timeout: null };
        case 'transform': return { mode: 'template', template: '{{input}}', inputs: ['input'] };
        case 'subworkflow': return { workflowId: '', workflowName: '' };
        default: return {};
    }
}
