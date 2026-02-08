# AI Studio — Use Cases & Scenarios

> These are real-world examples of what AI Studio does.
> Use these for: README, demo video, Product Hunt, Show HN, social media.

---

## Scenario 1: "Why did my agent do that?"

### The Problem Every Developer Has

You're using an AI agent. It made 7 tool calls, read 3 files, ran 2 shell commands, and gave you an answer. But the answer is **wrong**. Where did it go wrong? Which tool call returned bad data? You have no idea. It's a black box.

### What AI Studio Does

```
You: "Refactor the auth module to use JWT instead of sessions"

Agent thinks...
  ├── reads src/auth/session.ts          (tool: filesystem_read)
  ├── reads src/auth/middleware.ts        (tool: filesystem_read)
  ├── reads package.json                  (tool: filesystem_read)
  ├── runs: npm install jsonwebtoken      (tool: shell) ← YOU APPROVED THIS
  ├── writes src/auth/jwt.ts              (tool: filesystem_write) ← YOU APPROVED THIS
  ├── writes src/auth/middleware.ts        (tool: filesystem_write) ← YOU APPROVED THIS
  └── responds: "Done! Here's what I changed..."

You open Inspector:

┌─ Timeline ──────────────────────────────────────────────┐
│                                                         │
│  ● message.user         "Refactor auth to JWT..."       │
│  ● llm.request          claude-sonnet → 2,341 tokens    │
│  ● tool: fs_read        src/auth/session.ts    12ms     │
│  ● tool: fs_read        src/auth/middleware.ts  8ms     │
│  ● tool: fs_read        package.json           6ms     │
│  ● tool: shell          npm install jsonweb... 3.2s ✓  │
│  ● tool: fs_write       src/auth/jwt.ts       14ms ✓  │
│  ● tool: fs_write       src/auth/middleware.ts 11ms ✓  │  ← click here
│  ● llm.response         "Done! Here's what..."         │
│                                                         │
│  Total: 2,341 in / 847 out │ $0.012 │ 4.8s            │
└─────────────────────────────────────────────────────────┘

You click the middleware.ts write. You see EXACTLY what was written.
You spot it: the agent forgot to remove the old session import.

You click [Branch from here] → edit the prompt → re-run.
Fixed in 30 seconds. Without the Inspector? You'd be staring at a diff for 10 minutes.
```

---

## Scenario 2: "Claude or GPT — which is better for my use case?"

### The Problem

You want to know: is Claude Sonnet or GPT-4o better at writing database migrations? You'd have to run both manually, copy-paste, compare. Tedious.

### What AI Studio Does

```
1. Create agent: "Migration Writer"
   - System prompt: "You write PostgreSQL migrations. Be precise."
   - Model: claude-sonnet-4-5
   - Tools: filesystem (read-only), shell (psql only)

2. Start session: "Add a users table with email, name, created_at"
   Agent writes the migration. Inspector shows: 1,200 tokens, $0.005, 2.1s

3. Click [Branch from here] on the original message
   Change model to: gpt-4o
   Same prompt runs again.

4. Open Compare View:

   ┌─────────── Claude ───────────┬─────────── GPT-4o ──────────┐
   │                              │                              │
   │  CREATE TABLE users (        │  CREATE TABLE users (        │
   │    id UUID PRIMARY KEY       │    id SERIAL PRIMARY KEY     │  ← different!
   │      DEFAULT gen_random_uuid │    email VARCHAR(255)        │
   │    email TEXT UNIQUE NOT NULL│      UNIQUE NOT NULL,        │
   │    name TEXT NOT NULL,       │    name VARCHAR(100)         │
   │    created_at TIMESTAMPTZ    │      NOT NULL,               │
   │      DEFAULT now()           │    created_at TIMESTAMP      │
   │  );                          │      DEFAULT CURRENT_TIMESTAMP│
   │                              │  );                          │
   │                              │                              │
   │  Tokens: 1,200 in / 340 out │  Tokens: 980 in / 290 out   │
   │  Cost: $0.005               │  Cost: $0.003                │
   │  Time: 2.1s                  │  Time: 1.8s                  │
   └──────────────────────────────┴──────────────────────────────┘

   Claude used UUID, GPT used SERIAL. Claude used TIMESTAMPTZ, GPT used TIMESTAMP.
   You pick the one that matches your stack. Data-driven decision in 60 seconds.
```

---

## Scenario 3: "Run it 50 times and tell me what breaks"

### The Problem

You built a prompt for generating API docs. Works fine on 3 examples. But will it work on your 50 endpoints? You're not going to click "send" 50 times.

### What AI Studio Does

