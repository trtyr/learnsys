//! SQLite 仓储层：Card / Topic / ReviewLog 的 CRUD。
//!
//! `review_card` 是 SM-2 调度的唯一入口：读卡 → 跑算法 → 原子更新卡 + 追加复习记录，
//! 全程在一个事务里。其余函数尽量直白。

use chrono::{DateTime, NaiveDate, Utc};
use rusqlite::{params, Connection, Row};

use crate::entity::{Card, ReviewLog, Topic, TopicStatus};
use crate::sm2;

#[derive(Debug, thiserror::Error)]
pub enum RepoError {
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("date parse: {0}")]
    Date(#[from] chrono::ParseError),
    #[error("not found: {0}")]
    NotFound(String),
}

pub type Result<T> = std::result::Result<T, RepoError>;

// ────────────────── 转换 helper ──────────────────

fn parse_date(s: &str) -> Result<NaiveDate> {
    Ok(NaiveDate::parse_from_str(s, "%Y-%m-%d")?)
}

fn to_date_str(d: NaiveDate) -> String {
    d.format("%Y-%m-%d").to_string()
}

/// 兼容 RFC3339（我们写入）与 SQLite datetime（列默认值）。
fn parse_dt(s: &str) -> Result<DateTime<Utc>> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Ok(dt.with_timezone(&Utc));
    }
    Ok(DateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")?.with_timezone(&Utc))
}

/// 把领域层 parse 错误塞回 rusqlite 的行映射闭包里。
fn conv<T, E>(r: std::result::Result<T, E>) -> rusqlite::Result<T>
where
    E: std::error::Error + Send + Sync + 'static,
{
    r.map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })
}

fn notfound<'a>(kind: &'a str, id: &'a str) -> impl Fn(rusqlite::Error) -> RepoError + 'a {
    move |e| match e {
        rusqlite::Error::QueryReturnedNoRows => RepoError::NotFound(format!("{kind} {id}")),
        other => RepoError::Sqlite(other),
    }
}

fn card_from_row(r: &Row) -> rusqlite::Result<Card> {
    let due: String = r.get("due")?;
    let created: String = r.get("created")?;
    let updated: String = r.get("updated")?;
    Ok(Card {
        id: r.get("id")?,
        topic: r.get("topic")?,
        front: r.get("front")?,
        back: r.get("back")?,
        ef: r.get("ef")?,
        interval: r.get("interval")?,
        reps: r.get("reps")?,
        due: conv(parse_date(&due))?,
        created: conv(parse_date(&created))?,
        updated: conv(parse_dt(&updated))?,
    })
}

fn topic_from_row(r: &Row) -> rusqlite::Result<Topic> {
    let ls: Option<String> = r.get("last_studied")?;
    let created: String = r.get("created")?;
    let status: String = r.get("status")?;
    Ok(Topic {
        id: r.get("id")?,
        name: r.get("name")?,
        stage: r.get("stage")?,
        status: TopicStatus::parse(&status),
        last_studied: ls.as_deref().and_then(|s| parse_date(s).ok()),
        next_plan: r.get("next_plan")?,
        created: conv(parse_date(&created))?,
    })
}

fn log_from_row(r: &Row) -> rusqlite::Result<ReviewLog> {
    let reviewed_at: String = r.get("reviewed_at")?;
    let prev: Option<String> = r.get("prev_due")?;
    let new_due: String = r.get("new_due")?;
    Ok(ReviewLog {
        id: r.get("id")?,
        card_id: r.get("card_id")?,
        quality: r.get("quality")?,
        reviewed_at: conv(parse_dt(&reviewed_at))?,
        prev_due: prev.as_deref().and_then(|s| parse_date(s).ok()),
        new_due: conv(parse_date(&new_due))?,
    })
}

// ───────────────────── Card ─────────────────────

pub fn insert_card(conn: &Connection, c: &Card) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO cards
         (id, topic, front, back, ef, interval, reps, due, created, updated)
         VALUES (?,?,?,?,?,?,?,?,?,?)",
        params![
            c.id,
            c.topic,
            c.front,
            c.back,
            c.ef,
            c.interval,
            c.reps,
            to_date_str(c.due),
            to_date_str(c.created),
            c.updated.to_rfc3339(),
        ],
    )?;
    Ok(())
}

pub fn get_card(conn: &Connection, id: &str) -> Result<Card> {
    conn.query_row("SELECT * FROM cards WHERE id = ?", params![id], card_from_row)
        .map_err(notfound("card", id))
}

