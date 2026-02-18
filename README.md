<p align="center">
  <img src="docs/screenshots/screenshot-1-main-dashboard.png" alt="AI Studio" width="800">
</p>

<h1 align="center">AI Studio</h1>

<p align="center">
  <strong>The open-source IDE for AI agents.</strong><br>
  Build, run, debug, and compare AI agents — with full visibility into every decision and dollar spent.
</p>

<p align="center">
  <a href="#features">Features</a> &middot;
  <a href="#node-editor">Node Editor</a> &middot;
  <a href="#quick-start">Quick Start</a> &middot;
  <a href="#architecture">Architecture</a> &middot;
  <a href="#roadmap">Roadmap</a>
</p>

<p align="center">
  <a href="https://github.com/akv004/ai-studio-template/stargazers"><img src="https://img.shields.io/github/stars/akv004/ai-studio-template?style=social" alt="Stars"></a>
  <img src="https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-blue" alt="Platform">
  <img src="https://img.shields.io/badge/license-MIT-green" alt="License">
  <img src="https://img.shields.io/badge/PRs-welcome-brightgreen" alt="PRs Welcome">
</p>

---

## What is AI Studio?

**Chrome DevTools for AI agents** — a desktop IDE where you build, run, inspect, and compare AI agents with full visibility into every step, every token, and every dollar.

> You say: "Refactor the auth module to use JWT"
>
> The agent reads 3 files, installs a package, writes 2 files. You see every step in the Inspector — every tool call, every file read, every approval decision. The agent used 2,341 tokens and cost you $0.012. It took 4.8 seconds.
>
> Something wrong? Click any step. See the exact input and output. Click **Branch from here**. Try a different approach. Compare results side-by-side.

---

## Features

### Agent Inspector (Flagship)

No other tool gives you this level of visibility into AI agent behavior:

- **Event Timeline** — every message, LLM call, tool execution as a navigable timeline
- **Tool Call Deep-Dive** — click any tool call to see input, output, approval status, duration
- **Token & Cost Tracking** — per-turn and cumulative, with model pricing
- **Session Branching** — fork from any point, try different models, diff results
- **Export** — full session as JSON or Markdown
- **Keyboard Navigation** — arrow keys, vim-style, filter chips

### Node Editor — Visual AI Pipelines

**"Unreal Blueprints for AI agents."** Build complex AI workflows by connecting nodes — no code required:

- **8 Node Types** — Input, Output, LLM, Tool, Router, Approval, Transform, Subworkflow
- **DAG Execution Engine** — topological sort, parallel branches via `tokio::join_all`
- **Live Execution View** — watch data flow through nodes with status badges and cost per node
- **5 Bundled Templates** — Code Review, Research, Data Pipeline, Multi-Model Compare, Safe Executor
- **Export/Import** — share workflows as JSON files
- **Blender-inspired UI** — dark theme, labeled handles, collapsible nodes, context menu + keyboard shortcuts

### Hybrid Intelligence

AI Studio auto-picks the best model for each step:

- **Simple questions** &rarr; Local Llama (free, private, fast)
- **Complex code** &rarr; Claude Sonnet (best at code)
- **Vision tasks** &rarr; GPT-4o (strong vision)
- **Large context** &rarr; Gemini Flash (1M context, cheap)
- **Budget controls** — set monthly limits, automatic fallback to local when budget runs low
- **Budget enforcement** — hard limits respected: `local_only`, `cheapest_cloud`, or `ask` when exhausted

### MCP-Native Tools

