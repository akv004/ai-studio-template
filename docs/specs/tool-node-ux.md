# AI Studio — Tool Node UX Specification

> **Version**: 1.0
> **Status**: PLANNED
> **Phase**: 4C (highest priority item)
> **Depends on**: mcp-integration.md, node-editor.md, phase4-automation-canvas.md

---

## Problem

The Tool node is the **only node in the workflow palette that's effectively unusable without reading source code**. Every other node has clear config fields (LLM has provider/model dropdowns, File Read has a path field, Shell Exec has a command field). The Tool node has:

1. **A blank text field** where you type a tool name — but you don't know what tools exist
2. **No parameter schema** — even if you know the name, you don't know what inputs it expects
3. **No validation** — type a wrong name, get a 404 at runtime
4. **Overlap confusion** — `builtin__shell` exists alongside the Shell Exec node. When do you use which?

This makes the Tool node — which should be the **most powerful node** (connect any MCP tool) — the most confusing one.

### User Pain (Observed)

- User wired a text Input directly into a Tool node → got "Input should be a valid dictionary" error from sidecar
- User didn't know `builtin__shell` existed — no way to discover it
- User didn't know the tool expects `{"command": "..."}` — no schema shown anywhere
- Documentation had to explain what the code should have made obvious

---

## Solution: Tool Browser + Dynamic Schema Form

Transform the Tool node from "type a magic string" into "browse → pick → configure."

### Core Features

| Feature | What it does | Priority |
|---------|-------------|----------|
| **Tool Picker Dropdown** | Searchable, grouped dropdown of all available tools | P0 |
| **Dynamic Input Form** | Auto-generate config fields from tool's `input_schema` | P0 |
| **Schema Preview** | Show expected input format below the picker | P1 |
| **Built-in Tool Hints** | When selecting a built-in tool, suggest the dedicated node instead | P1 |
| **Palette MCP Section** | MCP tools appear directly in the node palette, not just via Tool node | P2 |

---

## Design

### 1. Tool Picker Dropdown (P0)

Replace the free-text `toolName` input with a searchable dropdown grouped by server.

```
┌─────────────────────────────────────────┐
│  Tool                                   │
│  ┌───────────────────────────────────┐  │
│  │ 🔍 Search tools...               │  │
│  ├───────────────────────────────────┤  │
│  │ ▸ Built-in                        │  │
│  │   shell — Execute a shell command │  │
│  │   read_file — Read file contents  │  │
│  │   write_file — Write to a file    │  │
│  │   list_directory — List files     │  │
│  │                                   │  │
│  │ ▸ GitHub (5 tools)                │  │
│  │   create_issue — Create an issue  │  │
│  │   list_pulls — List pull requests │  │
│  │   search_repos — Search repos     │  │
│  │   ...                             │  │
│  │                                   │  │
│  │ ▸ Filesystem (8 tools)            │  │
│  │   read — Read a file              │  │
│  │   write — Write a file            │  │
│  │   ...                             │  │
│  │                                   │  │
│  │ ⚠ No MCP servers connected.      │  │
│  │   Add servers in Settings.        │  │
│  └───────────────────────────────────┘  │
│                                         │
│  Approval: [Auto ▾]                     │
└─────────────────────────────────────────┘
```

**Behavior:**
- Fetches tool list from sidecar via new Tauri IPC command `list_available_tools`
- Groups by server name (Built-in, GitHub, Filesystem, etc.)
- Each entry shows: tool name + one-line description
- Search filters across name + description
- If no MCP servers are connected, show a hint linking to Settings
- Selected tool populates `toolName` (qualified: `server__name`) and `serverName`
- Dropdown is cached per session, refreshed on MCP server connect/disconnect events

**Built-in tool hints:** When the user selects `builtin__shell`, show a subtle info banner:

```
ℹ Tip: The Shell Exec node does the same thing with a simpler config.
  Use the Tool node when you need dynamic input from upstream nodes.
```

