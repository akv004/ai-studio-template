# AI Studio — Agent Inspector Specification

> **Version**: 2.0
> **Status**: Draft
> **Depends on**: product-vision.md, architecture.md, event-system.md, data-model.md
> **This is the flagship feature.**

---

## What Is the Inspector?

The Agent Inspector is **Chrome DevTools for AI agents**. It gives developers complete visibility into what an agent did, why it did it, how much it cost, and the ability to replay or branch from any point.

No other tool in the AI ecosystem offers this level of inspection depth with a GUI. This is the feature people will screenshot, share, and choose AI Studio for.

---

## Layout

```
┌──────────────────────────────────────────────────────────────────┐
│  Inspector                                          [Session ▾]  │
├────────────┬─────────────────────────────────────────────────────┤
│            │                                                     │
│  Timeline  │  Detail Panel                                       │
│  (left)    │  (right)                                            │
│            │                                                     │
│  ┌──────┐  │  Shows details for whatever is selected in the      │
│  │ msg  │◄─│  timeline. Adapts its content based on event type.  │
│  │ seq=1│  │                                                     │
│  ├──────┤  │  ┌─────────────────────────────────────────────┐    │
│  │ llm  │  │  │  When a tool event is selected:             │    │
│  │ seq=2│  │  │                                             │    │
│  ├──────┤  │  │  Tool: shell                                │    │
│  │ tool │  │  │  Input: git status                          │    │
│  │ seq=3│◄─│──│  Output: (collapsible text block)           │    │
│  ├──────┤  │  │  Duration: 120ms                            │    │
│  │ tool │  │  │  Approval: auto (rule: git*)                │    │
│  │ seq=4│  │  │  Exit code: 0                               │    │
│  ├──────┤  │  │                                             │    │
│  │ llm  │  │  │  [Replay from here] [Branch] [Copy output] │    │
│  │ seq=5│  │  └─────────────────────────────────────────────┘    │
│  ├──────┤  │                                                     │
│  │ msg  │  │                                                     │
│  │ seq=6│  │                                                     │
│  └──────┘  │                                                     │
│            │                                                     │
├────────────┴─────────────────────────────────────────────────────┤
│  Stats Bar                                                       │
│  Tokens: 3,847 in / 412 out │ Cost: $0.014 │ Duration: 4.2s    │
│  Tool calls: 2 (2 approved, 0 denied) │ Model: claude-sonnet    │
└──────────────────────────────────────────────────────────────────┘
```

### Three Zones

1. **Timeline** (left panel, ~250px): Vertical list of events, color-coded by type. Scrollable. Clickable.
2. **Detail Panel** (right panel, fills remaining space): Shows full details for the selected event.
3. **Stats Bar** (bottom, fixed): Aggregated session metrics — always visible.

---

## Timeline Panel

### Event Cards

Each event in the timeline is rendered as a compact card:

```
┌─────────────────────────┐
│ 🟦 seq=3  tool.requested │
│ shell: git status        │
│ 14:30:02  120ms          │
└─────────────────────────┘
```

### Color Coding

| Event Category | Color | Hex |
|---|---|---|
| Messages (user) | Blue | `#3B82F6` |
| Messages (assistant) | Green | `#22C55E` |
| LLM inference | Purple | `#A855F7` |
| Tool requested | Yellow | `#EAB308` |
| Tool completed | Green | `#22C55E` |
| Tool error / denied | Red | `#EF4444` |
| Session lifecycle | Gray | `#6B7280` |
| MCP events | Cyan | `#06B6D4` |

### Grouping

Sequential related events are visually grouped:

```
┌─ Tool Call Group ────────────┐
│  tool.requested  (seq=3)     │
│  tool.approved   (seq=4)     │
│  tool.started    (seq=5)     │
│  tool.completed  (seq=6)     │
└──────────────────────────────┘
```

Grouping rules:
- Events sharing the same `tool_call_id` are grouped
- `llm.request.started` + `llm.response.chunk*` + `llm.response.completed` are grouped
- Groups are collapsible (click to expand/collapse)

### Filtering

Top of timeline has filter chips:

```
[All] [Messages] [LLM] [Tools] [Errors]
```

Click to filter the timeline. Multiple can be active.

### Search

`Cmd+F` in Inspector opens search. Searches across:
- Message content
- Tool inputs/outputs
- Error messages

Matching events are highlighted in the timeline.

---

## Detail Panel

The detail panel adapts based on the selected event type.

### For `message.user`

```
┌─────────────────────────────────────────┐
│  User Message                  seq=1    │
│  ──────────────────────────────────     │
│                                         │
│  Tell me about the current git status   │
│  and list any modified files.           │
│                                         │
│  ───────────────────────────────────    │
│  14:30:00.123                           │
│  [Copy] [Branch from here]             │
└─────────────────────────────────────────┘
```

