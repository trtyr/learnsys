//! 数据库 schema 定义与初始化。
//!
//! 日期存为 ISO 文本（`YYYY-MM-DD`），时间戳存为 RFC3339 文本。
//! 外键级联：删 topic 连带删其下卡片；删卡片连带删其复习记录。

use rusqlite::Connection;

/// 当前 schema 版本（写入 `PRAGMA user_version`）。
pub const SCHEMA_VERSION: i64 = 1;

pub const SCHEMA_SQL: &str = r#"
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS topics (
    id           TEXT PRIMARY KEY,
    name         TEXT NOT NULL,
    stage        TEXT NOT NULL DEFAULT '',
    status       TEXT NOT NULL DEFAULT 'active',
    last_studied TEXT,
    next_plan    TEXT NOT NULL DEFAULT '',
    created      TEXT NOT NULL DEFAULT (date('now'))
);

CREATE TABLE IF NOT EXISTS cards (
    id       TEXT PRIMARY KEY,
    topic    TEXT NOT NULL REFERENCES topics(id) ON DELETE CASCADE,
    front    TEXT NOT NULL,
    back     TEXT NOT NULL,
    ef       REAL NOT NULL DEFAULT 2.5,
    interval INTEGER NOT NULL DEFAULT 0,
    reps     INTEGER NOT NULL DEFAULT 0,
    due      TEXT NOT NULL,
    created  TEXT NOT NULL,
    updated  TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_cards_due   ON cards(due);
CREATE INDEX IF NOT EXISTS idx_cards_topic ON cards(topic);

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

PRAGMA user_version = 1;
"#;

/// 在给定连接上初始化 schema（幂等，可重复调用）。
pub fn init(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(SCHEMA_SQL)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_creates_all_tables_and_version() {
        let conn = Connection::open_in_memory().unwrap();
        init(&conn).unwrap();

        let v: i64 = conn
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(v, SCHEMA_VERSION);

        for t in ["topics", "cards", "review_logs"] {
            let exists: i64 = conn
                .query_row(
                    &format!(
                        "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='{t}'"
                    ),
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(exists, 1, "缺表: {t}");
        }
    }

    #[test]
    fn init_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        init(&conn).unwrap();
        init(&conn).unwrap(); // 第二次不应报错（IF NOT EXISTS）
    }
}