### 2. Dynamic Input Form (P0)

When a tool is selected, render its `input_schema` as form fields in the config panel.

**Before (current):**
```
┌────────────────────────────┐
│ TOOL Configuration         │
│                            │
│ Tool Name: [builtin__shell]│
│ Approval:  [Auto ▾]       │
└────────────────────────────┘
```

**After:**
```
┌────────────────────────────────────┐
│ TOOL Configuration                 │
│                                    │
│ Tool: [builtin__shell ▾]           │
│ "Execute a shell command"          │
│                                    │
│ ── Parameters ──────────────────── │
│ command* [                       ] │
│   "The shell command to execute"   │
│                                    │
│ timeout  [30                     ] │
│   "Timeout in seconds"             │
│                                    │
│ cwd      [                       ] │
│   "Working directory (optional)"   │
│                                    │
│ ── Settings ────────────────────── │
│ Approval: [Auto ▾]                │
│ □ Use incoming data (ignore form)  │
│                                    │
│ ── Input Preview ───────────────── │
│ {"command": "...", "timeout": 30}  │
└────────────────────────────────────┘
```

**Schema → Form mapping:**

| JSON Schema type | Form field |
|-----------------|-----------|
| `string` | Text input (single line) or textarea if `maxLength > 200` |
| `number` / `integer` | Number input with min/max from schema |
| `boolean` | Checkbox |
| `enum` / `oneOf` | Dropdown |
| `object` | JSON textarea (fallback for nested objects) |
| `array` | JSON textarea (fallback) |

**Required fields** get an asterisk `*` and are validated before run.

**"Use incoming data" toggle:** When checked, the form is disabled (grayed out) and the node uses whatever JSON arrives from upstream. When unchecked (default), the form values are serialized as `toolInput` in the node config. This eliminates the confusion about static vs dynamic input.

### 3. Schema Preview on Node (P1)

The Tool node on the canvas itself shows a compact schema hint:

```
┌────────────────────────────┐
│ 🔧 TOOL                   │
│ builtin__shell             │
│ ┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄ │
│ command* · timeout · cwd   │
│ ┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄ │
│ ● input          result ●  │
└────────────────────────────┘
```

The parameter names (with `*` for required) give immediate visual context. When collapsed, only tool name and server show.

### 4. Palette MCP Section (P2)

MCP tools appear directly in the node palette as draggable items, not just via the generic Tool node.

```
Node Palette
├── Inputs/Outputs
│   ├── Input
│   └── Output
├── AI
│   ├── LLM
│   └── Router
├── Tools
│   ├── Tool (generic)
│   ├── Shell Exec
│   ├── ...
│   └── ── MCP Tools ──
│       ├── github: create_issue
│       ├── github: list_pulls
│       ├── filesystem: read
│       └── ...
```

Dragging an MCP tool from the palette creates a **pre-configured Tool node** with `toolName` already set. This is the "each MCP tool becomes a pre-configured Tool node" vision from the node-editor spec (line 437) — currently not implemented.

---

## Technical Design

### Backend Changes

#### 1. Sidecar: Include `input_schema` in tool summary

**File**: `apps/sidecar/agent/mcp/registry.py`

```python
def to_summary(self) -> list[dict]:
    """Summary for the /mcp/tools endpoint — now includes input_schema."""
    return [
        {
            "server": t.server,
            "name": t.name,
            "qualified_name": t.qualified_name,
            "description": t.description,
            "input_schema": t.input_schema,  # ADD THIS
        }
        for t in self._tools.values()
    ]
```

#### 2. Tauri: New IPC command `list_available_tools`

**File**: `apps/desktop/src-tauri/src/commands/mcp.rs`

New command that proxies to sidecar's `/mcp/tools`:

```rust
#[tauri::command]
pub async fn list_available_tools(
    sidecar: tauri::State<'_, SidecarManager>,
) -> Result<serde_json::Value, AppError> {
    let resp = sidecar.proxy_request("GET", "/mcp/tools", None).await?;
    Ok(resp)
}
```

