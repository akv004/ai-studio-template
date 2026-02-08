# AI Studio — Product Vision

> **Version**: 2.1 (restructured + hybrid intelligence + UI vision)
> **Status**: Draft
> **Supersedes**: competitive-roadmap.md, agent-roadmap.md (these remain as historical references)

---

## One-Line Pitch

**AI Studio is a desktop-native IDE for AI agents — build, run, debug, and compare agents across any model, with full visibility into every decision and dollar spent.**

---

## The Problem

The AI agent ecosystem has a massive UX gap:

| Existing Tool | What It Does Well | What It Lacks |
|---|---|---|
| **OpenClaw** (145K+ stars) | Autonomous agents in messaging apps. Actually does things. | Black box — no visibility into what agent did. Security concerns. No cost tracking. No debugging. |
| **Claude Code / Cursor** | Great coding assistants | Single-model, no tool visibility, no replay, can't compare models |
| **LangChain / CrewAI** | Powerful frameworks | Code-only, no visual debugging, painful iteration cycles |
| **LangSmith** | Tracing & monitoring | Cloud-only, paid, no local-first, no replay/branching |
| **LM Studio / Jan.ai** | Easy local model UIs | Chat-only — no agent orchestration, no tools, no pipelines |

**The gap**: No tool gives developers a **visual, interactive control plane** for AI agents with hybrid intelligence (auto-pick best model), full debugging, and cost control.

---

## The Solution

AI Studio is a **desktop-native control plane** where developers:

1. **Define** agents (system prompt, model, provider, tools, permissions)
2. **Run** agents interactively or headlessly
3. **Inspect** every decision — token-by-token, tool-by-tool, with cost tracking
4. **Debug** by replaying, branching, and diffing agent sessions
5. **Automate** with batch runs and event-driven pipelines

Think of it as **Chrome DevTools meets an AI agent runtime — with the intelligence to pick the right model for each step automatically.**

---

## Target User

**Primary**: Developers who work with AI agents and want a GUI-first alternative to terminal-based tools.

These users:
- Understand APIs, models, prompts, and tool use
- Want visibility into what agents are doing (not just chat output)
- Value local-first privacy and control
- Expect professional-grade UX (keyboard shortcuts, fast navigation, dark mode)
- May use multiple LLM providers (local Ollama + cloud APIs)

**Not targeting (for now)**: Non-technical users, enterprise teams, AI researchers doing training/fine-tuning.

---

## The 5 Pillars

AI Studio is organized around 5 focused modules. Everything that doesn't serve these pillars is out of scope for the core product.

### 1. Agents

> Define agents as structured configurations.

An agent is a combination of: system prompt + model/provider + available tools + permissions + behavior rules. Agents are the "source code" of AI Studio.

**Key capabilities:**
- Create, edit, duplicate, delete agents
- Configure: system prompt, model, provider, temperature, max tokens
- Assign tools (MCP servers, built-in tools)
- Set permission levels (auto-approve patterns, require approval, deny)
- Version history (track changes to agent configs over time)
- Import/export as portable JSON

### 2. Sessions

> Interactive, real-time conversations with agents.

Sessions are where work happens. A session binds a user to an agent and records everything.

**Key capabilities:**
- Chat with any configured agent
- Real-time tool approval workflow (approve/deny/auto-approve)
- Streaming responses with token-by-token display
- Conversation branching (fork from any message, explore alternatives)
- Session persistence (resume where you left off)
- Multi-session management (multiple sessions open simultaneously)
- Inline artifact display (code blocks, images, files created by tools)

### 3. Runs

> Headless, batch, or scheduled agent execution.

Runs are sessions without a human in the loop (or with minimal intervention). They're the "CI/CD for agents."

**Key capabilities:**
- Execute an agent with a predefined prompt/task
- Batch runs (same agent, multiple inputs)
- Auto-approval rules (pre-configured for headless execution)
- Full event recording (same fidelity as interactive sessions)
- Run status tracking (pending, running, completed, failed)
- Artifact collection (files created, commands run, outputs generated)
- Trigger mechanisms (manual, API call, future: cron/webhook)

