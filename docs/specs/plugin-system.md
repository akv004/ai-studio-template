# AI Studio — Plugin System Specification

> **Version**: 1.0
> **Status**: Draft
> **Depends on**: mcp-integration.md, node-editor.md, architecture.md

---

## Overview

The plugin system extends AI Studio with third-party capabilities. Plugins can provide **tools** (via MCP protocol), **workflow node types**, and **metadata** for discovery and management.

**Design principle**: Build on MCP. A plugin is a packaged MCP server with a manifest that adds metadata, permissions, and lifecycle management.

---

## Plugin Format

Plugins live in `~/.ai-studio/plugins/` (or a user-configured directory):

```
~/.ai-studio/plugins/
  github-tools/
    plugin.json          # Manifest (required)
    main.py              # Entry point
    requirements.txt     # Dependencies
  jira-connector/
    plugin.json
    index.js
    package.json
```

### Manifest (`plugin.json`)

```json
{
  "id": "github-tools",
  "name": "GitHub Tools",
  "version": "1.0.0",
  "description": "GitHub integration — issues, PRs, repos",
  "author": "AI Studio Community",
  "homepage": "https://github.com/ai-studio/plugin-github",
  "license": "MIT",
  "engine": ">=0.1.0",

  "entry_point": "main.py",
  "runtime": "python",
  "transport": "stdio",

  "permissions": ["network"],

  "provides": {
    "tools": true,
    "node_types": []
  }
}
```

### Manifest Fields

| Field | Required | Description |
|-------|----------|-------------|
| `id` | Yes | Unique identifier (kebab-case, e.g., `github-tools`) |
| `name` | Yes | Display name |
| `version` | Yes | SemVer version |
| `description` | Yes | Short description |
| `author` | No | Author name |
| `homepage` | No | URL to source/docs |
| `license` | No | SPDX license identifier |
| `engine` | No | Minimum AI Studio version required |
| `entry_point` | Yes | Main file to execute |
| `runtime` | Yes | `python` \| `node` \| `binary` |
| `transport` | Yes | `stdio` (only supported transport for v1) |
| `permissions` | Yes | Array of permission scopes |
| `provides.tools` | No | `true` if plugin provides MCP tools |
| `provides.node_types` | No | Array of custom node type IDs (Phase 4) |

### Permission Scopes

| Scope | What It Grants |
|-------|---------------|
| `network` | Outbound HTTP/HTTPS requests |
| `filesystem` | Read/write to the file system |
| `shell` | Execute shell commands |
| `env` | Access environment variables |

Plugins declare permissions upfront. Users see them before enabling.

---

## Data Model

### SQLite Schema (v7)

```sql
CREATE TABLE IF NOT EXISTS plugins (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    version TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    author TEXT NOT NULL DEFAULT '',
    homepage TEXT NOT NULL DEFAULT '',
    license TEXT NOT NULL DEFAULT '',
    runtime TEXT NOT NULL DEFAULT 'python',
    entry_point TEXT NOT NULL,
    transport TEXT NOT NULL DEFAULT 'stdio',
    permissions TEXT NOT NULL DEFAULT '[]',
    provides_tools INTEGER NOT NULL DEFAULT 0,
    provides_node_types TEXT NOT NULL DEFAULT '[]',
    directory TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 0,
    installed_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

---

## Plugin Lifecycle

```
Install → Scan directory → Parse manifest → Insert into plugins table (disabled)
Enable  → Validate permissions → Start subprocess → Register tools → Mark enabled
Disable → Stop subprocess → Unregister tools → Mark disabled
Remove  → Disable + Delete from table (files left for user to clean up)
```

### Install Flow

1. User places plugin directory in `~/.ai-studio/plugins/`
2. User clicks "Scan for Plugins" in Settings (or app scans on startup)
3. App reads `plugin.json` from each subdirectory
4. Validates manifest fields
5. Inserts into `plugins` table with `enabled = 0`

### Enable Flow

1. User clicks "Enable" on a plugin
2. App shows permission prompt: "This plugin requests: [network, filesystem]"
3. On approval: spawn subprocess using `runtime` + `entry_point`
4. If `provides.tools = true`: connect via MCP protocol, discover tools
5. Register discovered tools in the tool registry
6. Update `enabled = 1`

### Subprocess Management

Plugins run as isolated subprocesses, exactly like MCP servers:

```
Python plugin:  python /path/to/plugin/main.py
Node plugin:    node /path/to/plugin/index.js
Binary plugin:  /path/to/plugin/main
```

Communication is via **stdio JSON-RPC** (MCP protocol). The sidecar manages plugin processes alongside MCP server processes.

---

## IPC Commands (Tauri)

| Command | Input | Output | Description |
|---------|-------|--------|-------------|
| `list_plugins` | — | `Plugin[]` | List all installed plugins |
| `scan_plugins` | — | `ScanResult` | Scan plugin directory, install new ones |
| `enable_plugin` | `{ id }` | `()` | Enable a plugin (starts subprocess) |
| `disable_plugin` | `{ id }` | `()` | Disable a plugin (stops subprocess) |
| `remove_plugin` | `{ id }` | `()` | Remove plugin from registry |

---

## TypeScript Types

```typescript
interface Plugin {
  id: string;
  name: string;
  version: string;
  description: string;
  author: string;
  homepage: string;
  license: string;
  runtime: 'python' | 'node' | 'binary';
  entryPoint: string;
  transport: 'stdio';
  permissions: PluginPermission[];
  providesTools: boolean;
  providesNodeTypes: string[];
  directory: string;
  enabled: boolean;
  installedAt: string;
  updatedAt: string;
}

