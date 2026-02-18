# Workflow Templates

Bundled workflow templates for AI Studio's Node Editor.

## Available Templates

| Template | Nodes | Pattern | Description |
|----------|-------|---------|-------------|
| **Code Review** | 4 | Input → LLM → Router → Output | Analyze PR, classify by severity |
| **Research Assistant** | 4 | Input → LLM → Transform → Output | Research a topic, produce formatted report |
| **Data Pipeline** | 4 | Input → LLM → Transform → Output | Extract structured data from raw input |
| **Multi-Model Compare** | 6 | Input → 3x LLM → Transform → Output | Same prompt to 3 models, compare |
| **Safe Executor** | 5 | Input → LLM → Approval → Tool → Output | Plan command, human approval, execute |
| **Email Classifier** | 5 | Input → LLM → Router → LLM → Output | Classify emails, auto-draft urgent replies |
| **Content Moderator** | 5 | Input → LLM → Router → Approval → Output | Screen content with human review for borderline |
| **Translation Pipeline** | 6 | 2x Input → LLM → Transform → LLM → Output | Detect language, translate to target |
| **Meeting Notes** | 5 | Input → 2x LLM → Transform → Output | Parallel summarize + extract action items |
| **RAG Search** | 5 | Input → Tool → Transform → LLM → Output | Search then answer with retrieved context |

## Template Format

Templates are React Flow graph JSON files with this structure:

```json
{
  "nodes": [
    {
      "id": "unique_id",
      "type": "input|output|llm|tool|router|approval|transform|subworkflow",
      "position": { "x": 0, "y": 0 },
      "data": { ... }
    }
  ],
  "edges": [
    {
      "id": "edge_id",
      "source": "source_node_id",
      "target": "target_node_id",
      "sourceHandle": "output_handle_name",
      "targetHandle": "input_handle_name"
    }
  ],
  "viewport": { "x": 0, "y": 0, "zoom": 1 }
}
```

### Node Types

| Type | Handles (in → out) | Data Fields |
|------|-------------------|-------------|
| `input` | — → `output` | `name`, `dataType`, `default` |
| `output` | `value` → — | `name`, `format` |
| `llm` | `prompt` → `response` | `provider`, `model`, `systemPrompt`, `temperature`, `maxTokens` |
| `tool` | `input` → `result` | `toolName`, `serverName`, `approval` |
| `router` | `input` → `branch-N` | `mode` (llm/rule), `branches[]` |
| `approval` | `data` → `approved` | `message`, `showData`, `timeout` |
| `transform` | `input` → `output` | `mode` (template), `template` |
| `subworkflow` | `input` → `output` | `workflowId` |

### Template References

Use `{{node_id.handle}}` to reference outputs from other nodes in transform templates:
- `{{llm_1.response}}` — output from an LLM node
- `{{input_1}}` — shorthand for input node value
- `{{tool_1.result}}` — output from a tool execution

### Provider/Model Fields

Leave `provider` and `model` as empty strings (`""`) to let the user configure them after import. The node editor will prompt for these when running.

## Contributing a Template

1. Create a `.json` file following the format above
2. Test it by importing via the Node Editor's import button
3. Add it to `src/commands/templates.rs` in the `TEMPLATES` array
4. Add a row to the table in this README
5. Submit a PR

**Tips:**
- Keep templates focused — one clear use case per template
- Use 4-8 nodes (not too simple, not overwhelming)
- Leave provider/model empty for flexibility
- Use descriptive node IDs (`llm_1`, `router_1`, not `n1`, `n2`)
- Space nodes ~300px apart horizontally for readability
