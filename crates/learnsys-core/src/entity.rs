//! 领域实体：Card / Topic / ReviewLog。
//!
//! 这些是平台存储的原子单位。SM-2 调度字段（ef/interval/reps/due）随复习流动，
//! 由 [`crate::sm2`] 计算、由 [`crate::repo`] 持久化。实体本身不持有任何行为逻辑。

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

// ───────────────────────── Card ─────────────────────────

/// 一张知识卡片，SM-2 调度的最小单位。一张卡 = 一个原子知识点。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub id: String,
    pub topic: String,
    pub front: String,
    pub back: String,
    /// 难度系数 Easy Factor（1.3–2.8），新卡默认 2.5；越低越难。
    pub ef: f64,
    /// 当前复习间隔（天）。
    pub interval: i64,
    /// 连续成功复习次数。
    pub reps: i64,
    /// 下次到期日。
    pub due: NaiveDate,
    pub created: NaiveDate,
    pub updated: DateTime<Utc>,
    /// 所属模块（nullable：散卡无模块）。
    pub module_id: Option<String>,
    /// 标签（自由文本数组，JSON 存储）。
    pub tags: Vec<String>,
    /// 代码块（如 Rust 代码片段）。
    pub code_block: Option<String>,
    /// 配图 URL 列表。
    pub image_urls: Vec<String>,
    /// 出处（来自哪个视频 / 文章 / 文档，URL 或描述）。
    pub source: Option<String>,
    /// 关联卡片 id 列表（双向链接，底层同源的知识点互相指）。
    pub related: Vec<String>,
}

impl Card {
    /// 新建一张卡片，SM-2 字段归零、due=今天（立刻进入复习队列）。
    pub fn new(
        topic: impl Into<String>,
        front: impl Into<String>,
        back: impl Into<String>,
    ) -> Self {
        let today = Utc::now().date_naive();
        Self {
            id: gen_id(),
            topic: topic.into(),
            front: front.into(),
            back: back.into(),
            ef: 2.5,
            interval: 0,
            reps: 0,
            due: today,
            created: today,
            updated: Utc::now(),
            module_id: None,
            tags: vec![],
            code_block: None,
            image_urls: vec![],
            source: None,
            related: vec![],
        }
    }
}

/// 卡片编辑补丁：`None` = 不改，`Some` = 替换。`code_block` 用 `Some("")` 表示清空。
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CardPatch {
    pub front: Option<String>,
    pub back: Option<String>,
    /// 主题 **id**（API 层已解析）。
    pub topic: Option<String>,
    pub tags: Option<Vec<String>>,
    pub code_block: Option<String>,
    pub image_urls: Option<Vec<String>>,
    /// 挂到模块：`Some("")` 脱离，`Some(id)` 挂到模块，`None` 不改。
    pub module_id: Option<String>,
    /// 出处：`Some("")` 清空，`Some(v)` 设置，`None` 不改。
    pub source: Option<String>,
    /// 关联卡片 id 列表：`None` 不改，`Some(list)` 替换。
    pub related: Option<Vec<String>>,
}

/// 目标编辑补丁。
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct GoalPatch {
    pub title: Option<String>,
    pub description: Option<String>,
    pub success_criteria: Option<String>,
}

/// 路径编辑补丁。
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PathwayPatch {
    pub name: Option<String>,
    pub methodology: Option<String>,
    pub description: Option<String>,
}

/// 模块编辑补丁。
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ModulePatch {
    pub title: Option<String>,
    pub description: Option<String>,
}

// ──────────────────────── Topic ─────────────────────────

/// 主题/学习计划单位。stage 是自由文本（承接现有 progress.md 的"阶段"），不做强 schema。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TopicStatus {
    #[default]
    Active,
    Completed,
    Paused,
}

impl TopicStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Completed => "completed",
            Self::Paused => "paused",
        }
    }
    pub fn parse(s: &str) -> Self {
        match s.trim() {
            "completed" => Self::Completed,
            "paused" => Self::Paused,
            _ => Self::Active,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topic {
    pub id: String,
    pub name: String,
    pub stage: String,
    pub status: TopicStatus,
    pub last_studied: Option<NaiveDate>,
    pub next_plan: String,
    pub created: NaiveDate,
}

impl Topic {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: gen_id(),
            name: name.into(),
            stage: String::new(),
            status: TopicStatus::Active,
            last_studied: None,
            next_plan: String::new(),
            created: Utc::now().date_naive(),
        }
    }
}

// ────────────────────── ReviewLog ───────────────────────

/// 复习记录（不可变流水）。用于统计/审计，不参与调度计算——调度状态在 Card 上。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewLog {
    pub id: i64,
    pub card_id: String,
    /// 复习自评质量分 0–5。
    pub quality: i64,
    pub reviewed_at: DateTime<Utc>,
    pub prev_due: Option<NaiveDate>,
    pub new_due: NaiveDate,
}

