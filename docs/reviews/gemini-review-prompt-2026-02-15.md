# AI Studio — Design Review Request

**Date**: 2026-02-15
**Reviewer**: Gemini 3 Pro
**Previous review**: 2026-02-09 (all high-priority items resolved)

---

## What to Review

AI Studio is a desktop-native IDE for AI agents ("Chrome DevTools for AI agents"). Stack: Tauri 2 (Rust) + React 19 + Python FastAPI sidecar. Phase 1 is complete. Phase 2 is nearly complete. We're preparing for Phase 3 (node editor, replay, open-source launch).

**Please review these 4 areas:**

1. **Session branching implementation** — new feature just shipped. Correctness, edge cases, potential bugs.
2. **Phase 2 completeness** — are there gaps before we call P2 done?
3. **Phase 3 readiness** — is the architecture ready for the node editor and replay features?
4. **Open-source launch readiness** — DX, onboarding, docs, first impressions.

**Format your review as:**
- Checkboxes `- [ ]` for actionable items
- Priority labels: **High**, **Medium**, **Low**
- For each item: what's wrong, why it matters, suggested fix
- If something is already correct, say so explicitly (previous reviewer made false claims)

---

## What's Been Built (Phase 1 + Phase 2 so far)

- SQLite local-first persistence (WAL mode, schema v3 with migrations)
- Agent CRUD (provider, model, system prompt, tools_mode, mcp_servers, approval_rules)
- Session CRUD with real chat (verified with Google Gemini provider)
- **Session branching** (NEW — branch at any message, copy messages, lineage tracking)
- Inspector (event timeline, detail panel, stats, filters, export, keyboard nav)
- MCP tool system (registry, built-in tools, external MCP servers via stdio)
- Event system (WS bridge, live streaming, cost calculation)
- Runs (async background execution, status tracking)
- Error handling (toast notifications, error state in all 19 store actions)
- Onboarding wizard (3-step first-run UX)
- Command palette (Cmd+K)

---

## File 1: `apps/desktop/src-tauri/src/db.rs` (Database + Migrations)

