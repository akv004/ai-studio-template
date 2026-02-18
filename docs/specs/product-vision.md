# AI Studio — Product Vision

> **Version**: 3.0 (universal automation canvas + expanded node vision)
> **Status**: Draft
> **Supersedes**: competitive-roadmap.md, agent-roadmap.md (these remain as historical references)

---

## One-Line Pitch

**AI Studio is a visual automation platform where AI is a first-class building block — connect any input, any model, any tool, any output, and watch data flow through your pipeline in real-time.**

---

## The Problem

Two gaps exist in tooling today:

**Gap 1 — AI Agent tooling**: No tool gives developers a visual, interactive control plane for AI agents with hybrid intelligence, full debugging, and cost control.

| Existing Tool | What It Does Well | What It Lacks |
|---|---|---|
| **OpenClaw** (145K+ stars) | Autonomous agents in messaging apps | Black box — no visibility, no cost tracking, no debugging |
| **Claude Code / Cursor** | Great coding assistants | Single-model, no tool visibility, no replay |
| **LangChain / CrewAI** | Powerful frameworks | Code-only, no visual debugging, painful iteration |
| **LangSmith** | Tracing & monitoring | Cloud-only, paid, no local-first |
| **LM Studio / Jan.ai** | Easy local model UIs | Chat-only — no orchestration, no pipelines |

**Gap 2 — Visual automation**: Existing automation platforms (n8n, Node-RED, Make.com) connect systems visually but treat AI as an afterthought — a single "OpenAI" node bolted on. None of them have hybrid model routing, cost tracking per node, approval gates, or an Inspector that replays every decision.

**The intersection**: No tool exists that is both a **visual automation canvas** (connect anything to anything) AND an **AI-native runtime** (smart model routing, full observability, human-in-the-loop). AI Studio fills this gap.

---

## The Solution

AI Studio is a **visual automation platform with AI as a first-class building block**.

The Node Editor is the core experience. Users build pipelines by connecting nodes on a canvas:

1. **Any input** — text prompts, SQL queries, file reads, webhook payloads, IoT sensor data, message queue consumers
2. **Any processing** — LLM inference (any provider/model), data transforms, conditional routing, human approval gates, code execution
3. **Any output** — text results, file writes, database inserts, API calls, message queue publish, IoT device commands, email/Slack notifications
4. **Full visibility** — every node shows its I/O, cost, latency in real-time. The Inspector replays any execution step-by-step.
5. **Hybrid intelligence** — the platform auto-picks the right model per step based on task complexity and budget

Think of it as **Node-RED meets ComfyUI — a universal automation canvas where AI nodes sit alongside database nodes, file nodes, and API nodes, all with full observability and cost control.**

### What Makes This Different from n8n/Node-RED/Make.com

| Capability | n8n / Node-RED | AI Studio |
|---|---|---|
| AI/LLM support | Single "OpenAI" node, no routing | Multi-provider, hybrid routing, cost per node |
| Model comparison | Not possible | Fan-out to N models, compare side-by-side |
| Observability | Basic logs | Inspector — replay, branch, diff any execution |
| Approval gates | Not native | First-class human-in-the-loop gates |
| Cost tracking | None | Per-node, per-model, budget controls |
| Local-first | Cloud-hosted (mostly) | All data on disk, no account required |
| Plugin architecture | Proprietary | MCP-native (open standard) |

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
| ~~Visual workflow graph builder~~ | **Promoted to Phase 3** — see `node-editor.md`. React Flow + DAG execution engine. |
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

### vs. n8n / Node-RED / Make.com (Automation Platforms)
These are workflow automation tools with hundreds of integrations. They're great at connecting SaaS services. But they treat AI as a bolt-on — a single "OpenAI" node. No model routing, no cost tracking, no approval gates, no Inspector. AI Studio is **AI-native automation**: every node in the pipeline has full observability, and the platform intelligently routes between models.

**Positioning: "n8n connects your apps. AI Studio connects your apps AND your AI — with full visibility into every decision and dollar spent."**

### vs. ComfyUI / Langflow (AI Visual Builders)
ComfyUI is image generation focused. Langflow is LLM-chain focused. Both are narrow. AI Studio is a **universal automation canvas** — SQL databases, file systems, message queues, APIs, IoT devices are all node types alongside LLM and tool nodes. Plus: desktop-native, local-first, with an Inspector that no visual builder offers.