/// 列卡片。`topic` 为主题**名**（如 "rust"），None 则全部。
pub fn list_cards(conn: &Connection, topic: Option<&str>) -> Result<Vec<Card>> {
    let cards = match topic {
        Some(name) => {
            let mut stmt = conn.prepare(
                "SELECT c.* FROM cards c JOIN topics t ON c.topic = t.id
                 WHERE t.name = ? ORDER BY c.due",
            )?;
            let rows = stmt.query_map(params![name], card_from_row)?;
            rows.collect::<rusqlite::Result<Vec<_>>>()?
        }
        None => {
            let mut stmt = conn.prepare("SELECT * FROM cards ORDER BY due")?;
            let rows = stmt.query_map([], card_from_row)?;
            rows.collect::<rusqlite::Result<Vec<_>>>()?
        }
    };
    Ok(cards)
}

/// 到期卡片（due <= today）。`topic` 为主题**名**。
pub fn due_cards(conn: &Connection, today: NaiveDate, topic: Option<&str>) -> Result<Vec<Card>> {
    let today_s = to_date_str(today);
    let cards = match topic {
        Some(name) => {
            let mut stmt = conn.prepare(
                "SELECT c.* FROM cards c JOIN topics t ON c.topic = t.id
                 WHERE c.due <= ? AND t.name = ? ORDER BY c.due",
            )?;
            let rows = stmt.query_map(params![today_s, name], card_from_row)?;
            rows.collect::<rusqlite::Result<Vec<_>>>()?
        }
        None => {
            let mut stmt = conn.prepare("SELECT * FROM cards WHERE due <= ? ORDER BY due")?;
            let rows = stmt.query_map(params![today_s], card_from_row)?;
            rows.collect::<rusqlite::Result<Vec<_>>>()?
        }
    };
    Ok(cards)
}

pub fn delete_card(conn: &Connection, id: &str) -> Result<()> {
    let n = conn.execute("DELETE FROM cards WHERE id = ?", params![id])?;
    if n == 0 {
        return Err(RepoError::NotFound(format!("card {id}")));
    }
    Ok(())
}

/// 记录一次复习：跑 SM-2 → 原子更新卡 + 追加复习记录。返回更新后的卡。
pub fn review_card(
    conn: &Connection,
    id: &str,
    quality: i64,
    today: NaiveDate,
) -> Result<Card> {
    let tx = conn.unchecked_transaction()?;
    let mut card: Card = tx
        .query_row("SELECT * FROM cards WHERE id = ?", params![id], card_from_row)
        .map_err(notfound("card", id))?;

    let s = sm2::sm2(card.ef, card.interval, card.reps, quality, today);
    let now = Utc::now();
    tx.execute(
        "UPDATE cards SET ef=?, interval=?, reps=?, due=?, updated=? WHERE id=?",
        params![s.ef, s.interval, s.reps, to_date_str(s.due), now.to_rfc3339(), id],
    )?;
    tx.execute(
        "INSERT INTO review_logs (card_id, quality, reviewed_at, prev_due, new_due)
         VALUES (?,?,?,?,?)",
        params![
            id,
            quality,
            now.to_rfc3339(),
            to_date_str(card.due),
            to_date_str(s.due)
        ],
    )?;
    tx.commit()?;

    card.ef = s.ef;
    card.interval = s.interval;
    card.reps = s.reps;
    card.due = s.due;
    card.updated = now;
    Ok(card)
}

// ──────────────────── Topic ────────────────────

pub fn upsert_topic(conn: &Connection, t: &Topic) -> Result<()> {
    conn.execute(
        "INSERT INTO topics (id, name, stage, status, last_studied, next_plan, created)
         VALUES (?,?,?,?,?,?,?)
         ON CONFLICT(id) DO UPDATE SET
            name=excluded.name, stage=excluded.stage, status=excluded.status,
            last_studied=excluded.last_studied, next_plan=excluded.next_plan",
        params![
            t.id,
            t.name,
            t.stage,
            t.status.as_str(),
            t.last_studied.map(to_date_str),
            t.next_plan,
            to_date_str(t.created)
        ],
    )?;
    Ok(())
}

pub fn get_topic(conn: &Connection, id: &str) -> Result<Topic> {
    conn.query_row("SELECT * FROM topics WHERE id = ?", params![id], topic_from_row)
        .map_err(notfound("topic", id))
}

pub fn get_topic_by_name(conn: &Connection, name: &str) -> Result<Topic> {
    conn.query_row("SELECT * FROM topics WHERE name = ?", params![name], topic_from_row)
        .map_err(notfound("topic", name))
}