// ───────────────────────── Goal ─────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GoalStatus {
    #[default]
    Active,
    Achieved,
    Abandoned,
}
impl GoalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Achieved => "achieved",
            Self::Abandoned => "abandoned",
        }
    }
    pub fn parse(s: &str) -> Self {
        match s.trim() {
            "achieved" => Self::Achieved,
            "abandoned" => Self::Abandoned,
            _ => Self::Active,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub id: String,
    pub title: String,
    pub description: String,
    pub success_criteria: String,
    pub topic: Option<String>,
    pub status: GoalStatus,
    pub created: NaiveDate,
    pub achieved_at: Option<NaiveDate>,
}
impl Goal {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            id: gen_id(),
            title: title.into(),
            description: String::new(),
            success_criteria: String::new(),
            topic: None,
            status: GoalStatus::Active,
            created: Utc::now().date_naive(),
            achieved_at: None,
        }
    }
}

// ──────────────────────── Pathway ────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pathway {
    pub id: String,
    pub name: String,
    pub methodology: String,
    pub description: String,
    pub goal_id: String,
    pub is_active: bool,
    pub created: NaiveDate,
}
impl Pathway {
    pub fn new(name: impl Into<String>, goal_id: impl Into<String>) -> Self {
        Self {
            id: gen_id(),
            name: name.into(),
            methodology: String::new(),
            description: String::new(),
            goal_id: goal_id.into(),
            is_active: false,
            created: Utc::now().date_naive(),
        }
    }
}

// ──────────────────────── Module ─────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModuleStatus {
    #[default]
    NotStarted,
    Learning,
    Mastered,
}
impl ModuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NotStarted => "not_started",
            Self::Learning => "learning",
            Self::Mastered => "mastered",
        }
    }
    pub fn parse(s: &str) -> Self {
        match s.trim() {
            "learning" => Self::Learning,
            "mastered" => Self::Mastered,
            _ => Self::NotStarted,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    pub id: String,
    pub title: String,
    pub topic: Option<String>,
    pub description: String,
    pub status: ModuleStatus,
}
impl Module {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            id: gen_id(),
            title: title.into(),
            topic: None,
            description: String::new(),
            status: ModuleStatus::NotStarted,
        }
    }
}

// ──────────────────── PathwayModule ──────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathwayModule {
    pub pathway_id: String,
    pub module_id: String,
    pub sort_order: i64,
    /// 前置模块 id 列表（简单依赖）。
    pub depends_on: Vec<String>,
}
impl PathwayModule {
    pub fn new(
        pathway_id: impl Into<String>,
        module_id: impl Into<String>,
        sort_order: i64,
    ) -> Self {
        Self {
            pathway_id: pathway_id.into(),
            module_id: module_id.into(),
            sort_order,
            depends_on: vec![],
        }
    }
}

// ──────────────────────── Session ────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: i64,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub goal_id: Option<String>,
    pub pathway_id: Option<String>,
    pub summary: String,
    pub new_cards: i64,
    pub reviewed: i64,
}

// ──────────────────── Resource ─────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub id: String,
    pub title: String,
    pub url: String,
    pub notes: String,
    pub module_id: Option<String>,
    pub card_id: Option<String>,
    pub created: NaiveDate,
}
impl Resource {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            id: gen_id(),
            title: title.into(),
            url: String::new(),
            notes: String::new(),
            module_id: None,
            card_id: None,
            created: Utc::now().date_naive(),
        }
    }
}

// ──────────────────── LearnerProfile ─────────────────────

/// AI 温和记忆（单例，id=1）。半结构化：固定字段 + 自由 notes。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnerProfile {
    pub id: i64,
    pub level: String,
    pub style: String,
    pub weak_points: Vec<String>,
    pub preferences: serde_json::Value,
    pub notes: String,
    pub updated: DateTime<Utc>,
}
impl Default for LearnerProfile {
    fn default() -> Self {
        Self {
            id: 1,
            level: String::new(),
            style: String::new(),
            weak_points: vec![],
            preferences: serde_json::json!({}),
            notes: String::new(),
            updated: Utc::now(),
        }
    }
}

// ───────────────────────── 公用 ─────────────────────────

/// 生成卡片/主题 id：`YYYY-MM-DD-<6位hex>`，对齐现有 Python 版格式。
fn gen_id() -> String {
    let date = Utc::now().date_naive().format("%Y-%m-%d");
    let hex = uuid::Uuid::new_v4().simple().to_string();
    format!("{date}-{}", &hex[..6])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_card_defaults() {
        let c = Card::new("rust", "什么是所有权?", "每值只有一个所有者");
        assert_eq!(c.topic, "rust");
        assert_eq!(c.ef, 2.5);
        assert_eq!(c.interval, 0);
        assert_eq!(c.reps, 0);
        assert!(c
            .id
            .starts_with(&Utc::now().date_naive().format("%Y-%m-%d").to_string()));
    }

    #[test]
    fn topic_status_roundtrip() {
        for s in ["active", "completed", "paused"] {
            assert_eq!(TopicStatus::parse(s).as_str(), s);
        }
        assert_eq!(TopicStatus::parse("garbage").as_str(), "active");
    }
}