```rust
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Database {
    pub conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn init() -> Result<Self, String> {
        let db_path = Self::db_path()?;
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create data directory: {e}"))?;
        }
        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open database: {e}"))?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
            .map_err(|e| format!("Failed to set pragmas: {e}"))?;
        let db = Self { conn: Arc::new(Mutex::new(conn)) };
        db.migrate()?;
        Ok(db)
    }

    fn db_path() -> Result<PathBuf, String> {
        let home = dirs::home_dir()
            .ok_or_else(|| "Cannot determine home directory".to_string())?;
        Ok(home.join(".ai-studio").join("data.db"))
    }

    fn migrate(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS _meta (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );"
        ).map_err(|e| format!("Migration _meta failed: {e}"))?;

        let version: i64 = conn
            .query_row(
                "SELECT COALESCE((SELECT value FROM _meta WHERE key = 'schema_version'), '0')",
                [],
                |row| row.get::<_, String>(0),
            )
            .map_err(|e| format!("Failed to read schema version: {e}"))?
            .parse()
            .unwrap_or(0);

        if version < 1 { self.migrate_v1(&conn)?; }
        if version < 2 { self.migrate_v2(&conn)?; }
        if version < 3 { self.migrate_v3(&conn)?; }
        Ok(())
    }

    fn migrate_v1(&self, conn: &Connection) -> Result<(), String> {
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS agents (
                id             TEXT PRIMARY KEY,
                name           TEXT NOT NULL,
                description    TEXT NOT NULL DEFAULT '',
                provider       TEXT NOT NULL,
                model          TEXT NOT NULL,
                system_prompt  TEXT NOT NULL DEFAULT '',
                temperature    REAL NOT NULL DEFAULT 0.7,
                max_tokens     INTEGER NOT NULL DEFAULT 4096,
                tools          TEXT NOT NULL DEFAULT '[]',
                created_at     TEXT NOT NULL,
                updated_at     TEXT NOT NULL,
                is_archived    INTEGER NOT NULL DEFAULT 0
            );
            CREATE INDEX IF NOT EXISTS idx_agents_archived ON agents(is_archived);

            CREATE TABLE IF NOT EXISTS sessions (
                id                  TEXT PRIMARY KEY,
                agent_id            TEXT NOT NULL REFERENCES agents(id),
                title               TEXT NOT NULL DEFAULT '',
                parent_session_id   TEXT REFERENCES sessions(id),
                branch_from_seq     INTEGER,
                status              TEXT NOT NULL DEFAULT 'active',
                message_count       INTEGER NOT NULL DEFAULT 0,
                event_count         INTEGER NOT NULL DEFAULT 0,
                total_input_tokens  INTEGER NOT NULL DEFAULT 0,
                total_output_tokens INTEGER NOT NULL DEFAULT 0,
                total_cost_usd      REAL NOT NULL DEFAULT 0.0,
                created_at          TEXT NOT NULL,
                updated_at          TEXT NOT NULL,
                ended_at            TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_sessions_agent ON sessions(agent_id);
            CREATE INDEX IF NOT EXISTS idx_sessions_status ON sessions(status);
            CREATE INDEX IF NOT EXISTS idx_sessions_updated ON sessions(updated_at DESC);

            CREATE TABLE IF NOT EXISTS messages (
                id            TEXT PRIMARY KEY,
                session_id    TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
                seq           INTEGER NOT NULL,
                role          TEXT NOT NULL,
                content       TEXT NOT NULL,
                model         TEXT,
                provider      TEXT,
                input_tokens  INTEGER,
                output_tokens INTEGER,
                cost_usd      REAL,
                duration_ms   INTEGER,
                tool_calls    TEXT,
                created_at    TEXT NOT NULL,
                UNIQUE(session_id, seq)
            );
            CREATE INDEX IF NOT EXISTS idx_messages_session_seq ON messages(session_id, seq);

            CREATE TABLE IF NOT EXISTS events (
                event_id   TEXT PRIMARY KEY,
                type       TEXT NOT NULL,
                ts         TEXT NOT NULL,
                session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
                source     TEXT NOT NULL,
                seq        INTEGER NOT NULL,
                payload    TEXT NOT NULL DEFAULT '{}',
                cost_usd   REAL,
                UNIQUE(session_id, seq)
            );
            CREATE INDEX IF NOT EXISTS idx_events_session_type ON events(session_id, type);
            CREATE INDEX IF NOT EXISTS idx_events_session_seq ON events(session_id, seq);

            CREATE TABLE IF NOT EXISTS runs (
                id                 TEXT PRIMARY KEY,
                agent_id           TEXT NOT NULL REFERENCES agents(id),
                session_id         TEXT REFERENCES sessions(id),
                name               TEXT NOT NULL DEFAULT '',
                input              TEXT NOT NULL,
                status             TEXT NOT NULL DEFAULT 'pending',
                output             TEXT,
                error              TEXT,
                total_events       INTEGER NOT NULL DEFAULT 0,
                total_tokens       INTEGER NOT NULL DEFAULT 0,
                total_cost_usd     REAL NOT NULL DEFAULT 0.0,
                duration_ms        INTEGER,
                created_at         TEXT NOT NULL,
                started_at         TEXT,
                completed_at       TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_runs_agent ON runs(agent_id);
            CREATE INDEX IF NOT EXISTS idx_runs_status ON runs(status);

            CREATE TABLE IF NOT EXISTS settings (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS provider_keys (
                provider   TEXT PRIMARY KEY,
                api_key    TEXT NOT NULL,
                base_url   TEXT,
                updated_at TEXT NOT NULL
            );

            INSERT OR REPLACE INTO _meta (key, value) VALUES ('schema_version', '1');
            "
        ).map_err(|e| format!("Migration v1 failed: {e}"))?;
        println!("[db] Migrated to schema v1");
        Ok(())
    }

    fn migrate_v2(&self, conn: &Connection) -> Result<(), String> {
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS mcp_servers (
                id         TEXT PRIMARY KEY,
                name       TEXT NOT NULL UNIQUE,
                transport  TEXT NOT NULL DEFAULT 'stdio',
                command    TEXT,
                args       TEXT NOT NULL DEFAULT '[]',
                url        TEXT,
                env        TEXT NOT NULL DEFAULT '{}',
                enabled    INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            INSERT OR REPLACE INTO _meta (key, value) VALUES ('schema_version', '2');
            "
        ).map_err(|e| format!("Migration v2 failed: {e}"))?;
        println!("[db] Migrated to schema v2 (mcp_servers)");
        Ok(())
    }

    fn migrate_v3(&self, conn: &Connection) -> Result<(), String> {
        let alter_stmts = [
            "ALTER TABLE agents ADD COLUMN tools_mode TEXT NOT NULL DEFAULT 'restricted'",
            "ALTER TABLE agents ADD COLUMN mcp_servers TEXT NOT NULL DEFAULT '[]'",
            "ALTER TABLE agents ADD COLUMN approval_rules TEXT NOT NULL DEFAULT '[]'",
        ];
        for stmt in &alter_stmts {
            match conn.execute(stmt, []) {
                Ok(_) => {}
                Err(e) if e.to_string().contains("duplicate column") => {}
                Err(e) => return Err(format!("Migration v3 ALTER failed: {e}")),
            }
        }
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS approval_rules (
                id           TEXT PRIMARY KEY,
                name         TEXT NOT NULL,
                tool_pattern TEXT NOT NULL,
                action       TEXT NOT NULL,
                priority     INTEGER DEFAULT 0,
                enabled      INTEGER DEFAULT 1,
                created_at   TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_approval_rules_enabled
                ON approval_rules(enabled, priority DESC);
            INSERT OR REPLACE INTO _meta (key, value) VALUES ('schema_version', '3');
            "
        ).map_err(|e| format!("Migration v3 failed: {e}"))?;
        println!("[db] Migrated to schema v3 (agents schema alignment + approval_rules)");
        Ok(())
    }
}

pub fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}
```