### For `message.assistant`

```
┌─────────────────────────────────────────┐
│  Assistant Response            seq=6    │
│  ──────────────────────────────────     │
│                                         │
│  Here's the current git status:         │
│  (rendered markdown)                    │
│                                         │
│  ───────────────────────────────────    │
│  Model: claude-sonnet-4-5-20250929      │
│  Provider: anthropic                    │
│  Tokens: 1,247 in / 89 out             │
│  Cost: $0.005                           │
│  Duration: 1,832ms                      │
│  TTFT: 340ms                            │
│  Stop reason: end_turn                  │
│                                         │
│  [Copy] [Branch from here] [Re-run]    │
└─────────────────────────────────────────┘
```

### For Tool Events (Grouped)

```
┌─────────────────────────────────────────┐
│  Tool Call                     seq=3-6  │
│  ──────────────────────────────────     │
│                                         │
│  Tool: shell                            │
│  MCP Server: (built-in)                 │
│                                         │
│  ▸ Input                                │
│  ┌─────────────────────────────────┐    │
│  │ command: git status             │    │
│  │ timeout: 30                     │    │
│  └─────────────────────────────────┘    │
│                                         │
│  ▸ Output                               │
│  ┌─────────────────────────────────┐    │
│  │ On branch main                  │    │
│  │ Changes not staged for commit:  │    │
│  │   modified: src/App.tsx         │    │
│  └─────────────────────────────────┘    │
│                                         │
│  ───────────────────────────────────    │
│  Approval: auto_approve (rule: git*)    │
│  Duration: 120ms                        │
│  Exit code: 0                           │
│                                         │
│  [Copy input] [Copy output] [Re-run]   │
│  [Branch from here]                     │
└─────────────────────────────────────────┘
```

### For `llm.response.error`

```
┌─────────────────────────────────────────┐
│  ⚠ LLM Error                  seq=5    │
│  ──────────────────────────────────     │
│                                         │
│  Error: Rate limit exceeded             │
│  Code: 429                              │
│  Provider: anthropic                    │
│  Model: claude-sonnet-4-5-20250929      │
│                                         │
│  ───────────────────────────────────    │
│  14:30:02.456                           │
│  [Retry from here] [Branch]            │
└─────────────────────────────────────────┘
```

---

## Stats Bar

Always visible at the bottom. Updates in real-time during active sessions.

```
┌──────────────────────────────────────────────────────────────────┐
│ ◆ 3,847 input tokens  ◆ 412 output tokens  ◆ $0.014 cost       │
│ ◆ 4.2s total  ◆ 2 tool calls (2✓ 0✗)  ◆ claude-sonnet-4-5     │
└──────────────────────────────────────────────────────────────────┘
```

**Fields:**
| Metric | Source |
|---|---|
| Input tokens | Sum of `input_tokens` from all `llm.response.completed` events |
| Output tokens | Sum of `output_tokens` from all `llm.response.completed` events |
| Cost | Sum of `cost_usd` from all `llm.response.completed` events |
| Total duration | `ended_at - created_at` from the session |
| Tool calls | Count of `tool.requested` events |
| Tool results | Count of `tool.approved` vs `tool.denied` |
| Model | From the most recent `llm.response.completed` event |

---

## Key Features

### 1. Live Inspection

When viewing an **active session**, the Inspector updates in real-time:
- New events append to the timeline
- Stats bar counters increment
- Streaming tokens show character-by-character in the detail panel
- Timeline auto-scrolls to newest event (unless user has scrolled up)

**Implementation**: The Inspector listens to `agent_event` Tauri events and appends them to its local state.

### 2. Replay

**What it does**: Re-execute a session from a specific point with the same (or modified) context.

**Flow:**
1. User selects an event in the timeline
2. Clicks "Replay from here"
3. System creates a new session (branch) with:
   - Same agent config
   - Messages up to the selected point
   - User can optionally edit the last message before replaying
4. Agent runs from that point, generating new events
5. Inspector shows the new session

**Use case**: "The agent made a bad tool call at step 5. Let me replay from step 4 with a modified prompt."

### 3. Branch & Compare

**What it does**: Fork from any point and run an alternative path. Then compare the two side-by-side.

**Compare view:**
```
┌─────────────────────┬─────────────────────┐
│  Original (S1)      │  Branch (S2)        │
│                     │                     │
│  seq=1: user msg    │  seq=1: user msg    │  (same)
│  seq=2: llm resp    │  seq=2: llm resp    │  (same)
│  seq=3: tool call   │  seq=3: tool call   │  (different!)
│  ...                │  ...                │
│                     │                     │
│  Tokens: 4,259      │  Tokens: 3,102      │
│  Cost: $0.018       │  Cost: $0.012       │
│  Tools: 3           │  Tools: 2           │
└─────────────────────┴─────────────────────┘
```