### 4. Inspector (Flagship Feature)

> Chrome DevTools for AI agents.

The Inspector is AI Studio's primary differentiator. It provides deep, visual insight into any session or run.

**Key capabilities:**
- **Event Timeline**: Every message, thought, tool call, and approval rendered as a navigable, zoomable timeline
- **Token & Cost Tracking**: Per-turn and cumulative token counts, cost breakdowns by model/provider
- **Tool Call Deep-Dive**: For each tool invocation — input, output, approval status, duration, exit codes
- **Replay**: Re-execute from any point in the timeline with the same or modified context
- **Branch & Compare**: Fork from any point, run alternatives, diff results side-by-side
- **Performance Metrics**: Latency per turn, time-to-first-token, total session duration
- **Export**: Full session data as JSON (machine-readable) or Markdown (human-readable)
- **Search & Filter**: Find specific tool calls, errors, or patterns across a session
- **Trace View**: Nested view showing agent reasoning → tool selection → execution → response incorporation

### 5. Settings

> Infrastructure configuration and preferences.

**Key capabilities:**
- Provider management (add/configure Ollama, Anthropic, OpenAI, Google AI)
- Model discovery and status checking
- MCP server configuration
- Global preferences (theme, keyboard shortcuts, default agent settings)
- Tool security defaults (global tool mode: sandboxed/restricted/full)
- Data management (storage location, export/import all data, clear data)

---

## What Is Explicitly Out of Scope

These features are **not** part of the core product. They may return as plugins once the plugin system exists.

| Feature | Reason for Exclusion |
|---|---|
| Vision / Video detection | Separate product domain. Plugin candidate. |
| Audio processing | Separate product domain. Plugin candidate. |
| Training / Fine-tuning | Different user workflow entirely. |
| Visual workflow graph builder | High effort, not needed for core agent UX. Revisit in Phase 3+. |
| Channel integrations (Telegram, Discord, Slack) | Keep the sidecar code, but not in the desktop UI. Separate deployment. |
| Teams / Collaboration | Requires account system, cloud sync. Future product expansion. |
| Plugin marketplace | Design for it, don't build it yet. |

---

## Design Principles

### 1. Depth Over Breadth
Build fewer features, but make each one the best in its category. The Inspector should be so good that developers choose AI Studio just for that.

### 2. Local-First, Always
All data on disk. No account required. No telemetry by default. Cloud sync is a future opt-in, never a requirement.

### 3. MCP-Native
The tool system is built on Model Context Protocol from day one. AI Studio is an MCP client. Tools are MCP servers. This makes the ecosystem interoperable — any MCP-compatible tool works immediately.

### 4. Event-Sourced Architecture
Every agent action emits a typed event. The event log is the source of truth. The Inspector reads from it. Runs produce it. Sessions produce it. This is the architectural spine.

### 5. Keyboard-First UX
Every action reachable by keyboard. Command palette (Cmd+K). Navigation shortcuts. Inline search. Developers live in their keyboards.

### 6. Hybrid Intelligence
The right model for each step, automatically. Simple questions go to local Llama (free). Complex code goes to Claude (best). Vision goes to GPT-4o. Budget controls ensure no surprise bills. No other tool does this.

### 7. Zero Config to First Value
`npm run tauri:dev` → app opens → pick a provider → start chatting. No config files, no YAML, no Docker required for basic usage.

---

## Competitive Positioning

### vs. OpenClaw (145K stars)
OpenClaw is messaging-first (WhatsApp, Telegram). Agents run autonomously. Huge ecosystem. But: **black box** — you can't see what the agent did. No cost tracking. Security researchers are raising alarms about its broad permissions. AI Studio is the opposite: full visibility, full control, full audit trail.

**Positioning: "OpenClaw lets agents run wild. AI Studio lets you see and control everything."**