type PluginPermission = 'network' | 'filesystem' | 'shell' | 'env';

interface ScanResult {
  installed: number;
  updated: number;
  errors: string[];
}
```

---

## Settings UI

```
┌─────────────────────────────────────────────────┐
│  Plugins                         [Scan Plugins] │
│                                                 │
│  ┌───────────────────────────────────────────┐  │
│  │ GitHub Tools              v1.0.0   [ON]   │  │
│  │ GitHub integration — issues, PRs, repos   │  │
│  │ by AI Studio Community                    │  │
│  │ Permissions: network                      │  │
│  │ Tools: 12 registered                      │  │
│  │                          [Disable] [Remove]│  │
│  ├───────────────────────────────────────────┤  │
│  │ Jira Connector            v0.2.1   [OFF]  │  │
│  │ Jira issue management                     │  │
│  │ by jira-community                         │  │
│  │ Permissions: network, env                 │  │
│  │                          [Enable] [Remove] │  │
│  └───────────────────────────────────────────┘  │
│                                                 │
│  Plugin directory: ~/.ai-studio/plugins/        │
│  [Open Directory]                               │
└─────────────────────────────────────────────────┘
```

---

## Integration with Existing Systems

### MCP Tool Registry

When a plugin with `provides.tools = true` is enabled, it's registered as an MCP server in the sidecar. The sidecar treats plugin tools identically to regular MCP tools:
- Tool names: `{plugin_id}:{tool_name}` (same namespacing)
- Same approval flow
- Same event recording
- Same Inspector visibility

### Node Editor (Phase 4)

Plugins declaring `provides.node_types` will register custom node types in the node editor. This requires:
- A node schema (inputs, outputs, config)
- A Rust executor function (provided via the plugin subprocess)
- A React component (future — plugin UI panels)

**Phase 4 scope** — for now, `provides.node_types` is stored but not acted upon.

### Agents

Agents can reference plugins in their tool configuration, same as MCP servers. Enabling a plugin makes its tools available to all agents that opt in.

---

## Implementation Plan

| Task | Effort | Phase |
|------|--------|-------|
| Schema v7 migration (plugins table) | Low | Now |
| Plugin CRUD commands in Rust | Medium | Now |
| Plugin scanner (read plugin.json from directory) | Medium | Now |
| TypeScript types + Zustand store | Low | Now |
| Settings UI — plugin list + enable/disable | Medium | Now |
| Plugin subprocess lifecycle (in sidecar) | High | Now |
| Plugin → MCP tool registration bridge | Medium | Now |
| Custom node types from plugins | High | Phase 4 |
| Plugin UI panels | High | Phase 4 |
| Plugin marketplace / gallery | High | Phase 4+ |

---

## What This Does NOT Include (Phase 4+)

- Plugin marketplace / gallery
- Auto-update mechanism
- Custom node types from plugins
- Plugin-provided React UI panels
- Plugin sandboxing beyond subprocess isolation
- Plugin-to-plugin communication
