// ============================================
// DATABASE — SQLite persistence layer
// Local-first storage for all AI Studio data
// ============================================

use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Thread-safe database handle managed as Tauri state.
#[derive(Clone)]
pub struct Database {
    pub conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// Open (or create) the database at `~/.ai-studio/data.db`
    /// and run all migrations.
    pub fn init() -> Result<Self, String> {
        let db_path = Self::db_path()?;

        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create data directory: {e}"))?;
        }

        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open database: {e}"))?;

        // Enable WAL mode for better concurrent read performance
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
            .map_err(|e| format!("Failed to set pragmas: {e}"))?;

        let db = Self { conn: Arc::new(Mutex::new(conn)) };
        db.migrate()?;
        Ok(db)
    }

    /// Returns `~/.ai-studio/data.db`
    fn db_path() -> Result<PathBuf, String> {
        let home = dirs::home_dir()
            .ok_or_else(|| "Cannot determine home directory".to_string())?;
        Ok(home.join(".ai-studio").join("data.db"))
    }

    /// Run schema migrations. Idempotent — safe to call on every launch.
    fn migrate(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS _meta (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );"
        ).map_err(|e| format!("Migration _meta failed: {e}"))?;

        // Check current schema version
        let version: i64 = conn
            .query_row(
                "SELECT COALESCE((SELECT value FROM _meta WHERE key = 'schema_version'), '0')",
                [],
                |row| row.get::<_, String>(0),
            )
            .map_err(|e| format!("Failed to read schema version: {e}"))?
            .parse()
            .unwrap_or(0);

        if version < 1 {
            self.migrate_v1(&conn)?;
        }
        if version < 2 {
            self.migrate_v2(&conn)?;
        }
        if version < 3 {
            self.migrate_v3(&conn)?;
        }
        if version < 4 {
            self.migrate_v4(&conn)?;
        }

        Ok(())
    }

    /// V1: Core tables — agents, sessions, messages, events, runs, settings, provider_keys
    fn migrate_v1(&self, conn: &Connection) -> Result<(), String> {
        conn.execute_batch(
            "
            -- Agents
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

            -- Sessions
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

            -- Messages
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

            -- Events (Inspector reads from here)
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

            -- Runs
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

            -- Settings (key-value)
            CREATE TABLE IF NOT EXISTS settings (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            -- Provider API Keys
            CREATE TABLE IF NOT EXISTS provider_keys (
                provider   TEXT PRIMARY KEY,
                api_key    TEXT NOT NULL,
                base_url   TEXT,
                updated_at TEXT NOT NULL
            );

            -- Record schema version
            INSERT OR REPLACE INTO _meta (key, value) VALUES ('schema_version', '1');
            "
        ).map_err(|e| format!("Migration v1 failed: {e}"))?;

        println!("[db] Migrated to schema v1");
        Ok(())
    }

    /// V2: MCP servers table
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

    /// V3: Agents schema alignment — tools_mode, mcp_servers, approval_rules columns + global approval_rules table
    fn migrate_v3(&self, conn: &Connection) -> Result<(), String> {
        // ALTER TABLE one-at-a-time; catch "duplicate column" for idempotency
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

    /// V4: Session branching fixes — parent index + ON DELETE SET NULL
    /// SQLite doesn't support ALTER CONSTRAINT, so we recreate the sessions table.
    fn migrate_v4(&self, conn: &Connection) -> Result<(), String> {
        conn.execute_batch(
            "
            -- Add missing parent session index
            CREATE INDEX IF NOT EXISTS idx_sessions_parent ON sessions(parent_session_id);

            -- Nullify parent_session_id when parent is deleted (can't alter FK, use trigger)
            CREATE TRIGGER IF NOT EXISTS trg_sessions_parent_delete
            AFTER DELETE ON sessions
            BEGIN
                UPDATE sessions SET parent_session_id = NULL
                WHERE parent_session_id = OLD.id;
            END;

            INSERT OR REPLACE INTO _meta (key, value) VALUES ('schema_version', '4');
            "
        ).map_err(|e| format!("Migration v4 failed: {e}"))?;

        println!("[db] Migrated to schema v4 (session branching fixes)");
        Ok(())
    }
}

// ============================================
// HELPER — generate ISO 8601 timestamp
// ============================================

pub fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}