---

## File 2: `branch_session` command (from `commands.rs`)

```rust
#[tauri::command]
pub fn branch_session(
    db: tauri::State<'_, Database>,
    session_id: String,
    seq: i64,
) -> Result<Session, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    // 1. Look up parent session
    let (agent_id, parent_title): (String, String) = conn
        .query_row(
            "SELECT agent_id, title FROM sessions WHERE id = ?1",
            params![session_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| "Parent session not found".to_string())?;

    // 2. Look up agent
    let (agent_name, agent_model): (String, String) = conn
        .query_row(
            "SELECT name, model FROM agents WHERE id = ?1",
            params![agent_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| "Agent not found".to_string())?;

    // 3. Create new session
    let new_id = Uuid::new_v4().to_string();
    let now = now_iso();
    let branch_title = format!("Branch of {parent_title}");

    conn.execute(
        "INSERT INTO sessions (id, agent_id, title, status, parent_session_id, branch_from_seq, created_at, updated_at)
         VALUES (?1, ?2, ?3, 'active', ?4, ?5, ?6, ?7)",
        params![new_id, agent_id, branch_title, session_id, seq, now, now],
    )
    .map_err(|e| format!("Failed to create branch session: {e}"))?;

    // 4. Copy messages where seq <= branch point
    let mut stmt = conn
        .prepare(
            "SELECT seq, role, content, model, provider, input_tokens, output_tokens,
                    cost_usd, duration_ms, created_at
             FROM messages WHERE session_id = ?1 AND seq <= ?2
             ORDER BY seq ASC",
        )
        .map_err(|e| e.to_string())?;

    let rows: Vec<(i64, String, String, Option<String>, Option<String>,
                    Option<i64>, Option<i64>, Option<f64>, Option<i64>, String)> = stmt
        .query_map(params![session_id, seq], |row| {
            Ok((
                row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?,
                row.get(5)?, row.get(6)?, row.get(7)?, row.get(8)?, row.get(9)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let msg_count = rows.len() as i64;
    for (m_seq, role, content, model, provider, in_tok, out_tok, cost, dur, created) in &rows {
        let msg_id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO messages (id, session_id, seq, role, content, model, provider,
                                   input_tokens, output_tokens, cost_usd, duration_ms, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![msg_id, new_id, m_seq, role, content, model, provider,
                    in_tok, out_tok, cost, dur, created],
        )
        .map_err(|e| format!("Failed to copy message: {e}"))?;
    }

    // 5. Update message_count on new session
    conn.execute(
        "UPDATE sessions SET message_count = ?1 WHERE id = ?2",
        params![msg_count, new_id],
    )
    .map_err(|e| format!("Failed to update message count: {e}"))?;

    Ok(Session {
        id: new_id,
        agent_id,
        title: branch_title,
        status: "active".to_string(),
        message_count: msg_count,
        event_count: 0,
        total_input_tokens: 0,
        total_output_tokens: 0,
        total_cost_usd: 0.0,
        created_at: now.clone(),
        updated_at: now,
        ended_at: None,
        agent_name: Some(agent_name),
        agent_model: Some(agent_model),
        parent_session_id: Some(session_id),
        branch_from_seq: Some(seq),
    })
}
```