### vs. OpenClaw (145K stars)
Messaging-first, autonomous agents. Black box — you can't see what the agent did. AI Studio is the opposite: full visibility, full control, full audit trail. Agents are visual pipelines you can inspect step-by-step.

### vs. LangSmith
Cloud-only tracing. Paid. AI Studio is **local-first, free, and open-source** with richer debugging (branching, replay, compare). LangSmith shows traces. AI Studio lets you replay and branch from any point.

### vs. Claude Code / Cursor
Coding-focused, single-model. AI Studio is a visual automation platform with **hybrid intelligence** — auto-picks the best model per step. Compare Claude vs GPT side-by-side on the same pipeline.

### The Moat
**Universal Canvas + AI-Native + Inspector + Local-first** — this combination doesn't exist:
- Universal: any input source → any processing → any output destination (not just LLM chains)
- AI-native: hybrid model routing, cost per node, approval gates (not a bolt-on OpenAI node)
- Inspector: replay, branch, compare any execution (no one does this for free)
- MCP: interoperable with the growing tool ecosystem
- Local-first: your data never leaves your machine
- Plugin system: community-contributed node types via open standard

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

Detailed phase plan in `phase-plan.md`. High-level:

| Phase | Focus | Outcome |
|---|---|---|
| **0: Foundation** | Event system, data model, API contracts, MCP integration design | All specs written, architecture locked |
| **1: Core Loop** | Agent CRUD, Sessions (real chat), basic Inspector, SQLite persistence | Usable product for early adopters |
| **2: Power Features** | Full Inspector (replay, branch, diff, cost), Runs, auto-approval rules | Feature-complete for developers |
| **3: Node Editor** | Visual pipeline builder (3A canvas, 3B execution, 3C polish + templates) | AI workflow automation |
| **4: Universal Canvas** | I/O connector nodes (database, file, HTTP, queue, IoT), plugin system | Connect anything to anything |
| **5: Community** | Plugin marketplace, community templates, one-click installers | Growth-ready product |

### Phase 4 Vision: Universal Automation Canvas

Phase 4 transforms the Node Editor from an "AI workflow builder" into a **universal automation platform**. The architecture already supports this — each node type is just an executor function. Phase 4 adds connector nodes:

**New Input Node Types:**
- `database_read` — SQL query against SQLite/PostgreSQL/MySQL, returns rows as JSON
- `file_read` — Read CSV, JSON, XML, text files from local filesystem
- `http_request` — GET/POST to any REST API, returns response
- `webhook_listen` — HTTP endpoint that triggers the workflow on incoming request
- `queue_consume` — Read from Kafka, RabbitMQ, Redis Streams, MQTT
- `iot_sensor` — Read from connected IoT devices (MQTT, serial, GPIO)
- `cron_trigger` — Time-based scheduled execution

**New Output Node Types:**
- `database_write` — INSERT/UPDATE/DELETE against any supported database
- `file_write` — Write CSV, JSON, text, PDF to local filesystem
- `http_post` — Send data to any REST/GraphQL API endpoint
- `queue_publish` — Push messages to Kafka, RabbitMQ, Redis Streams, MQTT
- `iot_command` — Send commands to IoT devices
- `notification` — Email, Slack, Discord, webhook notification
- `display` — Rich visual output (charts, tables, formatted reports)

**New Processing Node Types:**
- `code` — Execute Python/JavaScript snippets (sandboxed)
- `validator` — JSON Schema validation, data quality checks
- `merge` — Wait for multiple branches and combine results (OR/AND logic)
- `loop` — Iterate over array input, execute subgraph per item
- `cache` — Memoize expensive operations (LLM calls, API requests)
- `rate_limiter` — Throttle execution rate for API compliance

**What stays the same:**
- Every node emits events → Inspector shows full pipeline execution
- Every node tracks cost → budget controls apply universally
- Approval gates work anywhere in the pipeline
- Hybrid intelligence routes LLM nodes automatically
- All data stays local-first on disk

**Plugin system enables community node types** — anyone can build a node type (e.g., "Snowflake connector", "Stripe payments", "Arduino GPIO") and share it. The node is a Rust executor + React component + JSON schema.

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
| 12 | `node-editor.md` | Visual pipeline builder — the 10k-star feature (Phase 3) |
