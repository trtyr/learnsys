//! learnsys-migrate —— 把现有 markdown 卡片导入 SQLite。
//!
//! 用法:
//!   learnsys-migrate [源数据目录]          # 默认 ~/.pi/learning-data
//!
//! 源目录结构（对齐 Python sm2.py）:
//!   <src>/cards/<topic>/<id>.md          # frontmatter + 正面 \n---\n 背面
//!   <src>/progress.md                    # 主题阶段表（best-effort 解析）
//!
//! 幂等：重跑用 INSERT OR REPLACE / INSERT OR IGNORE，安全覆盖。

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{NaiveDate, Utc};
use rusqlite::{params, Connection};

use learnsys_core::db;

fn main() {
    let src = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(default_src);

    if !src.exists() {
        eprintln!("❌ 源目录不存在: {}", src.display());
        std::process::exit(1);
    }

    let conn = match db::connect() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ 打开数据库失败: {e}");
            std::process::exit(1);
        }
    };

    let (topics_n, cards_n) = match import_cards(&conn, &src.join("cards")) {
        Ok(x) => x,
        Err(e) => {
            eprintln!("❌ 导入卡片失败: {e}");
            std::process::exit(1);
        }
    };

    let updated = import_progress(&conn, &src.join("progress.md")).unwrap_or(0);

    println!("✅ 迁移完成");
    println!("   源: {}", src.display());
    println!("   topics: {topics_n}  cards: {cards_n}  progress 行更新: {updated}");
    println!("   数据库: {}", db::db_path().display());
}

fn default_src() -> PathBuf {
    let home = std::env::var("HOME").expect("HOME 未设置");
    PathBuf::from(home).join(".pi/learning-data")
}

// ───────────────────── 卡片导入 ─────────────────────

struct RawCard {
    id: String,
    front: String,
    back: String,
    ef: f64,
    interval: i64,
    reps: i64,
    due: NaiveDate,
    created: NaiveDate,
}

fn parse_card(text: &str) -> Option<RawCard> {
    let rest = text.strip_prefix("---\n")?;
    let end = rest.find("\n---\n")?;
    let meta_str = &rest[..end];
    let body = &rest[end + "\n---\n".len()..];

    let mut meta: HashMap<&str, &str> = HashMap::new();
    for line in meta_str.lines() {
        if let Some((k, v)) = line.split_once(':') {
            meta.insert(k.trim(), v.trim());
        }
    }
    let (front, back) = match body.split_once("\n---\n") {
        Some((f, b)) => (f.trim().to_string(), b.trim().to_string()),
        None => (body.trim().to_string(), String::new()),
    };
    let get = |k: &str| meta.get(k).copied().unwrap_or("");
    Some(RawCard {
        id: get("id").to_string(),
        front,
        back,
        ef: get("ef").parse().unwrap_or(2.5),
        interval: get("interval").parse().unwrap_or(0),
        reps: get("reps").parse().unwrap_or(0),
        due: NaiveDate::parse_from_str(get("due"), "%Y-%m-%d").ok()?,
        created: NaiveDate::parse_from_str(get("created"), "%Y-%m-%d").ok()?,
    })
}

fn import_cards(conn: &Connection, cards_dir: &Path) -> rusqlite::Result<(usize, usize)> {
    let mut topic_ids: HashMap<String, String> = HashMap::new();
    let mut cards_n = 0;

    let entries = match fs::read_dir(cards_dir) {
        Ok(e) => e,
        Err(_) => {
            eprintln!("⚠️  cards 目录不存在或为空: {}", cards_dir.display());
            return Ok((0, 0));
        }
    };

    let now = Utc::now().to_rfc3339();
    for entry in entries.flatten() {
        let topic_dir = entry.path();
        if !topic_dir.is_dir() {
            continue;
        }
        let topic_name = topic_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        let tid = ensure_topic(conn, &topic_name, &mut topic_ids)?;

        let sub = match fs::read_dir(&topic_dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for md in sub.flatten() {
            let path = md.path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            let text = match fs::read_to_string(&path) {
                Ok(t) => t,
                Err(_) => continue,
            };
            let card = match parse_card(&text) {
                Some(c) => c,
                None => {
                    eprintln!("⚠️  跳过（解析失败）: {}", path.display());
                    continue;
                }
            };
            conn.execute(
                "INSERT OR REPLACE INTO cards
                 (id, topic, front, back, ef, interval, reps, due, created, updated)
                 VALUES (?,?,?,?,?,?,?,?,?,?)",
                params![
                    card.id,
                    tid,
                    card.front,
                    card.back,
                    card.ef,
                    card.interval,
                    card.reps,
                    card.due.to_string(),
                    card.created.to_string(),
                    now,
                ],
            )?;
            cards_n += 1;
        }
    }
    Ok((topic_ids.len(), cards_n))
}

fn ensure_topic(
    conn: &Connection,
    name: &str,
    cache: &mut HashMap<String, String>,
) -> rusqlite::Result<String> {
    if let Some(id) = cache.get(name) {
        return Ok(id.clone());
    }
    let existing: Option<String> = conn
        .query_row("SELECT id FROM topics WHERE name=?", params![name], |r| {
            r.get(0)
        })
        .ok();
    let id = existing.unwrap_or_else(gen_topic_id);
    conn.execute(
        "INSERT OR IGNORE INTO topics (id, name) VALUES (?, ?)",
        params![id, name],
    )?;
    cache.insert(name.to_string(), id.clone());
    Ok(id)
}

fn gen_topic_id() -> String {
    let date = Utc::now().date_naive().format("%Y-%m-%d");
    let hex = uuid::Uuid::new_v4().simple().to_string();
    format!("topic-{date}-{}", &hex[..6])
}

// ─────────────────── progress.md 解析 ───────────────────
// best-effort：`| 主题 | 阶段 | 上次学习 | 下次计划 | 到期复习 |`
fn import_progress(conn: &Connection, path: &Path) -> rusqlite::Result<usize> {
    let text = match fs::read_to_string(path) {
        Ok(t) => t,
        Err(_) => return Ok(0),
    };
    let mut n = 0;
    for line in text.lines() {
        let line = line.trim();
        if !line.starts_with('|') || line.contains("---") {
            continue; // 表头 / 分隔行
        }
        let cells: Vec<&str> = line
            .trim_matches('|')
            .split('|')
            .map(|s| s.trim())
            .collect();
        if cells.len() < 4 {
            continue;
        }
        let name = cells[0];
        let stage = cells.get(1).copied().unwrap_or("");
        let last_studied = cells
            .get(2)
            .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
            .map(|d| d.to_string());
        let next_plan = cells.get(3).copied().unwrap_or("");
        let rows = conn.execute(
            "UPDATE topics SET
                stage = COALESCE(NULLIF(?, ''), stage),
                last_studied = ?,
                next_plan = COALESCE(NULLIF(?, ''), next_plan)
             WHERE name = ?",
            params![stage, last_studied, next_plan, name],
        )?;
        n += rows;
    }
    Ok(n)
}