---

## File 3: Session struct (from `commands.rs`)

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: String,
    pub agent_id: String,
    pub title: String,
    pub status: String,
    pub message_count: i64,
    pub event_count: i64,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_cost_usd: f64,
    pub created_at: String,
    pub updated_at: String,
    pub ended_at: Option<String>,
    pub agent_name: Option<String>,
    pub agent_model: Option<String>,
    pub parent_session_id: Option<String>,
    pub branch_from_seq: Option<i64>,
}
```

---

## File 4: TypeScript Session type (`packages/shared/types/agent.ts`)

```typescript
export interface Session {
    id: string;
    agentId: string;
    title: string;
    status: string;
    messageCount: number;
    eventCount: number;
    totalInputTokens: number;
    totalOutputTokens: number;
    totalCostUsd: number;
    createdAt: string;
    updatedAt: string;
    endedAt: string | null;
    agentName: string | null;
    agentModel: string | null;
    parentSessionId: string | null;
    branchFromSeq: number | null;
}
```

---

## File 5: Zustand store — `branchSession` action (`apps/ui/src/state/store.ts`)

```typescript
branchSession: async (sessionId, seq) => {
    try {
        const session = await invoke<Session>('branch_session', { sessionId, seq });
        set((s) => ({ sessions: [session, ...s.sessions] }));
        get().addToast('Branch created', 'success');
        return session;
    } catch (e) {
        const msg = `Failed to branch session: ${e}`;
        set({ error: msg });
        get().addToast(msg, 'error');
        throw e;
    }
},
```

---

## File 6: SessionsPage UI — branch button + lineage badge (`apps/ui/src/app/pages/SessionsPage.tsx`)

```tsx
import { Plus, MessageSquare, Send, Loader2, Trash2, Search, GitBranch } from 'lucide-react';
import { useState, useEffect, useRef } from 'react';
import { useAppStore } from '../../state/store';

export function SessionsPage() {
    const {
        sessions, sessionsLoading, fetchSessions, createSession, deleteSession, branchSession,
        agents, fetchAgents,
        messages, messagesLoading, fetchMessages, sendMessage, sending,
        error, openInspector,
    } = useAppStore();
    const [selectedSessionId, setSelectedSessionId] = useState<string | undefined>();
    const [input, setInput] = useState('');
    const [showNewSession, setShowNewSession] = useState(false);
    const [selectedAgentId, setSelectedAgentId] = useState('');
    const chatEndRef = useRef<HTMLDivElement>(null);

    // ... effects omitted for brevity (fetch on mount, auto-scroll, etc.)

    const handleBranch = async (msgSeq: number) => {
        if (!selectedSessionId) return;
        try {
            const branch = await branchSession(selectedSessionId, msgSeq);
            setSelectedSessionId(branch.id);
        } catch { /* error handled by store */ }
    };

    return (
        <div className="animate-fade-in h-full flex flex-col">
            {/* ... header, new session dialog ... */}

            <div className="flex-1 flex gap-4 mt-4 overflow-hidden">
                {/* Session List */}
                <div className="w-80 panel flex flex-col">
                    {/* ... panel header ... */}
                    <div className="flex-1 overflow-y-auto p-2 space-y-2">
                        {sessions.map((session) => (
                            <div key={session.id} className={`p-3 rounded-lg cursor-pointer transition-all group ${...}`}
                                onClick={() => setSelectedSessionId(session.id)}>
                                <div className="flex items-center justify-between">
                                    <div className="font-medium text-sm truncate flex-1 flex items-center gap-1">
                                        {session.parentSessionId && (
                                            <span title="Branched session">
                                                <GitBranch className="w-3 h-3 text-[var(--text-muted)] flex-shrink-0" />
                                            </span>
                                        )}
                                        {session.title}
                                    </div>
                                    {/* ... hover actions (inspect, delete) ... */}
                                </div>
                                <div className="text-xs text-[var(--text-muted)] mt-1">
                                    {session.agentName} · {session.messageCount} messages
                                </div>
                            </div>
                        ))}
                    </div>
                </div>

                {/* Chat Area */}
                <div className="flex-1 panel flex flex-col">
                    {/* ... chat header ... */}
                    <div className="flex-1 overflow-y-auto p-4 space-y-4">
                        {messages.map((msg) => (
                            <div key={msg.id} className={`group/msg flex ${msg.role === 'user' ? 'justify-end' : 'justify-start'}`}>
                                <div className={`max-w-[80%] p-3 rounded-lg relative ${
                                    msg.role === 'user'
                                        ? 'bg-[var(--accent-primary)] text-white'
                                        : 'bg-[var(--bg-tertiary)]'
                                }`}>
                                    <div className="text-sm whitespace-pre-wrap">{msg.content}</div>
                                    <div className={`text-xs mt-2 flex items-center gap-2 ${
                                        msg.role === 'user' ? 'text-white/70' : 'text-[var(--text-muted)]'
                                    }`}>
                                        <span>{new Date(msg.createdAt).toLocaleTimeString()}</span>
                                        {msg.model && <span>· {msg.model}</span>}
                                        {msg.durationMs != null && <span>· {(msg.durationMs / 1000).toFixed(1)}s</span>}
                                        <button
                                            className="opacity-0 group-hover/msg:opacity-100 ml-auto p-0.5 rounded hover:bg-white/10 transition-all"
                                            onClick={() => handleBranch(msg.seq)}
                                            title="Branch from here"
                                        >
                                            <GitBranch className="w-3 h-3" />
                                        </button>
                                    </div>
                                </div>
                            </div>
                        ))}
                    </div>
                    {/* ... input bar ... */}
                </div>
            </div>
        </div>
    );
}
```

---

## File 7: `data-model.md` spec — Session Branching section

```
When a user wants to "replay from here" or "try a different approach", we create a branch:

