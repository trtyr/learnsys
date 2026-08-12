//! 数据库 schema 定义与初始化。
//!
//! 日期存为 ISO 文本（`YYYY-MM-DD`），时间戳存为 RFC3339 / SQLite datetime 文本。
//! 外键级联：删 topic 删其下卡；删卡片删复习记录；删 goal 删其路径；删路径删其模块序列。
//!
//! - v1: core（topics / cards / review_logs）
//! - v2: LMS 扩展（goals / pathways / modules / pathway_modules / sessions / learner_profile）
//!       + cards.module_id（挂到 Module 下，nullable 兼容现有散卡）

use rusqlite::Connection;

/// 当前 schema 版本（写入 `PRAGMA user_version`）。
pub const SCHEMA_VERSION: i64 = 2;

pub const SCHEMA_SQL: &str = r#"
PRAGMA foreign_keys = ON;

-- ───────────────────────── core (v1) ─────────────────────────

CREATE TABLE IF NOT EXISTS topics (
    id           TEXT PRIMARY KEY,
    name         TEXT NOT NULL,
    stage        TEXT NOT NULL DEFAULT '',
    status       TEXT NOT NULL DEFAULT 'active',
    last_studied TEXT,
    next_plan    TEXT NOT NULL DEFAULT '',
    created      TEXT NOT NULL DEFAULT (date('now'))
);

CREATE TABLE IF NOT EXISTS modules (
    id          TEXT PRIMARY KEY,
    title       TEXT NOT NULL,
    topic       TEXT REFERENCES topics(id) ON DELETE SET NULL,
    description TEXT NOT NULL DEFAULT '',
    status      TEXT NOT NULL DEFAULT 'not_started'
);

