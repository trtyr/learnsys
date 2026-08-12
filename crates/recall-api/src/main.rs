//! recall-api —— axum 服务入口。
//!
//! REST API（见 docs/plantree/.../api-contract.md）：
//!   POST   /api/cards            建卡片（topic 用名，不存在则自动建主题）
//!   GET    /api/cards?topic=     列卡片
//!   GET    /api/cards/due?topic= 今日待复习
//!   GET    /api/cards/:id        取一张
//!   DELETE /api/cards/:id        删卡
//!   POST   /api/cards/:id/review 记录复习 {quality:0-5} → SM-2 调度

use std::sync::{Arc, Mutex};

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Json, Router,
};
use chrono::{NaiveDate, Utc};
use serde::Deserialize;
use serde_json::{json, Value};
use tower_http::cors::CorsLayer;

use recall_core::entity::{Card, Topic, TopicStatus};
use recall_core::repo::{self, RepoError};

#[derive(Clone)]
struct AppState {
    db: Arc<Mutex<rusqlite::Connection>>,
}

#[tokio::main]
async fn main() {
    let conn = match recall_core::db::connect() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ 打开数据库失败: {e}");
            std::process::exit(1);
        }
    };
    let state = AppState {
        db: Arc::new(Mutex::new(conn)),
    };

    let app = Router::new()
        .route("/", get(health))
        .route("/api/cards", post(create_card).get(list_cards))
        .route("/api/cards/due", get(due_cards))
        .route("/api/cards/:id", get(get_card).delete(delete_card))
        .route("/api/cards/:id/review", post(review_card))
        .route("/api/topics", post(create_topic).get(list_topics))
        .route("/api/topics/:id", get(get_topic).put(update_topic))
        .route("/api/stats", get(stats))
        .route("/api/dashboard", get(dashboard))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = "127.0.0.1:7878";
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|e| panic!("绑定 {addr} 失败: {e}"));
    eprintln!("recall-api listening on http://{addr}");
    axum::serve(listener, app).await.unwrap();
}

// ───────────────────── 健康检查 ─────────────────────

async fn health() -> Json<Value> {
    Json(json!({
        "name": "recall",
        "status": "ok",
        "tagline": "headless learning data platform",
        "note": "AI calls the API; the platform has no AI of its own."
    }))
}

// ─────────────────────── DTO ───────────────────────

#[derive(Deserialize)]
struct CreateCard {
    topic: String,
    front: String,
    back: String,
}

#[derive(Deserialize)]
struct TopicQuery {
    topic: Option<String>,
}

#[derive(Deserialize)]
struct ReviewBody {
    quality: i64,
}

// ────────────────────── 错误模型 ──────────────────────

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(json!({ "code": self.code, "message": self.message })),
        )
            .into_response()
    }
}

impl From<RepoError> for ApiError {
    fn from(e: RepoError) -> Self {
        match e {
            RepoError::NotFound(what) => ApiError {
                status: StatusCode::NOT_FOUND,
                code: "not_found",
                message: what,
            },
            RepoError::Sqlite(e) => ApiError {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                code: "db_error",
                message: e.to_string(),
            },
            RepoError::Date(e) => ApiError {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                code: "date_error",
                message: e.to_string(),
            },
        }
    }
}

// ─────────────────────── handlers ───────────────────────

/// 把 card.topic（id）替换成 topic 名，供 API 对外展示。
fn name_topic(db: &rusqlite::Connection, card: &mut Card) {
    card.topic = recall_core::repo::get_topic(db, &card.topic)
        .map(|t| t.name)
        .unwrap_or_else(|_| card.topic.clone());
}

async fn create_card(
    State(s): State<AppState>,
    Json(body): Json<CreateCard>,
) -> Result<(StatusCode, Json<Card>), ApiError> {
    let db = s.db.lock().unwrap();
    // topic 用名：不存在则自动建一个空主题
    let topic = match repo::get_topic_by_name(&db, &body.topic) {
        Ok(t) => t,
        Err(RepoError::NotFound(_)) => {
            let t = Topic::new(&body.topic);
            repo::upsert_topic(&db, &t)?;
            t
        }
        Err(e) => return Err(e.into()),
    };
    let mut card = Card::new(topic.id, body.front, body.back);
    repo::insert_card(&db, &card)?;
    name_topic(&db, &mut card);
    Ok((StatusCode::CREATED, Json(card)))
}

