//! SQLite 仓储层：Card / Topic / ReviewLog 的 CRUD。
//!
//! `review_card` 是 SM-2 调度的唯一入口：读卡 → 跑算法 → 原子更新卡 + 追加复习记录，
//! 全程在一个事务里。其余函数尽量直白。

use chrono::{DateTime, NaiveDate, Utc};
use rusqlite::{params, Connection, Row};

use crate::entity::{
    Card, CardPatch, Goal, GoalStatus, LearnerProfile, Module, ModuleStatus, Pathway,
    PathwayModule, Resource, ReviewLog, Session, Topic, TopicStatus,
};
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

/// 把 JSON 数组列解析为字符串列表，失败返回空列表。
fn parse_json_list(s: &str) -> Vec<String> {
    serde_json::from_str(s).unwrap_or_default()
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
    let tags: String = r.get("tags")?;
    let image_urls: String = r.get("image_urls")?;
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
        module_id: r.get("module_id")?,
        tags: parse_json_list(&tags),
        code_block: r.get("code_block")?,
        image_urls: parse_json_list(&image_urls),
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
         (id, topic, module_id, tags, code_block, image_urls, front, back, ef, interval, reps, due, created, updated)
         VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
        params![
            c.id,
            c.topic,
            c.module_id,
            serde_json::to_string(&c.tags).unwrap_or_else(|_| String::from("[]")),
            c.code_block,
            serde_json::to_string(&c.image_urls).unwrap_or_else(|_| String::from("[]")),
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
    conn.query_row(
        "SELECT * FROM cards WHERE id = ?",
        params![id],
        card_from_row,
    )
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

/// 到期复习卡片（reps>0 且 due<=today）。`topic` 为主题**名**。新卡走 [`new_cards`]。
pub fn due_cards(conn: &Connection, today: NaiveDate, topic: Option<&str>) -> Result<Vec<Card>> {
    let today_s = to_date_str(today);
    let cards = match topic {
        Some(name) => {
            let mut stmt = conn.prepare(
                "SELECT c.* FROM cards c JOIN topics t ON c.topic = t.id
                 WHERE c.reps > 0 AND c.due <= ? AND t.name = ? ORDER BY c.due",
            )?;
            let rows = stmt.query_map(params![today_s, name], card_from_row)?;
            rows.collect::<rusqlite::Result<Vec<_>>>()?
        }
        None => {
            let mut stmt =
                conn.prepare("SELECT * FROM cards WHERE reps > 0 AND due <= ? ORDER BY due")?;
            let rows = stmt.query_map(params![today_s], card_from_row)?;
            rows.collect::<rusqlite::Result<Vec<_>>>()?
        }
    };
    Ok(cards)
}

/// 今天已消耗的新卡预算 = 今天首次复习（is_new=1）的新卡数。
pub fn new_introduced_today(conn: &Connection, today: NaiveDate) -> i64 {
    let s = to_date_str(today);
    conn.query_row(
        "SELECT count(*) FROM review_logs WHERE is_new = 1 AND substr(reviewed_at, 1, 10) = ?",
        params![s],
        |r| r.get(0),
    )
    .unwrap_or(0)
}

/// 新卡（从未复习，reps=0 且今天到期）——返回**剩余每日预算**张（new_per_day - 今天已学新卡数）。
pub fn new_cards(conn: &Connection, today: NaiveDate) -> Result<Vec<Card>> {
    let remaining = (new_per_day(conn) - new_introduced_today(conn, today)).max(0);
    let today_s = to_date_str(today);
    let mut stmt = conn
        .prepare("SELECT * FROM cards WHERE reps = 0 AND due <= ? ORDER BY created, id LIMIT ?")?;
    let rows = stmt.query_map(params![today_s, remaining], card_from_row)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

/// 顽固卡（leech）：EF < 1.5，或最近 4 次复习全失败（quality < 3）。
/// 只标记不处置——识别是平台的事，处置归 AI/人。
pub fn leech_cards(conn: &Connection) -> Result<Vec<Card>> {
    let mut leeches = Vec::new();
    for c in list_cards(conn, None)? {
        if c.ef < 1.5 {
            leeches.push(c);
            continue;
        }
        let logs = list_logs_by_card(conn, &c.id)?;
        let recent: Vec<i64> = logs.iter().take(4).map(|l| l.quality).collect();
        if recent.len() >= 4 && recent.iter().all(|&q| q < 3) {
            leeches.push(c);
        }
    }
    Ok(leeches)
}

fn has_review_on(conn: &Connection, d: NaiveDate) -> bool {
    let s = to_date_str(d);
    conn.query_row(
        "SELECT count(*) FROM review_logs WHERE substr(reviewed_at, 1, 10) = ?",
        params![s],
        |r| r.get::<_, i64>(0),
    )
    .map(|n| n > 0)
    .unwrap_or(false)
}

/// 连续学习天数（streak）。今天没学则从昨天起算，不被"今天还没学"打断。
pub fn streak(conn: &Connection, today: NaiveDate) -> i64 {
    let mut d = today;
    if !has_review_on(conn, d) {
        d -= chrono::Duration::days(1);
    }
    let mut days = 0;
    while has_review_on(conn, d) {
        days += 1;
        d -= chrono::Duration::days(1);
    }
    days
}

/// 测验抽取：从到期复习卡里随机抽 `n` 张（可选按主题名过滤）。
pub fn quiz_cards(
    conn: &Connection,
    today: NaiveDate,
    n: i64,
    topic: Option<&str>,
) -> Result<Vec<Card>> {
    let today_s = to_date_str(today);
    let cards = match topic {
        Some(name) => {
            let mut stmt = conn.prepare(
                "SELECT c.* FROM cards c JOIN topics t ON c.topic = t.id
                 WHERE c.reps > 0 AND c.due <= ? AND t.name = ? ORDER BY RANDOM() LIMIT ?",
            )?;
            let rows = stmt.query_map(params![today_s, name, n], card_from_row)?;
            rows.collect::<rusqlite::Result<Vec<_>>>()?
        }
        None => {
            let mut stmt = conn.prepare(
                "SELECT * FROM cards WHERE reps > 0 AND due <= ? ORDER BY RANDOM() LIMIT ?",
            )?;
            let rows = stmt.query_map(params![today_s, n], card_from_row)?;
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
pub fn review_card(conn: &Connection, id: &str, quality: i64, today: NaiveDate) -> Result<Card> {
    let tx = conn.unchecked_transaction()?;
    let mut card: Card = tx
        .query_row(
            "SELECT * FROM cards WHERE id = ?",
            params![id],
            card_from_row,
        )
        .map_err(notfound("card", id))?;

    let is_new = card.reps == 0;
    let s = sm2::sm2(card.ef, card.interval, card.reps, quality, today);
    let now = Utc::now();
    tx.execute(
        "UPDATE cards SET ef=?, interval=?, reps=?, due=?, updated=? WHERE id=?",
        params![
            s.ef,
            s.interval,
            s.reps,
            to_date_str(s.due),
            now.to_rfc3339(),
            id
        ],
    )?;
    tx.execute(
        "INSERT INTO review_logs (card_id, quality, reviewed_at, prev_due, new_due, is_new)
         VALUES (?,?,?,?,?,?)",
        params![
            id,
            quality,
            now.to_rfc3339(),
            to_date_str(card.due),
            to_date_str(s.due),
            is_new
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

/// 编辑卡片：应用补丁的非 `None` 字段，不触碰 SM-2 调度状态。
pub fn update_card(conn: &Connection, id: &str, patch: &CardPatch) -> Result<Card> {
    let mut card = get_card(conn, id)?;
    if let Some(front) = &patch.front {
        card.front = front.clone();
    }
    if let Some(back) = &patch.back {
        card.back = back.clone();
    }
    if let Some(topic) = &patch.topic {
        card.topic = topic.clone();
    }
    if let Some(tags) = &patch.tags {
        card.tags = tags.clone();
    }
    if let Some(code_block) = &patch.code_block {
        card.code_block = if code_block.is_empty() {
            None
        } else {
            Some(code_block.clone())
        };
    }
    if let Some(image_urls) = &patch.image_urls {
        card.image_urls = image_urls.clone();
    }
    card.updated = Utc::now();
    conn.execute(
        "UPDATE cards SET topic=?, tags=?, code_block=?, image_urls=?, front=?, back=?, updated=? WHERE id=?",
        params![
            card.topic,
            serde_json::to_string(&card.tags).unwrap_or_else(|_| String::from("[]")),
            card.code_block,
            serde_json::to_string(&card.image_urls).unwrap_or_else(|_| String::from("[]")),
            card.front,
            card.back,
            card.updated.to_rfc3339(),
            id,
        ],
    )?;
    Ok(card)
}

/// 搜索卡片：LIKE 子串匹配 front/back/tags，可选按主题名过滤。
pub fn search_cards(conn: &Connection, q: &str, topic: Option<&str>) -> Result<Vec<Card>> {
    let pattern = format!("%{q}%");
    let cards = match topic {
        Some(name) => {
            let mut stmt = conn.prepare(
                "SELECT c.* FROM cards c JOIN topics t ON c.topic = t.id
                 WHERE (c.front LIKE ? OR c.back LIKE ? OR c.tags LIKE ?) AND t.name = ?
                 ORDER BY c.due",
            )?;
            let rows = stmt.query_map(params![pattern, pattern, pattern, name], card_from_row)?;
            rows.collect::<rusqlite::Result<Vec<_>>>()?
        }
        None => {
            let mut stmt = conn.prepare(
                "SELECT * FROM cards WHERE front LIKE ? OR back LIKE ? OR tags LIKE ?
                 ORDER BY due",
            )?;
            let rows = stmt.query_map(params![pattern, pattern, pattern], card_from_row)?;
            rows.collect::<rusqlite::Result<Vec<_>>>()?
        }
    };
    Ok(cards)
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
    conn.query_row(
        "SELECT * FROM topics WHERE id = ?",
        params![id],
        topic_from_row,
    )
    .map_err(notfound("topic", id))
}

pub fn get_topic_by_name(conn: &Connection, name: &str) -> Result<Topic> {
    conn.query_row(
        "SELECT * FROM topics WHERE name = ?",
        params![name],
        topic_from_row,
    )
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

// ──────────────────── Settings ────────────────────

pub fn get_setting(conn: &Connection, key: &str) -> Option<String> {
    conn.query_row(
        "SELECT value FROM settings WHERE key = ?",
        params![key],
        |r| r.get(0),
    )
    .ok()
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?, ?)
         ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        params![key, value],
    )?;
    Ok(())
}

/// 每日新卡预算（默认 5）。
pub fn new_per_day(conn: &Connection) -> i64 {
    get_setting(conn, "new_per_day")
        .and_then(|v| v.parse().ok())
        .unwrap_or(5)
}

// ───────────────────── Goal ─────────────────────

fn goal_from_row(r: &Row) -> rusqlite::Result<Goal> {
    let created: String = r.get("created")?;
    let achieved: Option<String> = r.get("achieved_at")?;
    let status: String = r.get("status")?;
    Ok(Goal {
        id: r.get("id")?,
        title: r.get("title")?,
        description: r.get("description")?,
        success_criteria: r.get("success_criteria")?,
        topic: r.get("topic")?,
        status: GoalStatus::parse(&status),
        created: conv(parse_date(&created))?,
        achieved_at: achieved.as_deref().and_then(|s| parse_date(s).ok()),
    })
}

pub fn insert_goal(conn: &Connection, g: &Goal) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO goals (id,title,description,success_criteria,topic,status,created,achieved_at) VALUES (?,?,?,?,?,?,?,?)",
        params![g.id, g.title, g.description, g.success_criteria, g.topic, g.status.as_str(), to_date_str(g.created), g.achieved_at.map(to_date_str)],
    )?;
    Ok(())
}

pub fn get_goal(conn: &Connection, id: &str) -> Result<Goal> {
    conn.query_row("SELECT * FROM goals WHERE id=?", params![id], goal_from_row)
        .map_err(notfound("goal", id))
}

pub fn list_goals(conn: &Connection) -> Result<Vec<Goal>> {
    let mut stmt = conn.prepare("SELECT * FROM goals ORDER BY created DESC")?;
    let rows = stmt.query_map([], goal_from_row)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn update_goal_status(
    conn: &Connection,
    id: &str,
    status: GoalStatus,
    achieved_at: Option<NaiveDate>,
) -> Result<()> {
    let n = conn.execute(
        "UPDATE goals SET status=?, achieved_at=? WHERE id=?",
        params![status.as_str(), achieved_at.map(to_date_str), id],
    )?;
    if n == 0 {
        return Err(RepoError::NotFound(format!("goal {id}")));
    }
    Ok(())
}

// ──────────────────── Pathway ────────────────────

fn pathway_from_row(r: &Row) -> rusqlite::Result<Pathway> {
    let created: String = r.get("created")?;
    Ok(Pathway {
        id: r.get("id")?,
        name: r.get("name")?,
        methodology: r.get("methodology")?,
        description: r.get("description")?,
        goal_id: r.get("goal_id")?,
        is_active: r.get::<_, i64>("is_active")? != 0,
        created: conv(parse_date(&created))?,
    })
}

pub fn insert_pathway(conn: &Connection, p: &Pathway) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO pathways (id,name,methodology,description,goal_id,is_active,created) VALUES (?,?,?,?,?,?,?)",
        params![p.id, p.name, p.methodology, p.description, p.goal_id, p.is_active as i64, to_date_str(p.created)],
    )?;
    Ok(())
}

pub fn get_pathway(conn: &Connection, id: &str) -> Result<Pathway> {
    conn.query_row(
        "SELECT * FROM pathways WHERE id=?",
        params![id],
        pathway_from_row,
    )
    .map_err(notfound("pathway", id))
}

pub fn list_pathways_by_goal(conn: &Connection, goal_id: &str) -> Result<Vec<Pathway>> {
    let mut stmt = conn.prepare("SELECT * FROM pathways WHERE goal_id=? ORDER BY created")?;
    let rows = stmt.query_map(params![goal_id], pathway_from_row)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

// ──────────────────── Module ────────────────────

fn module_from_row(r: &Row) -> rusqlite::Result<Module> {
    let status: String = r.get("status")?;
    Ok(Module {
        id: r.get("id")?,
        title: r.get("title")?,
        topic: r.get("topic")?,
        description: r.get("description")?,
        status: ModuleStatus::parse(&status),
    })
}

pub fn insert_module(conn: &Connection, m: &Module) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO modules (id,title,topic,description,status) VALUES (?,?,?,?,?)",
        params![m.id, m.title, m.topic, m.description, m.status.as_str()],
    )?;
    Ok(())
}

pub fn get_module(conn: &Connection, id: &str) -> Result<Module> {
    conn.query_row(
        "SELECT * FROM modules WHERE id=?",
        params![id],
        module_from_row,
    )
    .map_err(notfound("module", id))
}

pub fn list_modules(conn: &Connection, topic: Option<&str>) -> Result<Vec<Module>> {
    let (sql, params_opt) = match topic {
        Some(t) => (
            "SELECT * FROM modules WHERE topic=? ORDER BY title",
            Some(t.to_string()),
        ),
        None => ("SELECT * FROM modules ORDER BY title", None),
    };
    let mut stmt = conn.prepare(sql)?;
    let rows = match &params_opt {
        Some(p) => stmt.query_map(params![p.as_str()], module_from_row)?,
        None => stmt.query_map([], module_from_row)?,
    };
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

// ──────────────── PathwayModule ─────────────────

fn pm_from_row(r: &Row) -> rusqlite::Result<PathwayModule> {
    let deps: String = r.get("depends_on").unwrap_or_default();
    Ok(PathwayModule {
        pathway_id: r.get("pathway_id")?,
        module_id: r.get("module_id")?,
        sort_order: r.get("sort_order")?,
        depends_on: if deps.is_empty() {
            vec![]
        } else {
            deps.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        },
    })
}

pub fn insert_pathway_module(conn: &Connection, pm: &PathwayModule) -> Result<()> {
    let deps = pm.depends_on.join(",");
    conn.execute(
        "INSERT OR REPLACE INTO pathway_modules (pathway_id,module_id,sort_order,depends_on) VALUES (?,?,?,?)",
        params![pm.pathway_id, pm.module_id, pm.sort_order, deps],
    )?;
    Ok(())
}

pub fn list_pathway_modules(conn: &Connection, pathway_id: &str) -> Result<Vec<PathwayModule>> {
    let mut stmt =
        conn.prepare("SELECT * FROM pathway_modules WHERE pathway_id=? ORDER BY sort_order")?;
    let rows = stmt.query_map(params![pathway_id], pm_from_row)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

/// 给定路径，按顺序 + 依赖计算下一个可学模块。
/// 返回 (模块, 当前序号, 总模块数)。None = 路径完成或被卡住。
pub fn next_module(conn: &Connection, pathway_id: &str) -> Result<Option<(Module, usize, usize)>> {
    let pms = list_pathway_modules(conn, pathway_id)?;
    if pms.is_empty() {
        return Ok(None);
    }
    let mastered: std::collections::HashSet<String> = pms
        .iter()
        .filter_map(|pm| {
            let m = get_module(conn, &pm.module_id).ok()?;
            if matches!(m.status, ModuleStatus::Mastered) {
                Some(pm.module_id.clone())
            } else {
                None
            }
        })
        .collect();
    for (i, pm) in pms.iter().enumerate() {
        if mastered.contains(&pm.module_id) {
            continue;
        }
        if pm.depends_on.iter().all(|d| mastered.contains(d)) {
            let m = get_module(conn, &pm.module_id)?;
            return Ok(Some((m, i + 1, pms.len())));
        }
    }
    Ok(None)
}

// ──────────────────── Session ────────────────────

fn session_from_row(r: &Row) -> rusqlite::Result<Session> {
    let started: String = r.get("started_at")?;
    let ended: Option<String> = r.get("ended_at")?;
    Ok(Session {
        id: r.get("id")?,
        goal_id: r.get("goal_id")?,
        pathway_id: r.get("pathway_id")?,
        summary: r.get("summary")?,
        new_cards: r.get("new_cards")?,
        reviewed: r.get("reviewed")?,
        started_at: conv(parse_dt(&started))?,
        ended_at: ended.as_deref().and_then(|s| parse_dt(s).ok()),
    })
}

pub fn start_session(
    conn: &Connection,
    goal_id: Option<&str>,
    pathway_id: Option<&str>,
) -> Result<Session> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO sessions (started_at, goal_id, pathway_id) VALUES (?,?,?)",
        params![now, goal_id, pathway_id],
    )?;
    let id = conn.last_insert_rowid();
    Ok(Session {
        id,
        started_at: Utc::now(),
        ended_at: None,
        goal_id: goal_id.map(String::from),
        pathway_id: pathway_id.map(String::from),
        summary: String::new(),
        new_cards: 0,
        reviewed: 0,
    })
}

pub fn end_session(
    conn: &Connection,
    id: i64,
    summary: &str,
    new_cards: i64,
    reviewed: i64,
) -> Result<()> {
    let n = conn.execute(
        "UPDATE sessions SET ended_at=?, summary=?, new_cards=?, reviewed=? WHERE id=?",
        params![Utc::now().to_rfc3339(), summary, new_cards, reviewed, id],
    )?;
    if n == 0 {
        return Err(RepoError::NotFound(format!("session {id}")));
    }
    Ok(())
}

pub fn list_sessions(conn: &Connection, limit: i64) -> Result<Vec<Session>> {
    let mut stmt = conn.prepare("SELECT * FROM sessions ORDER BY started_at DESC LIMIT ?")?;
    let rows = stmt.query_map(params![limit], session_from_row)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

// ───────────────── LearnerProfile ────────────────

fn profile_from_row(r: &Row) -> rusqlite::Result<LearnerProfile> {
    let updated: String = r.get("updated")?;
    let wp: String = r.get("weak_points")?;
    let prefs: String = r.get("preferences")?;
    Ok(LearnerProfile {
        id: r.get("id")?,
        level: r.get("level")?,
        style: r.get("style")?,
        notes: r.get("notes")?,
        weak_points: serde_json::from_str(&wp).unwrap_or_default(),
        preferences: serde_json::from_str(&prefs).unwrap_or_else(|_| serde_json::json!({})),
        updated: conv(parse_dt(&updated))?,
    })
}

pub fn get_profile(conn: &Connection) -> Result<LearnerProfile> {
    conn.query_row(
        "SELECT * FROM learner_profile WHERE id=1",
        [],
        profile_from_row,
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => RepoError::NotFound("learner_profile".into()),
        other => RepoError::Sqlite(other),
    })
}

pub fn upsert_profile(conn: &Connection, p: &LearnerProfile) -> Result<()> {
    conn.execute(
        "INSERT INTO learner_profile (id, level, style, weak_points, preferences, notes, updated)
         VALUES (1,?,?,?,?,?,?)
         ON CONFLICT(id) DO UPDATE SET
            level=excluded.level, style=excluded.style,
            weak_points=excluded.weak_points, preferences=excluded.preferences,
            notes=excluded.notes, updated=excluded.updated",
        params![
            p.level,
            p.style,
            serde_json::to_string(&p.weak_points).unwrap_or_default(),
            p.preferences.to_string(),
            p.notes,
            p.updated.to_rfc3339(),
        ],
    )?;
    Ok(())
}

// ──────────────── 进度聚合 ────────────────

#[derive(Debug, serde::Serialize)]
pub struct ModuleMastery {
    pub module_id: String,
    pub total_cards: i64,
    /// 已有正向复习记录的卡片数（reps>0，粗略"已掌握"）
    pub learned: i64,
    pub avg_ef: f64,
    pub due_count: i64,
}

pub fn module_mastery(conn: &Connection, module_id: &str) -> Result<ModuleMastery> {
    let total: i64 = conn.query_row(
        "SELECT count(*) FROM cards WHERE module_id=?",
        params![module_id],
        |r| r.get(0),
    )?;
    let learned: i64 = conn.query_row(
        "SELECT count(*) FROM cards WHERE module_id=? AND reps>0",
        params![module_id],
        |r| r.get(0),
    )?;
    let avg_ef: f64 = conn.query_row(
        "SELECT COALESCE(AVG(ef),0) FROM cards WHERE module_id=?",
        params![module_id],
        |r| r.get(0),
    )?;
    let due: i64 = conn.query_row(
        "SELECT count(*) FROM cards WHERE module_id=? AND due<=?",
        params![module_id, to_date_str(Utc::now().date_naive())],
        |r| r.get(0),
    )?;
    Ok(ModuleMastery {
        module_id: module_id.to_string(),
        total_cards: total,
        learned,
        avg_ef,
        due_count: due,
    })
}

pub fn update_module_status(conn: &Connection, id: &str, status: ModuleStatus) -> Result<()> {
    let n = conn.execute(
        "UPDATE modules SET status=? WHERE id=?",
        params![status.as_str(), id],
    )?;
    if n == 0 {
        return Err(RepoError::NotFound(format!("module {id}")));
    }
    Ok(())
}

// ──────────────── resources ────────────────

fn resource_from_row(r: &Row) -> rusqlite::Result<Resource> {
    let created: String = r.get("created")?;
    Ok(Resource {
        id: r.get("id")?,
        title: r.get("title")?,
        url: r.get("url")?,
        notes: r.get("notes")?,
        module_id: r.get("module_id")?,
        card_id: r.get("card_id")?,
        created: conv(parse_date(&created))?,
    })
}

pub fn insert_resource(conn: &Connection, r: &Resource) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO resources (id,title,url,notes,module_id,card_id,created) VALUES (?,?,?,?,?,?,?)",
        params![r.id, r.title, r.url, r.notes, r.module_id, r.card_id, to_date_str(r.created)],
    )?;
    Ok(())
}

pub fn list_resources(
    conn: &Connection,
    module_id: Option<&str>,
    card_id: Option<&str>,
) -> Result<Vec<Resource>> {
    let (sql, param) = if let Some(mid) = module_id {
        (
            "SELECT * FROM resources WHERE module_id=? ORDER BY created",
            Some(mid.to_string()),
        )
    } else if let Some(cid) = card_id {
        (
            "SELECT * FROM resources WHERE card_id=? ORDER BY created",
            Some(cid.to_string()),
        )
    } else {
        (
            "SELECT * FROM resources ORDER BY created DESC LIMIT 50",
            None,
        )
    };
    let mut stmt = conn.prepare(sql)?;
    let rows = match &param {
        Some(p) => stmt.query_map(params![p.as_str()], resource_from_row)?,
        None => stmt.query_map([], resource_from_row)?,
    };
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

// ──────────────── heatmap ────────────────

#[derive(Debug, serde::Serialize)]
pub struct HeatmapDay {
    pub date: String,
    pub count: i64,
}

pub fn heatmap(conn: &Connection, days: i64) -> Result<Vec<HeatmapDay>> {
    let since = to_date_str(Utc::now().date_naive() - chrono::Duration::days(days));
    let mut stmt = conn.prepare(
        "SELECT substr(reviewed_at,1,10) as d, count(*) FROM review_logs WHERE d >= ? GROUP BY d ORDER BY d"
    )?;
    let rows = stmt.query_map(params![since], |r| {
        Ok(HeatmapDay {
            date: r.get(0)?,
            count: r.get(1)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

// ──────────────── goal progress ────────────────

#[derive(Debug, serde::Serialize)]
pub struct GoalProgress {
    pub goal_id: String,
    pub total_modules: usize,
    pub mastered: usize,
    pub percent: f64,
}

pub fn goal_progress(conn: &Connection, goal_id: &str) -> Result<GoalProgress> {
    let pws = list_pathways_by_goal(conn, goal_id)?;
    let mut mids = std::collections::HashSet::new();
    for pw in &pws {
        for pm in &list_pathway_modules(conn, &pw.id)? {
            mids.insert(pm.module_id.clone());
        }
    }
    let total = mids.len();
    let mastered = mids
        .iter()
        .filter(|mid| {
            get_module(conn, mid)
                .map(|m| matches!(m.status, ModuleStatus::Mastered))
                .unwrap_or(false)
        })
        .count();
    Ok(GoalProgress {
        goal_id: goal_id.to_string(),
        total_modules: total,
        mastered,
        percent: if total > 0 {
            (mastered as f64) / (total as f64) * 100.0
        } else {
            0.0
        },
    })
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
    pub due_soon: i64,  // 明后天到期（预警）
    pub new_cards: i64, // 今日可学新卡（reps=0）
    pub avg_ef: f64,
    pub by_topic: Vec<TopicCount>,
}

#[derive(Debug, serde::Serialize)]
pub struct Dashboard {
    pub due_today: i64,
    pub due_soon: i64,
    pub leech_count: i64,
    pub streak: i64,
    pub studied_today: bool,
    pub active_topics: Vec<Topic>,
    pub stats: Stats,
}

pub fn stats(conn: &Connection, today: NaiveDate) -> Result<Stats> {
    let today_s = to_date_str(today);
    let soon_s = to_date_str(today + chrono::Duration::days(2));
    let total: i64 = conn.query_row("SELECT count(*) FROM cards", [], |r| r.get(0))?;
    let due_today: i64 = conn.query_row(
        "SELECT count(*) FROM cards WHERE reps > 0 AND due <= ?",
        params![today_s],
        |r| r.get(0),
    )?;
    let new_cards: i64 = conn.query_row(
        "SELECT count(*) FROM cards WHERE reps = 0 AND due <= ?",
        params![today_s],
        |r| r.get(0),
    )?;
    let due_soon: i64 = conn.query_row(
        "SELECT count(*) FROM cards WHERE due > ? AND due <= ?",
        params![today_s, soon_s],
        |r| r.get(0),
    )?;
    let avg_ef: f64 = conn.query_row("SELECT COALESCE(AVG(ef), 0) FROM cards", [], |r| r.get(0))?;
    let mut stmt = conn.prepare(
        "SELECT t.name, count(c.id) FROM topics t LEFT JOIN cards c ON c.topic = t.id
         GROUP BY t.id, t.name ORDER BY count(c.id) DESC, t.name",
    )?;
    let by_topic = stmt
        .query_map([], |r| {
            Ok(TopicCount {
                topic: r.get(0)?,
                count: r.get(1)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(Stats {
        total_cards: total,
        due_today,
        due_soon,
        new_cards,
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
    let leech_count = leech_cards(conn)?.len() as i64;
    let streak = streak(conn, today);
    let studied_today = has_review_on(conn, today);
    Ok(Dashboard {
        due_today: stats.due_today,
        due_soon: stats.due_soon,
        leech_count,
        streak,
        studied_today,
        active_topics,
        stats,
    })
}

// ──────────────────── Export / Backup ────────────────────

#[derive(Debug, serde::Serialize)]
pub struct Export {
    pub topics: Vec<Topic>,
    pub cards: Vec<Card>,
    pub review_logs: Vec<ReviewLog>,
    pub goals: Vec<Goal>,
    pub pathways: Vec<Pathway>,
    pub modules: Vec<Module>,
    pub pathway_modules: Vec<PathwayModule>,
    pub sessions: Vec<Session>,
    pub resources: Vec<Resource>,
    pub profile: Option<LearnerProfile>,
}

/// 全量导出（JSON 用）。所有实体一次性拉出，无 LIMIT 截断。
pub fn export_all(conn: &Connection) -> Result<Export> {
    let pathways = {
        let mut stmt = conn.prepare("SELECT * FROM pathways ORDER BY created")?;
        let rows = stmt.query_map([], pathway_from_row)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()?
    };
    let pathway_modules = {
        let mut stmt =
            conn.prepare("SELECT * FROM pathway_modules ORDER BY pathway_id, sort_order")?;
        let rows = stmt.query_map([], pm_from_row)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()?
    };
    let review_logs = {
        let mut stmt = conn.prepare("SELECT * FROM review_logs ORDER BY reviewed_at")?;
        let rows = stmt.query_map([], log_from_row)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()?
    };
    let resources = {
        let mut stmt = conn.prepare("SELECT * FROM resources ORDER BY created")?;
        let rows = stmt.query_map([], resource_from_row)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()?
    };
    Ok(Export {
        topics: list_topics(conn)?,
        cards: list_cards(conn, None)?,
        review_logs,
        goals: list_goals(conn)?,
        pathways,
        modules: list_modules(conn, None)?,
        pathway_modules,
        sessions: list_sessions(conn, 100_000)?,
        resources,
        profile: get_profile(conn).ok(),
    })
}

/// markdown 聚合导出：卡片按主题分组，每张卡是 migrate 兼容的 frontmatter 块。
pub fn export_markdown(conn: &Connection) -> Result<String> {
    let cards = list_cards(conn, None)?;
    let topics = list_topics(conn)?;
    let name_of = |id: &str| {
        topics
            .iter()
            .find(|t| t.id == id)
            .map(|t| t.name.clone())
            .unwrap_or_else(|| id.to_string())
    };
    let mut by_topic: std::collections::BTreeMap<String, Vec<Card>> =
        std::collections::BTreeMap::new();
    for c in cards {
        by_topic.entry(name_of(&c.topic)).or_default().push(c);
    }
    let mut out = format!("# learnsys 导出 · {}\n\n", Utc::now().date_naive());
    for (topic, cards) in by_topic {
        out.push_str(&format!("## {topic}\n\n"));
        for c in cards {
            out.push_str(&format!(
                "---\nid: {}\nef: {}\ninterval: {}\nreps: {}\ndue: {}\ncreated: {}\ntags: {}\ncode_block: {}\nimage_urls: {}\n---\n{}\n---\n{}\n\n",
                c.id,
                c.ef,
                c.interval,
                c.reps,
                c.due,
                c.created,
                serde_json::to_string(&c.tags).unwrap_or_else(|_| String::from("[]")),
                serde_json::to_string(&c.code_block).unwrap_or_else(|_| String::from("null")),
                serde_json::to_string(&c.image_urls).unwrap_or_else(|_| String::from("[]")),
                c.front,
                c.back
            ));
        }
    }
    Ok(out)
}

/// 一致性快照备份（`VACUUM INTO` 生成独立库文件，可在线备份）。
pub fn backup(conn: &Connection, dest: &str) -> Result<()> {
    conn.execute("VACUUM INTO ?", params![dest])?;
    Ok(())
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
        // 新卡（reps=0）进 new 队列，不进 due 队列
        assert_eq!(new_cards(&c, Utc::now().date_naive()).unwrap().len(), 1);
        assert_eq!(
            due_cards(&c, Utc::now().date_naive(), None).unwrap().len(),
            0
        );

        delete_card(&c, &card.id).unwrap();
        assert!(get_card(&c, &card.id).is_err());
    }

    #[test]
    fn new_and_review_separated_after_review() {
        let c = mem();
        let t = seed_topic(&c);

        // 新卡：reps=0 → new 队列
        let newc = Card::new(t.id.clone(), "new", "a");
        insert_card(&c, &newc).unwrap();

        // 复习卡：reps>0 且 due<=today → due 队列
        let mut rev = Card::new(t.id.clone(), "rev", "a");
        rev.reps = 3;
        rev.interval = 5;
        rev.due = Utc::now().date_naive();
        insert_card(&c, &rev).unwrap();

        assert_eq!(new_cards(&c, Utc::now().date_naive()).unwrap().len(), 1);
        assert_eq!(
            due_cards(&c, Utc::now().date_naive(), None).unwrap().len(),
            1
        );

        // 复习一次后，新卡离开 new 队列（reps 0→1，due 推到明天）
        review_card(&c, &newc.id, 5, Utc::now().date_naive()).unwrap();
        assert_eq!(new_cards(&c, Utc::now().date_naive()).unwrap().len(), 0);
    }

    #[test]
    fn new_card_daily_budget_consumed_by_review() {
        let c = mem();
        let t = seed_topic(&c);
        set_setting(&c, "new_per_day", "2").unwrap();

        let mut ids = Vec::new();
        for i in 0..5 {
            let card = Card::new(t.id.clone(), format!("q{i}"), "a");
            ids.push(card.id.clone());
            insert_card(&c, &card).unwrap();
        }
        let today = Utc::now().date_naive();

        // 初始返回预算张（2）；重复调用（未复习）预算不消耗
        assert_eq!(new_cards(&c, today).unwrap().len(), 2);
        assert_eq!(new_cards(&c, today).unwrap().len(), 2);

        // 复习 1 张新卡 → 剩余预算 1
        review_card(&c, &ids[0], 4, today).unwrap();
        assert_eq!(new_introduced_today(&c, today), 1);
        assert_eq!(new_cards(&c, today).unwrap().len(), 1);

        // 再复习 1 张 → 预算耗尽
        review_card(&c, &ids[1], 4, today).unwrap();
        assert_eq!(new_introduced_today(&c, today), 2);
        assert_eq!(new_cards(&c, today).unwrap().len(), 0);
    }

    #[test]
    fn leech_detection_by_low_ef_and_repeated_failures() {
        let c = mem();
        let t = seed_topic(&c);

        // 低 EF 卡
        let mut low = Card::new(t.id.clone(), "low ef", "a");
        low.ef = 1.4;
        insert_card(&c, &low).unwrap();

        // 高 EF 但连续 4 次失败（q=2 每次 -0.32，4 次后 EF 1.52 >= 1.5，仍触发）
        let mut failing = Card::new(t.id.clone(), "failing", "a");
        failing.ef = 2.8;
        insert_card(&c, &failing).unwrap();
        for _ in 0..4 {
            review_card(&c, &failing.id, 2, Utc::now().date_naive()).unwrap();
        }

        let leeches = leech_cards(&c).unwrap();
        assert_eq!(leeches.len(), 2);
        assert!(leeches.iter().any(|x| x.id == low.id));
        assert!(leeches.iter().any(|x| x.id == failing.id));
    }

    #[test]
    fn settings_roundtrip_and_default() {
        let c = mem();
        assert_eq!(new_per_day(&c), 5);
        set_setting(&c, "new_per_day", "8").unwrap();
        assert_eq!(new_per_day(&c), 8);
        set_setting(&c, "new_per_day", "3").unwrap();
        assert_eq!(new_per_day(&c), 3);
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

    #[test]
    fn card_update_preserves_sm2_and_edits_content() {
        let c = mem();
        let t = seed_topic(&c);
        let card = Card::new(t.id.clone(), "q", "a");
        insert_card(&c, &card).unwrap();

        // 先复习一次，让 SM-2 状态脱离默认
        let after = review_card(&c, &card.id, 5, Utc::now().date_naive()).unwrap();
        assert_eq!(after.reps, 1);

        // 编辑 front + tags + code_block，不重置 SM-2
        let patch = CardPatch {
            front: Some("q2".into()),
            tags: Some(vec!["borrow".into(), "rust".into()]),
            code_block: Some("let x = 1;".into()),
            ..Default::default()
        };
        let updated = update_card(&c, &card.id, &patch).unwrap();
        assert_eq!(updated.front, "q2");
        assert_eq!(updated.tags, vec!["borrow", "rust"]);
        assert_eq!(updated.code_block.as_deref(), Some("let x = 1;"));
        assert_eq!(updated.reps, 1, "编辑不改 SM-2");
        assert_eq!(updated.ef, after.ef);

        // 空串清空 code_block
        let clear = CardPatch {
            code_block: Some(String::new()),
            ..Default::default()
        };
        let updated2 = update_card(&c, &card.id, &clear).unwrap();
        assert_eq!(updated2.code_block, None);
    }

    #[test]
    fn search_matches_front_back_and_tags() {
        let c = mem();
        let t = seed_topic(&c);
        let mut c1 = Card::new(t.id.clone(), "所有权是什么", "独占");
        c1.tags = vec!["rust".into(), "基础".into()];
        c1.due = Utc::now().date_naive();
        insert_card(&c, &c1).unwrap();
        let mut c2 = Card::new(t.id.clone(), "borrow checker", "编译期检查");
        c2.due = Utc::now().date_naive();
        insert_card(&c, &c2).unwrap();

        assert_eq!(search_cards(&c, "borrow", None).unwrap().len(), 1);
        assert_eq!(search_cards(&c, "独占", None).unwrap().len(), 1);
        assert_eq!(search_cards(&c, "基础", None).unwrap().len(), 1);
        assert_eq!(search_cards(&c, "不存在的词", None).unwrap().len(), 0);
    }

    #[test]
    fn export_all_captures_everything() {
        let c = mem();
        let t = seed_topic(&c);
        let card = Card::new(t.id.clone(), "q", "a");
        insert_card(&c, &card).unwrap();
        let e = export_all(&c).unwrap();
        assert_eq!(e.topics.len(), 1);
        assert_eq!(e.cards.len(), 1);
        assert_eq!(e.review_logs.len(), 0);
        assert_eq!(e.goals.len(), 0);
    }

    #[test]
    fn export_markdown_contains_content_fields() {
        let c = mem();
        let t = seed_topic(&c);
        let mut card = Card::new(t.id.clone(), "什么是所有权", "独占");
        card.tags = vec!["rust".into(), "基础".into()];
        card.code_block = Some("let x = 1;".into());
        card.image_urls = vec!["https://example.com/a.png".into()];
        insert_card(&c, &card).unwrap();
        let md = export_markdown(&c).unwrap();
        assert!(md.contains("## rust"));
        assert!(md.contains("什么是所有权"));
        assert!(md.contains("独占"));
        assert!(md.contains("\"rust\""));
        assert!(md.contains("\"let x = 1;\""));
        assert!(md.contains("https://example.com/a.png"));
    }

    #[test]
    fn backup_restores_card() {
        let dir = std::env::temp_dir().join(format!("learnsys-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let db_path = dir.join("main.db");
        let conn = Connection::open(&db_path).unwrap();
        schema::init(&conn).unwrap();
        let t = Topic::new("rust");
        upsert_topic(&conn, &t).unwrap();
        let card = Card::new(t.id.clone(), "q", "a");
        insert_card(&conn, &card).unwrap();

        let backup_path = dir.join("backup.db");
        backup(&conn, backup_path.to_str().unwrap()).unwrap();

        let bconn = Connection::open(&backup_path).unwrap();
        assert_eq!(get_card(&bconn, &card.id).unwrap().front, "q");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn quiz_returns_at_most_n() {
        let c = mem();
        let t = seed_topic(&c);
        for i in 0..6 {
            let mut card = Card::new(t.id.clone(), format!("q{i}"), "a");
            card.reps = 1;
            card.due = Utc::now().date_naive();
            insert_card(&c, &card).unwrap();
        }
        assert_eq!(
            quiz_cards(&c, Utc::now().date_naive(), 3, None)
                .unwrap()
                .len(),
            3
        );
    }

    #[test]
    fn streak_counts_consecutive_days() {
        let c = mem();
        let t = seed_topic(&c);
        let card = Card::new(t.id.clone(), "q", "a");
        insert_card(&c, &card).unwrap();

        let today = Utc::now().date_naive();
        for i in 0..3 {
            let d = to_date_str(today - chrono::Duration::days(i));
            c.execute(
                "INSERT INTO review_logs (card_id, quality, reviewed_at, new_due) VALUES (?,?,?,?)",
                params![card.id, 5, format!("{d}T10:00:00+00:00"), d],
            )
            .unwrap();
        }
        assert_eq!(streak(&c, today), 3);
        assert!(has_review_on(&c, today));
    }
}