CREATE TABLE IF NOT EXISTS cards (
    id        TEXT PRIMARY KEY,
    topic     TEXT NOT NULL REFERENCES topics(id) ON DELETE CASCADE,
    module_id TEXT REFERENCES modules(id) ON DELETE SET NULL,
    front     TEXT NOT NULL,
    back      TEXT NOT NULL,
    ef        REAL NOT NULL DEFAULT 2.5,
    interval  INTEGER NOT NULL DEFAULT 0,
    reps      INTEGER NOT NULL DEFAULT 0,
    due       TEXT NOT NULL,
    created   TEXT NOT NULL,
    updated   TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_cards_due    ON cards(due);
CREATE INDEX IF NOT EXISTS idx_cards_topic  ON cards(topic);

CREATE TABLE IF NOT EXISTS review_logs (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    card_id     TEXT NOT NULL REFERENCES cards(id) ON DELETE CASCADE,
    quality     INTEGER NOT NULL,
    reviewed_at TEXT NOT NULL,
    prev_due    TEXT,
    new_due     TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_logs_card ON review_logs(card_id);
CREATE INDEX IF NOT EXISTS idx_logs_time ON review_logs(reviewed_at);

-- ───────────────────────── lms (v2) ─────────────────────────

CREATE TABLE IF NOT EXISTS goals (
    id               TEXT PRIMARY KEY,
    title            TEXT NOT NULL,
    description      TEXT NOT NULL DEFAULT '',
    success_criteria TEXT NOT NULL DEFAULT '',
    topic            TEXT REFERENCES topics(id) ON DELETE SET NULL,
    status           TEXT NOT NULL DEFAULT 'active',
    created          TEXT NOT NULL DEFAULT (date('now')),
    achieved_at      TEXT
);

CREATE TABLE IF NOT EXISTS pathways (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    methodology TEXT NOT NULL DEFAULT '',
    description TEXT NOT NULL DEFAULT '',
    goal_id     TEXT NOT NULL REFERENCES goals(id) ON DELETE CASCADE,
    is_active   INTEGER NOT NULL DEFAULT 0,
    created     TEXT NOT NULL DEFAULT (date('now'))
);
CREATE INDEX IF NOT EXISTS idx_pathways_goal ON pathways(goal_id);

CREATE TABLE IF NOT EXISTS pathway_modules (
    pathway_id TEXT NOT NULL REFERENCES pathways(id) ON DELETE CASCADE,
    module_id  TEXT NOT NULL REFERENCES modules(id) ON DELETE CASCADE,
    sort_order INTEGER NOT NULL,
    depends_on TEXT NOT NULL DEFAULT '',
    PRIMARY KEY (pathway_id, module_id)
);
CREATE INDEX IF NOT EXISTS idx_pm_order ON pathway_modules(pathway_id, sort_order);

CREATE TABLE IF NOT EXISTS sessions (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    started_at TEXT NOT NULL,
    ended_at   TEXT,
    goal_id    TEXT REFERENCES goals(id) ON DELETE SET NULL,
    pathway_id TEXT REFERENCES pathways(id) ON DELETE SET NULL,
    summary    TEXT NOT NULL DEFAULT '',
    new_cards  INTEGER NOT NULL DEFAULT 0,
    reviewed   INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_sessions_time ON sessions(started_at);

CREATE TABLE IF NOT EXISTS learner_profile (
    id          INTEGER PRIMARY KEY CHECK (id = 1),
    level       TEXT NOT NULL DEFAULT '',
    style       TEXT NOT NULL DEFAULT '',
    weak_points TEXT NOT NULL DEFAULT '[]',
    preferences TEXT NOT NULL DEFAULT '{}',
    notes       TEXT NOT NULL DEFAULT '',
    updated     TEXT NOT NULL DEFAULT (datetime('now'))
);

PRAGMA user_version = 2;
"#;

pub const EXTRA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS resources (
    id        TEXT PRIMARY KEY,
    title     TEXT NOT NULL,
    url       TEXT NOT NULL DEFAULT '',
    notes     TEXT NOT NULL DEFAULT '',
    module_id TEXT REFERENCES modules(id) ON DELETE CASCADE,
    card_id   TEXT REFERENCES cards(id) ON DELETE CASCADE,
    created   TEXT NOT NULL DEFAULT (date('now'))
);
CREATE INDEX IF NOT EXISTS idx_resources_module ON resources(module_id);
CREATE INDEX IF NOT EXISTS idx_resources_card ON resources(card_id);
"#;

/// 在给定连接上初始化 schema（幂等，可重复调用）。
///
/// 新库直接建全表；旧 v1 库通过 [`ensure_column`] 补 cards.module_id。
pub fn init(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(SCHEMA_SQL)?;
    ensure_column(conn, "cards", "module_id", "TEXT")?;
    conn.execute_batch("CREATE INDEX IF NOT EXISTS idx_cards_module ON cards(module_id)")?;
    conn.execute_batch(EXTRA_SQL)?;
    Ok(())
}

/// 若 `table` 缺 `column`，则 ALTER TABLE 补列（兼容旧库渐进迁移）。
fn ensure_column(conn: &Connection, table: &str, column: &str, type_def: &str) -> Result<(), rusqlite::Error> {
    let cols: Vec<String> = {
        let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
        let mapped = stmt.query_map([], |r| r.get::<_, String>(1))?;
        mapped.filter_map(|c| c.ok()).collect()
    };
    if !cols.iter().any(|c| c == column) {
        conn.execute(&format!("ALTER TABLE {table} ADD COLUMN {column} {type_def}"), [])?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_TABLES: [&str; 9] = [
        "topics", "cards", "review_logs", "modules", "goals", "pathways",
        "pathway_modules", "sessions", "learner_profile",
    ];

    fn has_column(conn: &Connection, table: &str, column: &str) -> bool {
        let cols: Vec<String> = {
            let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})")).unwrap();
            stmt.query_map([], |r| r.get::<_, String>(1))
                .unwrap()
                .filter_map(|c| c.ok())
                .collect()
        };
        cols.iter().any(|c| c == column)
    }

    #[test]
    fn init_creates_all_tables_and_version() {
        let conn = Connection::open_in_memory().unwrap();
        init(&conn).unwrap();

        let v: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0)).unwrap();
        assert_eq!(v, SCHEMA_VERSION);

        for t in ALL_TABLES {
            let exists: i64 = conn
                .query_row(
                    &format!("SELECT count(*) FROM sqlite_master WHERE type='table' AND name='{t}'"),
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(exists, 1, "缺表: {t}");
        }
        assert!(has_column(&conn, "cards", "module_id"), "cards 缺 module_id");
    }

    #[test]
    fn init_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        init(&conn).unwrap();
        init(&conn).unwrap();
    }

    #[test]
    fn migrate_old_v1_db_gets_module_id() {
        // 模拟旧 v1 库：建完整的 v1 cards（含 due 等列），但不含 module_id
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE topics (id TEXT PRIMARY KEY, name TEXT, stage TEXT DEFAULT '', status TEXT DEFAULT 'active', last_studied TEXT, next_plan TEXT DEFAULT '', created TEXT DEFAULT (date('now')));
             CREATE TABLE modules (id TEXT PRIMARY KEY, title TEXT);
             CREATE TABLE cards (id TEXT PRIMARY KEY, topic TEXT, front TEXT, back TEXT, ef REAL DEFAULT 2.5, interval INTEGER DEFAULT 0, reps INTEGER DEFAULT 0, due TEXT, created TEXT, updated TEXT DEFAULT (datetime('now')));",
        )
        .unwrap();
        assert!(!has_column(&conn, "cards", "module_id"));
        init(&conn).unwrap();
        assert!(has_column(&conn, "cards", "module_id"), "旧库未补 module_id");
    }
}