async fn list_cards(
    State(s): State<AppState>,
    Query(q): Query<TopicQuery>,
) -> Result<Json<Vec<Card>>, ApiError> {
    let db = s.db.lock().unwrap();
    let mut cards = repo::list_cards(&db, q.topic.as_deref())?;
    for c in &mut cards {
        name_topic(&db, c);
    }
    Ok(Json(cards))
}

async fn due_cards(
    State(s): State<AppState>,
    Query(q): Query<TopicQuery>,
) -> Result<Json<Vec<Card>>, ApiError> {
    let db = s.db.lock().unwrap();
    let mut cards = repo::due_cards(&db, Utc::now().date_naive(), q.topic.as_deref())?;
    for c in &mut cards {
        name_topic(&db, c);
    }
    Ok(Json(cards))
}

async fn get_card(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Card>, ApiError> {
    let db = s.db.lock().unwrap();
    let mut card = repo::get_card(&db, &id)?;
    name_topic(&db, &mut card);
    Ok(Json(card))
}

async fn delete_card(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let db = s.db.lock().unwrap();
    repo::delete_card(&db, &id)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn review_card(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<ReviewBody>,
) -> Result<Json<Card>, ApiError> {
    let db = s.db.lock().unwrap();
    let mut card = repo::review_card(&db, &id, body.quality, Utc::now().date_naive())?;
    name_topic(&db, &mut card);
    Ok(Json(card))
}

// ─────────────────── topics ───────────────────

#[derive(Deserialize)]
struct CreateTopic {
    name: String,
}

async fn create_topic(
    State(s): State<AppState>,
    Json(body): Json<CreateTopic>,
) -> Result<(StatusCode, Json<Topic>), ApiError> {
    let db = s.db.lock().unwrap();
    let t = Topic::new(body.name);
    repo::upsert_topic(&db, &t)?;
    Ok((StatusCode::CREATED, Json(t)))
}

async fn list_topics(State(s): State<AppState>) -> Result<Json<Vec<Topic>>, ApiError> {
    let db = s.db.lock().unwrap();
    Ok(Json(repo::list_topics(&db)?))
}

async fn get_topic(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Topic>, ApiError> {
    let db = s.db.lock().unwrap();
    Ok(Json(repo::get_topic(&db, &id)?))
}

#[derive(Deserialize)]
struct UpdateTopic {
    stage: Option<String>,
    status: Option<String>,
    next_plan: Option<String>,
    last_studied: Option<String>, // ISO YYYY-MM-DD
}

async fn update_topic(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateTopic>,
) -> Result<Json<Topic>, ApiError> {
    let db = s.db.lock().unwrap();
    let mut t = repo::get_topic(&db, &id)?;
    if let Some(stage) = body.stage {
        t.stage = stage;
    }
    if let Some(status) = body.status {
        t.status = TopicStatus::parse(&status);
    }
    if let Some(next_plan) = body.next_plan {
        t.next_plan = next_plan;
    }
    if let Some(ls) = body.last_studied {
        t.last_studied = NaiveDate::parse_from_str(&ls, "%Y-%m-%d").ok();
    }
    repo::upsert_topic(&db, &t)?;
    Ok(Json(t))
}

// ─────────────────── 聚合 ───────────────────

async fn stats(State(s): State<AppState>) -> Result<Json<repo::Stats>, ApiError> {
    let db = s.db.lock().unwrap();
    Ok(Json(repo::stats(&db, Utc::now().date_naive())?))
}

async fn dashboard(State(s): State<AppState>) -> Result<Json<repo::Dashboard>, ApiError> {
    let db = s.db.lock().unwrap();
    Ok(Json(repo::dashboard(&db, Utc::now().date_naive())?))
}
