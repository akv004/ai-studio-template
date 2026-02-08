<p align="center">
  <img src="docs/screenshots/screenshot-1-main-dashboard.png" alt="AI Studio" width="800">
</p>

<h1 align="center">AI Studio</h1>

<p align="center">
  <strong>The open-source IDE for AI agents.</strong><br>
  Build, run, debug, and compare AI agents — with full visibility into every decision and dollar spent.
</p>

<p align="center">
  <a href="#what-can-you-do-with-ai-studio">What It Does</a> &middot;
  <a href="#why-ai-studio">Why AI Studio</a> &middot;
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

## What can you do with AI Studio?

**See everything your AI agent does.**

> You say: "Refactor the auth module to use JWT"
>
> The agent reads 3 files, installs a package, writes 2 files. You see every step in the Inspector — every tool call, every file read, every approval decision. The agent used 2,341 tokens and cost you $0.012. It took 4.8 seconds.
>
> Something wrong? Click any step. See the exact input and output. Click **Branch from here**. Try a different approach. Compare results side-by-side.

**Hybrid Intelligence — the right model for each step.**

> Simple question? Runs on your local Llama (free, private). Complex code? Routes to Claude Sonnet. Need vision? GPT-4o. You set a monthly budget — AI Studio picks the best model automatically and never surprises you with a bill.

**Control what your agent can do.**

> Read files? Auto-approved. Write files? You approve each one. `rm -rf`? Blocked. Always. Every decision logged and auditable.

**Compare models side-by-side.**

> Same prompt. Claude vs GPT vs Gemini vs local Llama. Compare output quality, speed, and cost in a split-screen view. Make data-driven decisions.

**Run agents at scale.**

> Batch 50 tasks. Auto-approve safe operations. See which ones failed and why. Export results. Total cost: $0.58.

---

## Why AI Studio?

| | ChatGPT / Claude | OpenClaw | Cursor | **AI Studio** |
|---|---|---|---|---|
| Can it use tools (files, shell, APIs)? | No | Yes | Limited | **Yes (MCP + built-in)** |
| Can you see what it did? | No | No | No | **Full Inspector** |
| Can you replay / branch? | No | No | No | **Yes** |
| Can you compare models? | No | No | No | **Side-by-side** |
| Hybrid intelligence (auto-pick model)? | No | No | No | **Yes** |
| Cost tracking per message? | No | No | No | **Yes** |
| Approval rules for safety? | N/A | Minimal | N/A | **Full rules engine** |
| Local-first (data stays on machine)? | No | Yes | No | **Yes** |
| Open source and free? | No | Yes | No | **Yes** |

---

## Features

### Agent Inspector (Flagship)

The **Chrome DevTools for AI agents**. No other tool gives you this level of visibility:

- **Event Timeline** — every message, LLM call, tool execution as a navigable timeline
- **Tool Call Deep-Dive** — click any tool call to see input, output, approval status, duration
- **Token & Cost Tracking** — per-turn and cumulative, with model pricing
- **Replay** — re-run from any point with the same or modified context
- **Branch & Compare** — fork from any point, try different models, diff results side-by-side
- **Export** — full session as JSON or Markdown

### Hybrid Intelligence

AI Studio auto-picks the best model for each step:

- **Simple questions** &rarr; Local Llama (free, private, fast)
- **Complex code** &rarr; Claude Sonnet (best at code)
- **Vision tasks** &rarr; GPT-4o (strong vision)
- **Large context** &rarr; Gemini Flash (1M context, cheap)
- **Budget controls** — set monthly limits, per-agent budgets, automatic fallback to local when budget runs low

### MCP-Native Tools