**Implementation**: Uses the branching model from data-model.md (`parent_session_id` + `branch_from_seq`).

### 4. Export

**Formats:**
- **JSON**: Full event log with all metadata. Machine-readable. Can be re-imported.
- **Markdown**: Human-readable conversation transcript with tool call summaries.

**JSON export structure:**
```json
{
  "export_version": 1,
  "exported_at": "2026-02-08T15:00:00Z",
  "session": { "id": "...", "agent_id": "...", "created_at": "..." },
  "agent": { "name": "...", "model": "...", "system_prompt": "..." },
  "events": [ ... ],
  "stats": { "total_tokens": 4259, "total_cost": 0.018, "total_duration_ms": 4200 }
}
```

**Markdown export structure:**
```markdown
# Session: Fix login bug
Agent: Code Assistant (claude-sonnet-4-5)
Date: 2026-02-08 14:30
Tokens: 4,259 | Cost: $0.018 | Duration: 4.2s

---

**User**: Tell me about the current git status

**Assistant**: Here's the current git status: ...

> Tool: shell (`git status`)
> Output: On branch main...
> Duration: 120ms | Approved: auto (git*)

**Assistant**: Based on the git status...
```

### 5. Keyboard Navigation

| Shortcut | Action |
|---|---|
| `↑` / `↓` | Navigate timeline events |
| `Enter` | Select event (show in detail panel) |
| `Cmd+F` | Search events |
| `Cmd+E` | Export session |
| `G` then `G` | Jump to first event |
| `Shift+G` | Jump to last event |
| `[` / `]` | Collapse/expand event groups |
| `F` | Toggle filter panel |

---

## Accessing the Inspector

### From Sessions
Every session has an "Inspect" button that opens the Inspector for that session.

```
Sessions Page
┌──────────────────────────────┐
│ Fix login bug      14:30     │
│ claude-sonnet  12 messages   │
│         [Open] [Inspect] 🔍 │
└──────────────────────────────┘
```

### From Runs
Every run links to its session's Inspector view.

### Direct Navigation
Sidebar: `Inspector` module. Shows a session picker, then the Inspector view.

### Keyboard
`Cmd+I` from any session → opens Inspector for current session.

---

## Phase 1 vs Phase 2 Scope

### Phase 1 (Core — Must Ship)
- Timeline with event cards (color-coded, grouped)
- Detail panel for all event types
- Stats bar with token/cost/duration
- Filter by event category
- Session selector
- Live inspection (real-time event streaming)
- Export as JSON

### Phase 2 (Power — Differentiator)
- Replay from any point
- Branch & Compare (side-by-side)
- Search across events
- Export as Markdown
- Keyboard navigation (vim-style)
- Performance waterfall view (timing visualization)
- Cost breakdown chart (pie chart by model/provider)

---

## Technical Implementation Notes

### State Management
The Inspector page maintains its own local state (not in the global Zustand store) because:
- Event lists can be very large (thousands of events)
- Multiple Inspector views could be open (in future, tabs)
- Local state avoids polluting the global store

```typescript
// InspectorPage local state
interface InspectorState {
  sessionId: string;
  events: Event[];
  selectedSeq: number | null;
  filters: Set<string>;       // Active filter categories
  isLive: boolean;            // Whether auto-following new events
  searchQuery: string;
}
```

### Performance Considerations
- **Virtualized list**: Timeline uses virtual scrolling (only renders visible events). Libraries: `react-window` or `@tanstack/virtual`.
- **Lazy payload loading**: For large tool outputs, store a truncated preview. Load full payload on demand when selected.
- **Pagination**: For sessions with 1000+ events, load in chunks of 200. Load more as user scrolls.
- **Debounced live updates**: During streaming, batch UI updates every 50ms to avoid excessive re-renders.

### Event Data Size
A typical event is 200-500 bytes of JSON. A session with 200 events is ~100KB. Even 5000 events is only ~2.5MB. This fits comfortably in memory.

Tool outputs (e.g., large file contents) are the outlier. For outputs > 10KB, store a truncated version in the event payload and the full output in the `artifacts/` directory, referenced by path.

---

## What Makes This Better Than Alternatives

| Feature | LangSmith | LangGraph Studio | AI Studio Inspector |
|---|---|---|---|
| Local-first (no cloud) | No (cloud only) | No (cloud trace) | **Yes** |
| Real-time streaming | Yes | Yes | **Yes** |
| Replay from point | No | No | **Yes** |
| Branch & compare | No | No | **Yes** |
| Cost tracking | Yes | Limited | **Yes** |
| Tool call deep-dive | Basic | Basic | **Full** (input, output, approval, timing) |
| Export | Limited | No | **JSON + Markdown** |
| Free / open source | No (paid) | No (paid) | **Yes** |
| Works offline | No | No | **Yes** |

The combination of **local-first + replay + branching + full tool traces + free** is unique.
