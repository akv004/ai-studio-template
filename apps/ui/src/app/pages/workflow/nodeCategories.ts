import {
    FileInput, FileOutput, Cpu, GitFork,
    Wrench, ShieldCheck, Repeat, MessageSquare,
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
        label: 'Logic',
        types: [
            { type: 'approval', label: 'Approval', icon: ShieldCheck, description: 'Human approval gate' },
            { type: 'transform', label: 'Transform', icon: Repeat, description: 'Data transformation' },
        ],
    },
    {
        label: 'Composition',
        types: [
            { type: 'subworkflow', label: 'Subworkflow', icon: MessageSquare, description: 'Embed another workflow' },
        ],
    },
];
