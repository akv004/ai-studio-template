# Show HN Draft

> **Title**: Show HN: AI Studio – Open-source desktop IDE for AI agents (Tauri + React + Python)

---

**Post body:**

Hi HN,

I built AI Studio — a desktop IDE for building, running, and debugging AI agents. Think "Chrome DevTools for AI agents."

**The problem**: When working with AI agents (Claude, GPT, Gemini, local models), you have zero visibility into what's happening. Why did it pick that tool? How much did that conversation cost? Can I replay from step 3 with a different model?

**What AI Studio does:**

- **Agent Inspector** — event timeline showing every LLM call, tool execution, and decision. Click any event to see exact input/output. Fork from any point to try different approaches.

- **Node Editor** — visual pipeline builder ("Unreal Blueprints for AI agents"). Connect LLM, Tool, Router, and Approval nodes to build workflows. DAG execution engine runs them with parallel branches. 10 bundled templates.

- **Hybrid Intelligence** — auto-picks the best model per step. Simple questions → local Llama (free). Complex code → Claude. Large context → Gemini. With monthly budget controls.

- **MCP-native tools** — built on Model Context Protocol. Connect any MCP server (GitHub, Postgres, Brave Search). Full approval workflow for every tool call.

- **Local-first** — all data in SQLite on your machine. No cloud account, no telemetry.

**Stack**: Tauri 2 (Rust) + React 19 + Python FastAPI sidecar. The Rust layer handles persistence, security, workflow execution, and smart routing. Python handles LLM providers and tool execution.

**What's built**: 5 LLM providers, 10 workflow templates, 8 node types, session branching, headless batch runs, plugin system, 31 unit tests, cross-platform installers.

GitHub: https://github.com/akv004/ai-studio-template

I'd love feedback on the architecture and what features would matter most to you.

---

**HN guidelines notes:**
- Keep under 300 words
- Lead with the problem
- Show technical depth but stay accessible
- End with a question to encourage discussion
- Link to repo prominently