```
Runs Page:

  Agent: "API Doc Writer"
  Input: (batch mode)
    - "Document GET /users"
    - "Document POST /users"
    - "Document GET /users/:id"
    - "Document PUT /users/:id"
    - ... (50 endpoints)

  Auto-approve: filesystem:read *   ← reads are safe
  Ask for: everything else

  [Start Batch Run]

  ┌──────────────────────────────────────────────────┐
  │  Run: API Doc Batch              Status: Running  │
  │                                                  │
  │  ████████████████░░░░░░░░░░░░░  34/50  (68%)    │
  │                                                  │
  │  ✓ 32 completed                                  │
  │  ✗ 2 failed                                      │
  │  ⏳ 16 remaining                                  │
  │                                                  │
  │  Tokens: 142,847 │ Cost: $0.58 │ Time: 3m 22s   │
  │                                                  │
  │  Failed:                                         │
  │  #12 "Document DELETE /admin/nuke" → tool denied │
  │  #28 "Document POST /upload" → timeout (60s)     │
  │                                                  │
  │  [View in Inspector] [Cancel] [Export Results]   │
  └──────────────────────────────────────────────────┘

  Click "View in Inspector" on #28 → see exactly where it got stuck.
  The agent tried to actually upload a file to test the endpoint.
  Fix the prompt: "Document the endpoint, do NOT test it."
  Re-run just the failed ones.
```

---

## Scenario 4: "I want my agent to use GitHub, Jira, and my database"

### The Problem

Your AI agent needs to: read GitHub issues, create Jira tickets, and query your Postgres database. In other tools, you'd write custom integrations or fight with plugin systems.

### What AI Studio Does

```
Settings → MCP Servers → [+ Add]

  ┌─────────────────────────────────────────────┐
  │  Popular MCP Servers                        │
  │                                             │
  │  [+ GitHub]        Issues, PRs, repos       │
  │  [+ Postgres]      Query your database      │
  │  [+ Brave Search]  Web search               │
  │  [+ Filesystem]    Read/write local files   │
  │                                             │
  │  Or add custom:                             │
  │  [+ Custom Server]                          │
  └─────────────────────────────────────────────┘

  Add GitHub → paste GITHUB_TOKEN → [Test] → 🟢 12 tools discovered
  Add Postgres → paste connection string → [Test] → 🟢 4 tools discovered

  Now create agent: "Project Manager Bot"
  - Model: Claude Sonnet
  - MCP Servers: ✅ GitHub, ✅ Postgres
  - Auto-approve: github:list_*, postgres:query (read-only)
  - Ask for: github:create_*, github:update_*

  Start session:
  You: "What are the open bugs? Cross-reference with our error_logs table."

  Agent:
    ├── github:list_issues { state: "open", label: "bug" }     ← auto-approved
    ├── postgres:query "SELECT * FROM error_logs WHERE..."      ← auto-approved
    └── responds with a correlated report

  Inspector shows every query, every API call, every response.
  You see exactly what SQL it ran. You see exactly which GitHub API it hit.
  Full audit trail.
```

---

## Scenario 5: "How much is this costing me?"

### The Problem

You're using Claude API. Your bill is $47 this month. Which agent is eating your budget? Which sessions were expensive? No idea.

### What AI Studio Does

```
Inspector → Any session:

  Stats Bar:
  ┌──────────────────────────────────────────────────────┐
  │  ◆ 12,847 input  ◆ 3,412 output  ◆ $0.067 cost     │
  │  ◆ 8 LLM calls   ◆ 14 tool calls  ◆ claude-sonnet   │
  └──────────────────────────────────────────────────────┘

  Every LLM call in the timeline shows:
  ┌──────────────────────────────────────┐
  │  llm.response.completed    seq=12   │
  │  Model: claude-sonnet-4-5           │
  │  Input: 2,341 tokens                │
  │  Output: 847 tokens                 │
  │  Cost: $0.012                       │
  │  Duration: 2.1s                     │
  │  TTFT: 340ms                        │
  └──────────────────────────────────────┘

  Sessions list shows cost per session.
  You can sort by cost → find the expensive ones → optimize prompts.
  You can compare branches: "this prompt costs $0.01, that one costs $0.05 for the same result."

  Local Ollama models show $0.00 cost. You see exactly what's free vs paid.
```

---

## Scenario 6: "I want to approve every dangerous command"

### The Problem

AI agents running `rm -rf`, `git push --force`, or `DROP TABLE` without asking. Terrifying.

### What AI Studio Does

