# Contributing to AI Studio

Thanks for your interest in contributing to AI Studio! This guide will help you get set up and make your first contribution.

## Prerequisites

| Tool | Version | Install |
|------|---------|---------|
| Node.js | 18+ | [nodejs.org](https://nodejs.org/) |
| Rust | Latest stable | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| Python | 3.10+ | [python.org](https://www.python.org/) |
| pnpm or npm | Latest | `npm install -g pnpm` (optional) |

## Setup

```bash
# Clone your fork
git clone https://github.com/<your-username>/ai-studio-template.git
cd ai-studio-template

# Install JS dependencies
npm install

# Install Python sidecar dependencies
cd apps/sidecar
python -m venv .venv
source .venv/bin/activate  # Windows: .venv\Scripts\activate
pip install -r requirements.txt
cd ../..

# Run the full desktop app
cd apps/desktop/src-tauri && cargo tauri dev
```

First Rust build takes ~3-5 minutes. After that, rebuilds are fast.

### Development Modes

| Mode | Command | Use When |
|------|---------|----------|
| Full desktop | `cd apps/desktop/src-tauri && cargo tauri dev` | End-to-end testing |
| UI only | `npm run dev` | Frontend/styling work (uses mock data) |
| Sidecar only | `cd apps/sidecar && python -m uvicorn server:app --port 8765` | API/provider work |

## Project Structure

```
ai-studio-template/
├── apps/
│   ├── ui/                  # React 19 + TypeScript + Tailwind
│   │   └── src/
│   │       ├── app/pages/   # 5 modules: Agents, Sessions, Runs, Inspector, Settings
│   │       ├── state/       # Zustand store (all IPC calls to Tauri)
│   │       └── types/       # TypeScript types
│   ├── desktop/src-tauri/   # Rust/Tauri — SQLite, IPC commands, event bridge
│   │   └── src/
│   │       ├── commands.rs  # All Tauri IPC command handlers
│   │       ├── db.rs        # SQLite schema + migrations
│   │       └── events.rs    # WebSocket bridge to sidecar
│   └── sidecar/             # Python FastAPI — LLM providers, MCP, tools
│       ├── server.py        # FastAPI app entry point
│       └── agent/           # Providers, chat, MCP client
├── packages/shared/         # Shared TypeScript types
├── docs/specs/              # 11 design specifications
└── package.json             # Monorepo workspace config
```

### Architecture (3 layers)

```
UI (React) → Tauri IPC → Rust/Tauri (SQLite, security) → HTTP/WS → Python Sidecar (LLMs, tools)
```

- **UI never talks to sidecar directly** — all communication goes through Tauri IPC
- **Tauri owns persistence** (SQLite) and security (tool approval, auth tokens)
- **Sidecar is stateless** — calls LLMs, executes tools, emits events

## How to Contribute

### Finding Work

1. Check [Issues](https://github.com/akv004/ai-studio-template/issues) for `good first issue` or `help wanted` labels
2. Check the [Phase Plan](docs/specs/phase-plan.md) for the current roadmap
3. Read the relevant [spec](docs/specs/) before starting any feature work

### Contribution Workflow

1. **Fork** the repository
2. **Create a branch** from `main`: `git checkout -b feat/my-feature`
3. **Read the spec** — check `docs/specs/` for the relevant specification
4. **Make your changes** — small, focused commits
5. **Test** — make sure the app builds and runs
6. **Push** to your fork and open a **Pull Request**

### Branch Naming

| Prefix | Use For |
|--------|---------|
| `feat/` | New features |
| `fix/` | Bug fixes |
| `docs/` | Documentation |
| `refactor/` | Code restructuring |

### Commit Messages

Format: `Type: Description`

```
Feat: Add cost breakdown chart to Inspector
Fix: Handle empty provider response in chat
Docs: Update architecture spec with event flow
Refactor: Extract timeline event component
```

## Coding Standards

### TypeScript (UI)

- React 19 with functional components only
- TypeScript strict mode — avoid `any`
- Named exports preferred
- Zustand for client state, Tauri IPC for persistence
- Tailwind CSS for styling
- Lucide React for icons

### Rust (Tauri)

- All IPC commands in `commands.rs`
- Snake_case in Rust becomes camelCase in JS (Tauri v2 auto-converts)
- Use `Arc<Mutex<Connection>>` for shared SQLite access
- Always return `Result<T, String>` from commands

### Python (Sidecar)

- FastAPI with Pydantic v2 models
- Type hints on all function signatures
- Async for I/O-bound operations
- Line length: 100 chars

### General Rules

- Follow existing patterns in the codebase
- Keep changes focused — don't refactor unrelated code
- No new dependencies without discussion in the PR
- Test with at least one LLM provider (Ollama is free and local)

## Architecture Decisions

Before proposing significant architectural changes, read:

- [Architecture](docs/specs/architecture.md) — system design and layer responsibilities
- [Event System](docs/specs/event-system.md) — the event-sourced backbone
- [Data Model](docs/specs/data-model.md) — SQLite schema

All architecture decisions are logged in `STATUS.md` under "Decisions Log."

## Specs

AI Studio is spec-driven. Every major feature has a design specification in `docs/specs/`.

| When | Do |
|------|----|
| Building a specced feature | Read the spec first, follow it |
| Spec doesn't match reality | Flag it in the PR — we'll update the spec |
| Building something new without a spec | Open an issue to discuss the approach first |

## Getting Help

- Open an [Issue](https://github.com/akv004/ai-studio-template/issues) for bugs or feature requests
- Check existing specs in `docs/specs/` for design context
- Read `STATUS.md` for current project state and decisions

## License

By contributing, you agree that your contributions will be licensed under the [MIT License](LICENSE).
