import type { NodeTypes } from '@xyflow/react';
import { InputNode } from './nodes/InputNode';
import { OutputNode } from './nodes/OutputNode';
import { LLMNode } from './nodes/LLMNode';
import { ToolNode } from './nodes/ToolNode';
import { RouterNode } from './nodes/RouterNode';
import { ApprovalNode } from './nodes/ApprovalNode';
import { TransformNode } from './nodes/TransformNode';
import { SubworkflowNode } from './nodes/SubworkflowNode';
import { HttpRequestNode } from './nodes/HttpRequestNode';
import { FileReadNode } from './nodes/FileReadNode';
import { FileGlobNode } from './nodes/FileGlobNode';
import { FileWriteNode } from './nodes/FileWriteNode';
import { ShellExecNode } from './nodes/ShellExecNode';
import { ValidatorNode } from './nodes/ValidatorNode';
import { IteratorNode } from './nodes/IteratorNode';
import { AggregatorNode } from './nodes/AggregatorNode';
import { KnowledgeBaseNode } from './nodes/KnowledgeBaseNode';

export const customNodeTypes: NodeTypes = {
    input: InputNode,
    output: OutputNode,
    llm: LLMNode,
    tool: ToolNode,
    router: RouterNode,
    approval: ApprovalNode,
    transform: TransformNode,
    subworkflow: SubworkflowNode,
    http_request: HttpRequestNode,
    file_read: FileReadNode,
    file_glob: FileGlobNode,
    file_write: FileWriteNode,
    shell_exec: ShellExecNode,
    validator: ValidatorNode,
    iterator: IteratorNode,
    aggregator: AggregatorNode,
    knowledge_base: KnowledgeBaseNode,
};
