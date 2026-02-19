export function defaultDataForType(type: string): Record<string, unknown> {
    switch (type) {
        case 'input': return { name: 'input', dataType: 'text', default: '' };
        case 'output': return { name: 'result', format: 'text' };
        case 'llm': return { provider: '', model: '', systemPrompt: '', temperature: 0.7, maxTokens: 4096, sessionMode: 'stateless', maxHistory: 20 };
        case 'tool': return { toolName: '', serverName: '', approval: 'auto' };
        case 'router': return { mode: 'pattern', branches: ['true', 'false'] };
        case 'approval': return { message: 'Review before continuing', showData: true, timeout: null };
        case 'transform': return { mode: 'template', template: '{{input}}', inputs: ['input'] };
        case 'subworkflow': return { workflowId: '', workflowName: '' };
        // Phase 4A new nodes
        case 'http_request': return {
            url: '', method: 'GET', headers: {}, body: '', timeout: 30,
            auth: 'none', authTokenSettingsKey: '', authHeader: 'Authorization',
            maxResponseBytes: 10485760,
        };
        case 'file_glob': return {
            directory: '', pattern: '*', mode: 'text', recursive: false,
            maxFiles: 100, maxSize: 10, sortBy: 'name', sortOrder: 'asc',
            csvDelimiter: ',', csvHasHeader: true,
        };
        case 'file_read': return {
            path: '', mode: 'text', encoding: 'utf-8',
            csvDelimiter: ',', csvHasHeader: true, maxSize: 10,
        };
        case 'file_write': return {
            path: '', mode: 'text', writeMode: 'overwrite',
            createDirs: true, csvDelimiter: ',', jsonPretty: true,
        };
        case 'shell_exec': return {
            command: '', workingDir: '', timeout: 30,
            shell: 'bash', envVars: {},
        };
        case 'validator': return {
            schema: '{}', failOnError: false,
        };
        // Phase 4B
        case 'iterator': return {
            mode: 'sequential', expression: '', maxConcurrency: 5,
        };
        case 'aggregator': return {
            strategy: 'array', separator: '\n',
        };
        default: return {};
    }
}