```
Agent config → Approval Rules:

  ┌──────────────────────────────────────────────────────┐
  │  Approval Rules for "Code Assistant"                 │
  │                                                     │
  │  ✅ Auto-approve                                     │
  │     builtin:filesystem_read:*     (reading is safe)  │
  │     builtin:shell:git status      (read-only git)    │
  │     builtin:shell:git diff        (read-only git)    │
  │     builtin:shell:git log         (read-only git)    │
  │                                                     │
  │  ❓ Ask me                                           │
  │     builtin:shell:git commit *    (I want to review) │
  │     builtin:filesystem_write:*    (review all writes)│
  │     github:create_*               (review creates)   │
  │                                                     │
  │  🚫 Always deny                                      │
  │     builtin:shell:rm -rf *        (never)            │
  │     builtin:shell:sudo *          (never)            │
  │     *:*delete*                    (never delete)     │
  └──────────────────────────────────────────────────────┘

  In session, when agent wants to run `git commit -m "fix bug"`:

  ┌──────────────────────────────────────┐
  │  🔔 Tool Approval Required          │
  │                                     │
  │  Tool: shell                        │
  │  Command: git commit -m "fix bug"   │
  │                                     │
  │  Rule matched: "git commit *" → ask │
  │                                     │
  │  [Approve]  [Deny]  [Approve All]   │
  └──────────────────────────────────────┘

  In the Inspector, you see every approval decision:
  "tool.approved — approved_by: user" or "tool.approved — approved_by: rule:auto_approve_readonly"

  Full audit trail of what was allowed and why.
```

---

## The README Hook (30 seconds)

Put this at the top of the README:

```markdown
## What can you do with AI Studio?

**See everything your AI agent does.**

> "Refactor auth to use JWT" → Agent reads 3 files, installs a package,
> writes 2 files. You see every step in the Inspector. The agent used
> 2,341 tokens and cost you $0.012. It took 4.8 seconds.
>
> Something wrong? Click any step. See the exact input and output.
> Click [Branch from here]. Try a different approach. Compare results.

**Control what your agent can do.**

> Read files? Auto-approved. Write files? You approve each one.
> `rm -rf`? Blocked. Always. Every decision logged and auditable.

**Compare models side-by-side.**

> Same prompt, Claude vs GPT. Compare output quality, speed, and cost
> in a split-screen view. Make data-driven decisions about which model
> to use.

**Run agents at scale.**

> Batch 50 tasks. Auto-approve safe operations. See which ones failed
> and why. Export results. Total cost: $0.58.
```

---

## Demo Video Script (2 minutes)

```
[0:00] "AI agents are black boxes. You send a prompt, magic happens,
        you get an answer. But what if the answer is wrong?"

[0:10]  Show: AI Studio launches. Clean dark UI. 5 tabs in sidebar.

[0:15]  "AI Studio gives you X-ray vision into every agent decision."

[0:20]  Show: Create an agent. Pick Claude Sonnet. Add system prompt.
        Add GitHub MCP server. 15 seconds.

[0:35]  Show: Start chat. "Find open bugs and summarize them."
        Agent calls github:list_issues. Auto-approved.
        Streaming response appears.

[0:50]  Show: Click "Inspect" button. Inspector opens.
        Timeline shows every event. Color-coded.
        Click a tool call → see input, output, duration.

[1:10]  Show: Stats bar. "That cost $0.008 and used 1,847 tokens."

[1:20]  Show: "What if we try GPT-4o instead?"
        Click [Branch from here]. Change model. Run again.

[1:35]  Show: Compare view. Side by side. Claude vs GPT.
        Different output, different cost. "Claude was better here,
        but GPT was 40% cheaper."

[1:50]  Show: Approval rules. "Read operations: auto-approve.
        Write operations: ask me. Delete: never."

[2:00]  "AI Studio. The IDE for AI agents.
        Open source. Local-first. Free forever.
        Star us on GitHub."
```

---

## Social Media Hooks

### Twitter/X thread opener
> Built an open-source "Chrome DevTools" for AI agents.
>
> See every tool call, every token, every dollar.
> Replay from any point. Branch and compare models.
>
> Claude vs GPT side-by-side with real cost data.
>
> It's free. It runs locally. Your data never leaves your machine.
>
> Thread 🧵👇

### Reddit r/LocalLLaMA hook
> I built an open-source desktop app that lets you inspect exactly what your AI agent does — every tool call, every token count, every cost. Works with Ollama (free), Claude, GPT, Gemini. Think Chrome DevTools but for AI agents. Replay sessions, branch and compare models, batch-run tasks. All local, all free.

### Hacker News hook
> Show HN: AI Studio – Open-source desktop IDE for debugging AI agents
>
> I was frustrated that AI agents are black boxes. You can't see what tools they called, why they chose one approach over another, or how much they cost. So I built AI Studio — a desktop app that records every decision an agent makes and lets you inspect, replay, and compare them.