Built on [Model Context Protocol](https://modelcontextprotocol.io/) — the open standard for AI tool integration:

- Connect any MCP server (GitHub, Postgres, Filesystem, Brave Search, and growing)
- Built-in tools: shell, filesystem, browser
- One-click setup from curated server list
- Full approval workflow for every tool call

### 5 Focused Modules

| Module | What It Does |
|--------|-------------|
| **Agents** | Create and configure AI agents (model, prompt, tools, permissions) |
| **Sessions** | Interactive chat with real-time tool approval and streaming |
| **Runs** | Headless batch execution — CI/CD for agents |
| **Inspector** | Deep-dive into any session: timeline, traces, cost, replay |
| **Settings** | Providers, MCP servers, budget, preferences |

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

<details>
<summary><strong>Sidecar API Examples</strong> (for development and testing)</summary>

```bash
# Chat with conversation memory
curl -X POST http://localhost:8765/chat \
  -H "Content-Type: application/json" \
  -d '{"message": "Hello!", "provider": "ollama"}'

# List available providers
curl http://localhost:8765/providers

# Health check
curl http://localhost:8765/status

# Execute shell command (via tool)
curl -X POST http://localhost:8765/tools/shell \
  -H "Content-Type: application/json" \
  -d '{"command": "ls -la"}'

# Read a file (via tool)
curl -X POST http://localhost:8765/tools/filesystem \
  -H "Content-Type: application/json" \
  -d '{"action": "read", "path": "README.md"}'
```

Swagger docs available at `http://localhost:8765/docs` when sidecar is running.

</details>

---

## Architecture

```
┌──────────────────────────────────────────────────┐
│              UI Layer (React + Tauri)             │
│  Agents  |  Sessions  |  Runs  |  Inspector      │
└────────────────────┬─────────────────────────────┘
                     │ Tauri IPC
┌────────────────────┴─────────────────────────────┐
│            Desktop Layer (Rust/Tauri)             │
│  SQLite DB  |  Tool Approval  |  Event Bridge    │
└────────────────────┬─────────────────────────────┘
                     │ HTTP + WebSocket
┌────────────────────┴─────────────────────────────┐
│            Agent Layer (Python Sidecar)           │
│  LLM Providers  |  MCP Client  |  Event Emitter  │
└──────────────────────────────────────────────────┘
```

**3 layers, clear responsibilities:**

- **UI (React)** — what you see. Never talks to the sidecar directly.
- **Desktop (Rust/Tauri)** — security boundary. Owns persistence (SQLite), tool approval, event bridging.
- **Agent (Python)** — does the work. Calls LLMs, executes tools, emits events.

**Key design decisions:**
- **Local-first**: All data in SQLite on your machine. No cloud, no account.
- **Event-sourced**: Every agent action is a typed event. Inspector reads from the event log.
- **MCP-native**: Tools are MCP servers. Interoperable with Claude Desktop, Cursor, Zed, etc.
- **Hybrid intelligence**: Smart router picks the best model per step with budget awareness.

### Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop Shell | Tauri 2.0 (Rust) |
| UI | React 19 + TypeScript + Vite 7 |
| Styling | Tailwind CSS 4 |
| State | Zustand |
| Database | SQLite (via rusqlite) |
| AI Backend | FastAPI (Python 3.10+) |
| LLM Providers | Ollama, Anthropic, OpenAI, Google AI |
| Tool System | Model Context Protocol (MCP) |

---

## Project Structure

```
ai-studio-template/
├── apps/
│   ├── ui/                  # React frontend (5 modules)
│   ├── desktop/src-tauri/   # Tauri/Rust (persistence, security, IPC)
│   └── sidecar/             # Python (LLM providers, MCP, tools, events)
├── packages/
│   └── shared/              # Shared TypeScript types
├── docs/
│   └── specs/               # 11 design specifications
└── package.json             # Monorepo workspace config
```

---

## Design Specs

The product is designed spec-first. Every feature has a detailed specification:

| Spec | Covers |
|------|--------|
| [Product Vision](docs/specs/product-vision.md) | Direction, pillars, competitive positioning |
| [Architecture](docs/specs/architecture.md) | 3-layer system, data flows, security |
| [Event System](docs/specs/event-system.md) | Typed events, transport, cost calculation |
| [Data Model](docs/specs/data-model.md) | SQLite schema, migrations, branching |
| [Agent Inspector](docs/specs/agent-inspector.md) | Flagship: timeline, replay, branch, export |
| [MCP Integration](docs/specs/mcp-integration.md) | Tool system, discovery, approval |
| [Hybrid Intelligence](docs/specs/hybrid-intelligence.md) | Smart routing, budget, savings |
| [API Contracts](docs/specs/api-contracts.md) | Every IPC command, endpoint, WebSocket |
| [UI Design](docs/specs/ui-design.md) | Wireframes, colors, interactions |
| [Use Cases](docs/specs/use-cases.md) | Real-world scenarios, demo script |
| [Phase Plan](docs/specs/phase-plan.md) | Implementation roadmap (Phase 0-3) |

---

## Roadmap

| Phase | Focus | Status |
|-------|-------|--------|
| **Phase 0** | Specs + codebase cleanup | Done |
| **Phase 1** | Core loop — agents, sessions, persistence, events, basic inspector, MCP | In progress |
| **Phase 2** | Power features — full inspector (replay, branch, compare), runs, hybrid intelligence | Planned |
| **Phase 3** | Ecosystem — plugins, templates, one-click installer, community launch | Planned |

See [Phase Plan](docs/specs/phase-plan.md) for the full task breakdown.

---

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd+K` | Command Palette |
| `Cmd+1-4` | Navigate modules |
| `Cmd+,` | Settings |
| `Cmd+N` | New agent |
| `Cmd+Shift+N` | New session |
| `Cmd+I` | Inspector for current session |
| `Cmd+Enter` | Send message |
| `Cmd+Shift+Enter` | Approve tool request |

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

## Troubleshooting

| Issue | Fix |
|-------|-----|
| `cargo: command not found` | Run `source ~/.cargo/env` or restart terminal |
| Port 1420 already in use | `lsof -ti:1420 \| xargs kill -9` |
| First Rust build is slow | Normal (~3-5 min). Subsequent builds are instant. |
| Sidecar won't start | Check Python 3.10+ installed, `pip install -r apps/sidecar/requirements.txt` |

---

## Contributing

We welcome contributions! Check the [Phase Plan](docs/specs/phase-plan.md) for open tasks, or look at issues labeled `good first issue`.

---

## License

MIT

---

<p align="center">
  <strong>AI Studio</strong> — See everything. Control everything. Spend less.<br>
  Built with Tauri, React, and Python.
</p>