Returns:
```json
{
  "tools": [
    {
      "server": "builtin",
      "name": "shell",
      "qualified_name": "builtin__shell",
      "description": "Execute a shell command",
      "input_schema": {
        "type": "object",
        "properties": {
          "command": {"type": "string", "description": "The shell command to execute"},
          "timeout": {"type": "number", "description": "Timeout in seconds", "default": 30},
          "cwd": {"type": "string", "description": "Working directory (optional)"}
        },
        "required": ["command"]
      }
    }
  ]
}
```

#### 3. Tauri: Emit event on MCP tool list change

When an MCP server connects/disconnects, emit `mcp_tools_changed` so the UI refreshes the cached tool list.

#### 4. Rust executor: Input validation (DONE)

Already implemented in this session — the executor now gives a clear error if incoming data is not a JSON object, pointing the user to use a Transform node.

### Frontend Changes

#### 1. Tool picker component

**File**: `apps/ui/src/app/pages/workflow/components/ToolPicker.tsx` (new)

```typescript
interface ToolInfo {
  server: string;
  name: string;
  qualified_name: string;
  description: string;
  input_schema: Record<string, unknown>;
}

// Fetches from list_available_tools IPC, groups by server
// Searchable combobox with server group headers
// Returns selected ToolInfo on pick
```

Uses the existing Zustand store pattern — add a `availableTools` slice that caches the tool list and refreshes on `mcp_tools_changed` events.

#### 2. Dynamic schema form component

**File**: `apps/ui/src/app/pages/workflow/components/SchemaForm.tsx` (new)

Renders a JSON Schema as form fields. Reusable for any node that needs schema-driven config (plugins in future).

```typescript
interface SchemaFormProps {
  schema: Record<string, unknown>;  // JSON Schema
  values: Record<string, unknown>;  // Current values
  onChange: (values: Record<string, unknown>) => void;
  disabled?: boolean;  // For "use incoming data" mode
}
```

Recursively renders schema properties as form fields. Falls back to JSON textarea for complex nested types.

#### 3. Updated NodeConfigPanel

**File**: `apps/ui/src/app/pages/workflow/NodeConfigPanel.tsx`

Replace the current Tool section:

```tsx
{type === 'tool' && (
    <>
        <ToolPicker
            value={data.toolName}
            onChange={(tool) => {
                update('toolName', tool.qualified_name);
                update('serverName', tool.server);
                update('toolSchema', tool.input_schema);
            }}
        />
        {data.toolSchema && (
            <>
                <label className="flex items-center gap-2 text-xs">
                    <input type="checkbox" checked={data.useIncoming}
                        onChange={e => update('useIncoming', e.target.checked)} />
                    Use incoming data (ignore form)
                </label>
                <SchemaForm
                    schema={data.toolSchema}
                    values={data.toolInput || {}}
                    onChange={v => update('toolInput', v)}
                    disabled={data.useIncoming}
                />
            </>
        )}
        <ApprovalSelect value={data.approval} onChange={v => update('approval', v)} />
    </>
)}
```

#### 4. Updated ToolNode.tsx

Show parameter names on the node face:

```tsx
// Extract required param names from schema
const schema = data.toolSchema as Record<string, unknown>;
const properties = schema?.properties as Record<string, unknown> || {};
const required = (schema?.required as string[]) || [];
const paramNames = Object.keys(properties).map(k =>
    required.includes(k) ? `${k}*` : k
).join(' · ');

// Render in node body
<div className="text-[9px] text-[#888] truncate">{paramNames}</div>
```

---

## Data Flow: Static vs Dynamic Input

This is the key UX decision that eliminates the "non-object input" error:

### Static Mode (form filled, "Use incoming data" unchecked)