Built on [Model Context Protocol](https://modelcontextprotocol.io/) — the open standard for AI tool integration:

- Connect any MCP server (GitHub, Postgres, Filesystem, Brave Search, and growing)
- Built-in tools: shell, filesystem, browser
- One-click setup from curated server list
- Full approval workflow for every tool call

### Plugin System

Extend AI Studio with third-party capabilities:

- Plugins live in `~/.ai-studio/plugins/` with a `plugin.json` manifest
- Communicate via stdio JSON-RPC (MCP protocol)
- Permission declarations (network, filesystem, shell, env)
- Enable/disable/scan from Settings UI
- See the [plugin spec](docs/specs/plugin-system.md) for details

### 6 Focused Modules

| Module | What It Does |
|--------|-------------|
| **Agents** | Create and configure AI agents (model, prompt, tools, permissions, routing) |
| **Sessions** | Interactive chat with real-time tool approval and streaming |
| **Runs** | Headless batch execution — CI/CD for agents |
| **Inspector** | Deep-dive into any session: timeline, traces, cost, replay |
| **Node Editor** | Visual pipeline builder — connect LLM, tool, router, and approval nodes |
| **Settings** | Providers, MCP servers, plugins, budget, preferences |

---

## Why AI Studio?

| | ChatGPT / Claude | Cursor | LangFlow | **AI Studio** |
|---|---|---|---|---|
| Full Inspector (traces, replay, branch)? | No | No | No | **Yes** |
| Visual pipeline builder? | No | No | Yes | **Yes (+ execution engine)** |
| Hybrid intelligence (auto-pick model)? | No | No | No | **Yes** |
| Cost tracking per message? | No | No | No | **Yes** |
| Approval rules for safety? | N/A | N/A | Minimal | **Full rules engine** |
| Local-first (data stays on machine)? | No | No | Partial | **Yes** |
| MCP-native tool system? | Limited | Limited | No | **Yes** |
| Desktop app (not SaaS)? | No | Yes | No | **Yes** |
| Open source and free? | No | No | Yes | **Yes** |

---

## Quick Start

### Prerequisites

| Tool | Version | Required |
|------|---------|----------|
| **Node.js** | 18+ | Yes |
| **Rust** | Latest stable | Yes (for Tauri desktop) |
| **Python** | 3.10+ | Yes (for AI sidecar) |

### Install

```bash
# 1. Clone
git clone https://github.com/akv004/ai-studio-template.git
cd ai-studio-template

# 2. Install dependencies
npm install

# 3. Install Python dependencies
pip install -r apps/sidecar/requirements.txt

# 4. Run (full desktop app)
npm run tauri:dev
```

First build takes ~3-5 minutes (Rust compilation). After that, it's instant.

### Other Run Modes

```bash
# Web UI only (for frontend development)
npm run dev

# Sidecar only (API development)
cd apps/sidecar && python server.py
# Swagger docs at http://localhost:8765/docs

# Docker (sidecar + Ollama)
docker compose up
```

### Connect a Model

**Local (free)**: Install [Ollama](https://ollama.ai), pull a model (`ollama pull llama3.2`), AI Studio auto-detects it.

**Cloud**: Open Settings, add your API key for Anthropic, OpenAI, or Google AI.

<details>
<summary><strong>Detailed Setup Guide</strong> (first-time setup, per-OS instructions)</summary>

#### Install Node.js

```bash
# macOS (Homebrew)
brew install node

# Windows (Chocolatey)
choco install nodejs

# Or download from https://nodejs.org/
```

#### Install Rust (Required for Tauri)

```bash
# macOS / Linux
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# When prompted, select: 1) Proceed with standard installation
# Then restart your terminal or run:
source ~/.cargo/env

# Verify
rustc --version
```

#### Install Tauri CLI

```bash
cargo install tauri-cli
```

> First time takes ~2-3 minutes (compiles 700+ crates).

</details>

<details>
<summary><strong>Docker Setup</strong> (sidecar + Ollama, GPU support)</summary>

```bash
# CPU mode
docker compose up

# GPU mode (requires nvidia-container-toolkit)
docker compose --profile gpu up
```

</details>

---

## Architecture

```
┌──────────────────────────────────────────────────┐
│              UI Layer (React 19 + Tauri)          │
│  Agents | Sessions | Runs | Inspector | Workflows│
└────────────────────┬─────────────────────────────┘
                     │ Tauri IPC
┌────────────────────┴─────────────────────────────┐
│            Desktop Layer (Rust/Tauri 2)           │
│  SQLite DB | Smart Router | Approval | Events    │
│  Workflow Engine (DAG walker, node executors)     │
└────────────────────┬─────────────────────────────┘
                     │ HTTP + WebSocket
┌────────────────────┴─────────────────────────────┐
│            Agent Layer (Python Sidecar)           │
│  LLM Providers | MCP Client | Event Emitter      │
└──────────────────────────────────────────────────┘
```

**3 layers, clear responsibilities:**

- **UI (React 19)** — what you see. Never talks to the sidecar directly.
- **Desktop (Rust/Tauri 2)** — security boundary. SQLite persistence, smart model router, tool approval, event bridging, workflow execution engine.
- **Agent (Python FastAPI)** — does the work. Calls LLMs, executes tools, emits events.

**Key design decisions:**
- **Local-first**: All data in SQLite on your machine. No cloud, no account.
- **Event-sourced**: Every agent action is a typed event. Inspector reads from the event log.
- **MCP-native**: Tools are MCP servers. Interoperable with Claude Desktop, Cursor, Zed, etc.
- **Hybrid intelligence**: Smart router picks the best model per step with budget awareness.
- **Spec-driven**: Every feature has a [design specification](docs/specs/) written before implementation.

### Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop Shell | Tauri 2.0 (Rust) |
| UI | React 19 + TypeScript + Vite |
| Styling | Tailwind CSS 4 |
| State | Zustand |
| Database | SQLite (WAL mode, via rusqlite) |
| AI Backend | FastAPI (Python 3.10+) |
| LLM Providers | Ollama, Anthropic, OpenAI, Google AI, Azure OpenAI |
| Tool System | Model Context Protocol (MCP) |
| Node Editor | React Flow (@xyflow/react) |
| Smart Router | Rust (static rules + budget-aware routing) |

---

## Project Structure

```
ai-studio-template/
├── apps/
│   ├── ui/                  # React frontend (6 modules)
│   │   └── src/app/pages/   # AgentsPage, SessionsPage, NodeEditorPage, etc
│   ├── desktop/src-tauri/   # Tauri/Rust backend
│   │   └── src/
│   │       ├── commands/    # 13 command modules (agents, chat, workflows, etc)
│   │       ├── workflow/    # DAG engine, validators, 7 node executors
│   │       ├── routing.rs   # Smart model router (14 tests)
│   │       └── db.rs        # SQLite schema v7 + migrations
│   └── sidecar/             # Python (LLM providers, MCP, tools, events)
├── packages/
│   └── shared/              # Shared TypeScript types
├── docs/
│   └── specs/               # 12 design specifications
└── package.json             # Monorepo workspace config
```

---

## Design Specs

Every feature is designed spec-first:

| Spec | Covers |
|------|--------|
| [Product Vision](docs/specs/product-vision.md) | Direction, pillars, competitive positioning |
| [Architecture](docs/specs/architecture.md) | 3-layer system, data flows, security |
| [Event System](docs/specs/event-system.md) | Typed events, transport, cost calculation |
| [Data Model](docs/specs/data-model.md) | SQLite schema, migrations, branching |
| [Agent Inspector](docs/specs/agent-inspector.md) | Flagship: timeline, replay, branch, export |
| [MCP Integration](docs/specs/mcp-integration.md) | Tool system, discovery, approval |
| [Hybrid Intelligence](docs/specs/hybrid-intelligence.md) | Smart routing, budget, savings |
| [Node Editor](docs/specs/node-editor.md) | Visual pipelines: 8 node types, DAG engine |
| [Plugin System](docs/specs/plugin-system.md) | Manifest, loader, permissions, lifecycle |
| [API Contracts](docs/specs/api-contracts.md) | Every IPC command, endpoint, WebSocket |
| [UI Design](docs/specs/ui-design.md) | Wireframes, colors, interactions |
| [Phase Plan](docs/specs/phase-plan.md) | Implementation roadmap (Phase 0-3) |

---

## Roadmap

| Phase | Focus | Status |
|-------|-------|--------|
| **Phase 0** | Specs + codebase cleanup | Done |
| **Phase 1** | Core loop — agents, sessions, persistence, events, inspector, MCP | Done |
| **Phase 2** | Polish — session branching, cost tracking, inspector improvements, onboarding | Done |
| **Phase 3** | Node editor, hybrid intelligence, plugin system | **In progress** |
| **Phase 4** | Universal automation — custom node types, loop/merge nodes, plugin marketplace | Planned |

### What's Built

- SQLite local-first persistence (WAL mode, schema v7, 7 migrations)
- 5 LLM providers (Ollama, Anthropic, OpenAI, Google AI, Azure)
- MCP-native tool system with registry, approval rules, and stdio client
- Multi-turn tool calling with event-sourced audit trail
- Agent Inspector with timeline, grouping, filters, keyboard nav, markdown export
- Session branching (fork-and-compare)
- Headless runs with async background execution
- Node Editor with 8 custom node types, React Flow canvas, DAG execution engine
- Smart model router (3 modes: single, auto, manual) with 14 unit tests
- Budget tracking with monthly limits and enforcement
- Plugin system with manifest scanning and Settings UI
- Blender-inspired node styling, context menus, keyboard shortcuts
- 5 bundled workflow templates + JSON export/import
- 31 Rust unit tests across routing, validation, and template resolution

See [STATUS.md](STATUS.md) for the detailed sprint board.

---

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd+K` | Command Palette |
| `Cmd+1-5` | Navigate modules |
| `Cmd+,` | Settings |
| `Cmd+N` | New agent |
| `Cmd+Shift+N` | New session |
| `Cmd+I` | Inspector for current session |
| `Cmd+Enter` | Send message |
| `Ctrl+D` | Delete selected node (Node Editor) |
| `Ctrl+A` | Select all nodes (Node Editor) |
| `Ctrl+C/V` | Copy/paste nodes (Node Editor) |

---

## Building for Production

```bash
npm run tauri:build
```

Creates platform-specific installers:
- **macOS**: `.dmg` + `.app` bundle
- **Windows**: `.msi` + `.exe` installer
- **Linux**: `.deb` + `.AppImage`

---

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

Check the [Phase Plan](docs/specs/phase-plan.md) for open tasks, or look at issues labeled `good first issue`.

---

## License

MIT

---

<p align="center">
  <strong>AI Studio</strong> — See everything. Control everything. Spend less.<br>
  Built with Tauri 2, React 19, and Python.
</p>