### vs. LM Studio / Jan.ai
Model runners with chat UIs. AI Studio is an **agent runtime** — it has tools, approval workflows, session management, hybrid intelligence, and deep inspection. Different category.

### vs. LangSmith
Cloud-only tracing and monitoring. Paid. AI Studio is **local-first, free, and open-source** with richer debugging (branching, replay, compare). LangSmith shows traces. AI Studio lets you replay and branch from any point.

### vs. Claude Code / Cursor
Coding-focused, single-model. AI Studio is agent-focused with **hybrid intelligence** — auto-picks the best model per step. The Inspector provides visibility these tools don't offer. You can compare Claude vs GPT side-by-side.

### The Moat
**Hybrid Intelligence + Inspector + MCP + Local-first** — this combination doesn't exist anywhere else:
- Hybrid: auto-route across local + cloud models with budget control (no one does this)
- Inspector: replay, branch, compare any session (no one does this for free)
- MCP: interoperable with the entire growing tool ecosystem
- Local-first: your data never leaves your machine

---

## Success Metrics

| Metric | Target | Why It Matters |
|---|---|---|
| Time to first agent run | < 3 minutes from install | Adoption depends on fast onboarding |
| Inspector opens per session | > 50% of sessions inspected | Proves the flagship feature has pull |
| Agent configs created per user | > 3 | Shows users are investing in the tool |
| Session replay usage | > 20% of sessions replayed | Validates the debugging value prop |
| GitHub stars (3 months) | 1,000+ | Community traction signal |
| Active weekly users (6 months) | 500+ | Retention signal |

---

## Phased Delivery (Summary)

Detailed phase plan will be in a separate spec (`phase-plan.md`). High-level:

| Phase | Focus | Outcome |
|---|---|---|
| **0: Foundation** | Event system, data model, API contracts, MCP integration design | All specs written, architecture locked |
| **1: Core Loop** | Agent CRUD, Sessions (real chat), basic Inspector, SQLite persistence | Usable product for early adopters |
| **2: Power Features** | Full Inspector (replay, branch, diff, cost), Runs, auto-approval rules, export/import | Feature-complete for developers |
| **3: Ecosystem** | Plugin system, MCP server marketplace, community templates, one-click installer | Growth-ready product |

---

## Relationship to Existing Specs

| Existing Spec | Status |
|---|---|
| `competitive-roadmap.md` | **Superseded** by this document. Retain as historical reference. |
| `agent-roadmap.md` | **Superseded** by this document + `phase-plan.md`. Retain as historical reference. |
| `future-capabilities.md` | **Partially superseded**. Event bus contract (Section: "Contract: Event Bus") is preserved and expanded in `event-system.md`. Video/detection sections are deferred (out of scope for core). Plugin architecture section is deferred to Phase 3. |
| `tools-api.md` | **Evolving**. Tool system will be redesigned around MCP in `mcp-integration.md`. Current REST tool endpoints remain for backward compatibility during transition. |

---

## Complete Spec Index

| # | Spec | What It Covers |
|---|---|---|
| 1 | `product-vision.md` | This document — direction, pillars, positioning |
| 2 | `architecture.md` | 3-layer system, data flows, security, migration |
| 3 | `event-system.md` | Typed event bus, envelope schema, transport, cost |
| 4 | `data-model.md` | SQLite schema, tables, branching, migrations |
| 5 | `agent-inspector.md` | Flagship: timeline, detail panel, replay, export |
| 6 | `mcp-integration.md` | MCP client, tool discovery, approval flow |
| 7 | `api-contracts.md` | Every IPC command, REST endpoint, WebSocket |
| 8 | `hybrid-intelligence.md` | Smart model routing, budget controls, savings |
| 9 | `ui-design.md` | Visual design, wireframes, first-run experience |
| 10 | `use-cases.md` | Real-world scenarios, README hooks, demo script |
| 11 | `phase-plan.md` | Phase 0-3 task breakdown and build order |