pub fn list_topics(conn: &Connection) -> Result<Vec<Topic>> {
    let mut stmt = conn.prepare("SELECT * FROM topics ORDER BY name")?;
    let rows = stmt.query_map([], topic_from_row)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

// ────────────────── ReviewLog ──────────────────

pub fn list_logs_by_card(conn: &Connection, card_id: &str) -> Result<Vec<ReviewLog>> {
    let mut stmt =
        conn.prepare("SELECT * FROM review_logs WHERE card_id = ? ORDER BY reviewed_at DESC")?;
    let rows = stmt.query_map(params![card_id], log_from_row)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

// ────────────────── 聚合视图（为 stats / dashboard 看板） ──────────────────

#[derive(Debug, serde::Serialize)]
pub struct TopicCount {
    pub topic: String,
    pub count: i64,
}

#[derive(Debug, serde::Serialize)]
pub struct Stats {
    pub total_cards: i64,
    pub due_today: i64,
    pub due_soon: i64, // 明后天到期（预警）
    pub avg_ef: f64,
    pub by_topic: Vec<TopicCount>,
}

#[derive(Debug, serde::Serialize)]
pub struct Dashboard {
    pub due_today: i64,
    pub due_soon: i64,
    pub active_topics: Vec<Topic>,
    pub stats: Stats,
}

pub fn stats(conn: &Connection, today: NaiveDate) -> Result<Stats> {
    let today_s = to_date_str(today);
    let soon_s = to_date_str(today + chrono::Duration::days(2));
    let total: i64 = conn.query_row("SELECT count(*) FROM cards", [], |r| r.get(0))?;
    let due_today: i64 = conn.query_row(
        "SELECT count(*) FROM cards WHERE due <= ?",
        params![today_s],
        |r| r.get(0),
    )?;
    let due_soon: i64 = conn.query_row(
        "SELECT count(*) FROM cards WHERE due > ? AND due <= ?",
        params![today_s, soon_s],
        |r| r.get(0),
    )?;
    let avg_ef: f64 =
        conn.query_row("SELECT COALESCE(AVG(ef), 0) FROM cards", [], |r| r.get(0))?;
    let mut stmt = conn.prepare(
        "SELECT t.name, count(c.id) FROM topics t LEFT JOIN cards c ON c.topic = t.id
         GROUP BY t.id, t.name ORDER BY count(c.id) DESC, t.name",
    )?;
    let by_topic = stmt
        .query_map([], |r| Ok(TopicCount { topic: r.get(0)?, count: r.get(1)? }))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(Stats {
        total_cards: total,
        due_today,
        due_soon,
        avg_ef,
        by_topic,
    })
}

pub fn dashboard(conn: &Connection, today: NaiveDate) -> Result<Dashboard> {
    let stats = stats(conn, today)?;
    let active_topics = list_topics(conn)?
        .into_iter()
        .filter(|t| matches!(t.status, TopicStatus::Active))
        .collect();
    Ok(Dashboard {
        due_today: stats.due_today,
        due_soon: stats.due_soon,
        active_topics,
        stats,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema;

    fn mem() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        schema::init(&c).unwrap();
        c
    }

    fn seed_topic(c: &Connection) -> Topic {
        let t = Topic::new("rust");
        upsert_topic(c, &t).unwrap();
        t
    }

    #[test]
    fn topic_upsert_get_list() {
        let c = mem();
        let t = seed_topic(&c);
        let by_name = get_topic_by_name(&c, "rust").unwrap();
        assert_eq!(by_name.id, t.id);
        assert_eq!(list_topics(&c).unwrap().len(), 1);
    }

    #[test]
    fn card_crud_and_due() {
        let c = mem();
        let t = seed_topic(&c);
        let mut card = Card::new(t.id.clone(), "q1", "a1");
        card.due = Utc::now().date_naive(); // 今天到期
        insert_card(&c, &card).unwrap();

        assert_eq!(get_card(&c, &card.id).unwrap().front, "q1");
        assert_eq!(list_cards(&c, Some("rust")).unwrap().len(), 1);
        assert_eq!(due_cards(&c, Utc::now().date_naive(), None).unwrap().len(), 1);

        delete_card(&c, &card.id).unwrap();
        assert!(get_card(&c, &card.id).is_err());
    }

    #[test]
    fn review_advances_and_logs() {
        let c = mem();
        let t = seed_topic(&c);
        let mut card = Card::new(t.id.clone(), "q", "a");
        card.due = Utc::now().date_naive();
        insert_card(&c, &card).unwrap();

        let today = Utc::now().date_naive();
        let after = review_card(&c, &card.id, 5, today).unwrap();
        assert_eq!(after.reps, 1);
        assert_eq!(after.interval, 1);
        assert!((after.ef - 2.6).abs() < 1e-9);

        let logs = list_logs_by_card(&c, &card.id).unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].quality, 5);
    }

    #[test]
    fn missing_card_is_not_found() {
        let c = mem();
        assert!(matches!(get_card(&c, "nope"), Err(RepoError::NotFound(_))));
    }
}
