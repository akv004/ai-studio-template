# Node Editor Pro Design Spec
**Status**: DRAFT
**Inspiration**: Unreal Engine 5 Blueprints, Blender Geometry Nodes

## 1. Visual Language: Typed Handles

To match the "Unreal" professional look, handles must be rigorously color-coded by data type.

| Data Type | Color | Hex | Reference |
|-----------|-------|-----|-----------|
| **Execution** | White | `#FFFFFF` | Flow control (if we add exec pins later) |
| **Boolean** | Red | `#EF4444` | True/False logic |
| **String** | Magenta | `#E879F9` | Text, Prompts |
| **JSON/Object** | Orange | `#F59E0B` | Complex data structures |
| **Float/Number**| Green | `#10B981` | Numeric values (Costs, Confidence) |
| **Any/Wildcard**| Gray | `#9CA3AF` | Universal connections |

**Shape Language**:
- **Data Pins**: Circular (`rounded-full`)
- **Execution Pins**: Arrow/Triangle (Future Phase)
- **Array/List**: Grid/Square (Future Phase)

## 2. Multi-IO Node Layouts

Nodes should expose their internal data flow, not hide it inside a black box.

### LLM Node (Expanded)
Instead of a single "Prompt -> Response" flow, expose the full context.

**Inputs (Left):**
- `system` (String): System prompt override.
- `context` (JSON): RAG context or conversation history.
- `prompt` (String): User message / Template.

**Outputs (Right):**
- `response` (String): The generated text.
- `usage` (JSON): Full usage object `{ input_tokens, output_tokens, ... }`.
- `cost` (String): Formatted cost string `"$0.002"`.

### Transform Node (Script)
Acts as the glue code.
- **Inputs**: Dynamic (added by user).
- **Outputs**: `result` (The computed value).

## 3. Visual Validation
- **Connection Logic**: Dragging a connection should highlight compatible pins and dim incompatible ones.
- **Example**: Dragging a `JSON` output should highlight `JSON` and `Any` inputs, but dim `Boolean` inputs.

## 4. "Add Pin" Workflow
For dynamic nodes (Transform, Router), an "Add Pin" button should be visible directly on the node or in its context menu, similar to Unreal's "Add Pin" on switch nodes.