```
[Any Node] → [Tool Node]
                 │
                 ├── toolInput = {"command": "git status"} (from form)
                 ├── incoming data = ignored
                 └── Sends: POST /tools/execute {"tool_name": "builtin__shell", "tool_input": {"command": "git status"}}
```

### Dynamic Mode ("Use incoming data" checked)

```
[Transform: {"command": "{{input}}"}] → [Tool Node]
                                            │
                                            ├── toolInput = null (form disabled)
                                            ├── incoming data = {"command": "echo hello"}
                                            └── Sends: POST /tools/execute {"tool_name": "builtin__shell", "tool_input": {"command": "echo hello"}}
```

### Template Mode (static form with `{{input}}` placeholders)

A future enhancement: allow `{{input}}` in form field values so the form acts as a template with dynamic substitution. Deferred — Transform node handles this for now.

---

## Migration

Existing workflows with Tool nodes continue to work unchanged:
- If `toolName` is set (string), it works as before
- New field `toolSchema` is populated lazily when the user opens the config panel
- `useIncoming` defaults to `true` if no `toolInput` is configured (preserves current behavior)
- `useIncoming` defaults to `false` if `toolInput` is configured

---

## Implementation Plan

| Step | Description | Effort | Files |
|------|------------|--------|-------|
| 4C.T1 | Sidecar: add `input_schema` to `/mcp/tools` response | Small | `registry.py` |
| 4C.T2 | Tauri: `list_available_tools` IPC command | Small | `commands/mcp.rs`, `lib.rs` |
| 4C.T3 | UI: Zustand `availableTools` store slice + IPC fetch + event refresh | Medium | `store.ts` |
| 4C.T4 | UI: `ToolPicker.tsx` — searchable grouped dropdown | Medium | new component |
| 4C.T5 | UI: `SchemaForm.tsx` — JSON Schema → form fields | Medium | new component |
| 4C.T6 | UI: Update `NodeConfigPanel.tsx` tool section | Small | existing file |
| 4C.T7 | UI: Update `ToolNode.tsx` — show param names on node face | Small | existing file |
| 4C.T8 | Tauri: emit `mcp_tools_changed` on connect/disconnect | Small | `commands/plugins.rs` |
| 4C.T9 | UI: Palette MCP section (P2 — optional) | Medium | `nodeCategories.ts`, palette component |

**Total estimate**: 3-4 focused implementation sessions.

**Critical path**: T1 → T2 → T3 → T4 → T6 (core picker working). T5 can be parallel. T7-T9 are polish.

---

## Verification Criteria

1. **Tool picker**: User can select any available tool from a dropdown without typing
2. **Schema form**: Selecting `builtin__shell` shows `command*`, `timeout`, `cwd` fields
3. **Static mode**: Filling the form and running works without upstream wiring
4. **Dynamic mode**: Checking "Use incoming data" and wiring a Transform works
5. **No MCP servers**: Picker shows only built-in tools + "Add servers in Settings" hint
6. **Refresh**: Connecting a new MCP server in Settings refreshes the tool list
7. **Migration**: Existing workflows with Tool nodes still execute correctly
8. **Tests**: `cargo test` passes, TypeScript compiles clean

---

## Why This is a Killer Feature

1. **Eliminates the #1 confusion point** — the Tool node goes from unusable to "browse and pick"
2. **Makes MCP real** — currently MCP servers connect but tools are invisible in workflows. This surfaces them as first-class workflow nodes.
3. **Schema-driven forms** are reusable for plugin-provided nodes (Phase 5) — building this once pays off forever
4. **Demo impact**: Show HN video shows "connect GitHub MCP server → instantly see 12 tools in the palette → drag create_issue into workflow → form auto-fills with title/body fields." That's a wow moment.
5. **No competitor has this**: LangFlow has a tool node but no MCP integration. n8n has connectors but no LLM-aware workflows. We're the only visual IDE where MCP tools become drag-and-drop workflow nodes with auto-generated forms.
