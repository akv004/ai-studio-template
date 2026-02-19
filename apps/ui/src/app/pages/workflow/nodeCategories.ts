import {
    FileInput, FileOutput, Cpu, GitFork,
    Wrench, ShieldCheck, Repeat, MessageSquare,
    Globe, Terminal, CheckSquare, FolderOpen,
} from 'lucide-react';

export interface NodeCategory {
    label: string;
    types: { type: string; label: string; icon: React.ElementType; description: string }[];
}

export const NODE_CATEGORIES: NodeCategory[] = [
    {
        label: 'Inputs / Outputs',
        types: [
            { type: 'input', label: 'Input', icon: FileInput, description: 'Workflow entry point' },
            { type: 'output', label: 'Output', icon: FileOutput, description: 'Workflow exit point' },
        ],
    },
    {
        label: 'AI',
        types: [
            { type: 'llm', label: 'LLM', icon: Cpu, description: 'Language model call' },
            { type: 'router', label: 'Router', icon: GitFork, description: 'Conditional branching' },
        ],
    },
    {
        label: 'Tools',
        types: [
            { type: 'tool', label: 'Tool', icon: Wrench, description: 'MCP or built-in tool' },
        ],
    },
    {
        label: 'Data I/O',
        types: [
            { type: 'http_request', label: 'HTTP Request', icon: Globe, description: 'Make HTTP API calls' },
            { type: 'file_glob', label: 'File Glob', icon: FolderOpen, description: 'Match files by pattern' },
            { type: 'file_read', label: 'File Read', icon: FileInput, description: 'Read local files' },
            { type: 'file_write', label: 'File Write', icon: FileOutput, description: 'Write local files' },
            { type: 'shell_exec', label: 'Shell Exec', icon: Terminal, description: 'Run shell commands' },
        ],
    },
    {
        label: 'Logic',
        types: [
            { type: 'approval', label: 'Approval', icon: ShieldCheck, description: 'Human approval gate' },
            { type: 'transform', label: 'Transform', icon: Repeat, description: 'Data transformation' },
            { type: 'validator', label: 'Validator', icon: CheckSquare, description: 'JSON Schema validation' },
        ],
    },
    {
        label: 'Composition',
        types: [
            { type: 'subworkflow', label: 'Subworkflow', icon: MessageSquare, description: 'Embed another workflow' },
        ],
    },
];