1. User is in session S1 at event seq=15
2. User clicks "Branch from here"
3. System creates new session S2 with:
   - parent_session_id = S1
   - branch_from_seq = 15
4. System copies messages from S1 where seq <= 15 into S2
5. S2 continues independently from that point
```

---

## File 8: `STATUS.md` — Current Sprint Board

Phase 2 task status:

| Task | Status |
|------|--------|
| Runs execution | DONE |
| DB wipe command | DONE |
| Error handling polish | DONE |
| Agents schema alignment | DONE |
| Sidecar error events | DONE |
| Onboarding / first-run UX | DONE |
| Session branching | DONE |
| Inspector improvements | Backlog |

Phase 3 planned:
- Node editor architecture ("Unreal Blueprints for AI agents")
- Replay from Inspector
- Plugin system
- Community templates
- Installers

---

## File 9: Phase Plan summary (from `docs/specs/phase-plan.md`)

Phase 2B (Replay & Branch) remaining tasks:
- 2B.2: Replay from point (re-execute with same/modified context) — NOT DONE
- 2B.3: Compare view (side-by-side sessions) — NOT DONE

Phase 2A (Full Inspector) remaining:
- 2A.8: Virtualized timeline (react-window) — NOT DONE

Phase 2D (Auto-Approval Rules):
- 2D.3: Rule evaluation engine in Tauri — NOT DONE
- 2D.4: "Auto-approved by rule: X" display in Inspector — NOT DONE

---

## Specific Questions for the Reviewer

1. **Branching edge cases**: What happens when branching a branch (branch of a branch)? The current code doesn't prevent it, which should be fine, but is the title "Branch of Branch of Chat with X" a UX problem?

2. **Transaction safety**: The `branch_session` command does multiple SQL operations (INSERT session, SELECT messages, INSERT messages loop, UPDATE count) without an explicit transaction. Is this a data integrity risk?

3. **Token/cost copying**: When branching, we copy messages but set `total_input_tokens`, `total_output_tokens`, `total_cost_usd` to 0 on the new session. Should we sum the copied messages' tokens/costs instead?

4. **Sidecar conversation history**: The sidecar holds in-memory conversation history keyed by `conversation_id = session_id`. When we branch, the sidecar doesn't know about the branch. The first message to the branched session will start fresh from the sidecar's perspective, but the UI shows copied messages. Is this a UX consistency problem?

5. **Missing `idx_sessions_parent` index**: The `data-model.md` spec defines `CREATE INDEX idx_sessions_parent ON sessions(parent_session_id)` but `db.rs` migrate_v1 doesn't create it. Should we add a v4 migration?

6. **Delete cascade**: If a parent session is deleted, what happens to its branches? Currently `sessions.parent_session_id` has no ON DELETE behavior defined. Should it be SET NULL or CASCADE?

7. **Phase 3 node editor**: The current architecture is event-sourced with a linear timeline. How should we evolve this for a visual node graph where execution flows through connected nodes? Any architectural concerns?
