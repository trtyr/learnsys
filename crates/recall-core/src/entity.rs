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
        }
    }
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
        assert!(c.id.starts_with(&Utc::now().date_naive().format("%Y-%m-%d").to_string()));
    }

    #[test]
    fn topic_status_roundtrip() {
        for s in ["active", "completed", "paused"] {
            assert_eq!(TopicStatus::parse(s).as_str(), s);
        }
        assert_eq!(TopicStatus::parse("garbage").as_str(), "active");
    }
}
