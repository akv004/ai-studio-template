export function defaultDataForType(type: string): Record<string, unknown> {
    switch (type) {
        case 'input': return { label: '', name: 'input', dataType: 'text', default: '' };
        case 'output': return { label: '', name: 'result', format: 'text' };
        case 'llm': return { label: '', provider: '', model: '', systemPrompt: '', temperature: 0.7, maxTokens: 4096, sessionMode: 'stateless', maxHistory: 20 };
        case 'tool': return { label: '', toolName: '', serverName: '', approval: 'auto' };
        case 'router': return { label: '', mode: 'pattern', branches: ['true', 'false'] };
        case 'approval': return { label: '', message: 'Review before continuing', showData: true, timeout: null };
        case 'transform': return { label: '', mode: 'template', template: '{{input}}', inputs: ['input'] };
        case 'subworkflow': return { label: '', workflowId: '', workflowName: '' };
        // Phase 4A new nodes
        case 'http_request': return {
            label: '', url: '', method: 'GET', headers: {}, body: '', timeout: 30,
            auth: 'none', authTokenSettingsKey: '', authHeader: 'Authorization',
            maxResponseBytes: 10485760,
        };
        case 'file_glob': return {
            label: '', directory: '', pattern: '*', mode: 'text', recursive: false,
            maxFiles: 100, maxSize: 10, sortBy: 'name', sortOrder: 'asc',
            csvDelimiter: ',', csvHasHeader: true,
        };
        case 'file_read': return {
            label: '', path: '', mode: 'text', encoding: 'utf-8',
            csvDelimiter: ',', csvHasHeader: true, maxSize: 10,
        };
        case 'file_write': return {
            label: '', path: '', mode: 'text', writeMode: 'overwrite',
            createDirs: true, csvDelimiter: ',', jsonPretty: true,
        };
        case 'shell_exec': return {
            label: '', command: '', workingDir: '', timeout: 30,
            shell: 'bash', envVars: {},
        };
        case 'validator': return {
            label: '', schema: '{}', failOnError: false,
        };
        // Phase 4B
        case 'iterator': return {
            label: '', mode: 'sequential', expression: '', maxConcurrency: 5,
        };
        case 'aggregator': return {
            label: '', strategy: 'array', separator: '\n',
        };
        default: return { label: '' };
    }
}
