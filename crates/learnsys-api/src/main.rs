//! learnsys-api —— axum 服务入口。
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

use learnsys_core::entity::{
    Card, CardPatch, Goal, GoalStatus, LearnerProfile, Module, ModuleStatus, Pathway,
    PathwayModule, Resource, Session, Topic, TopicStatus,
};
use learnsys_core::repo::{self, RepoError};

#[derive(Clone)]
struct AppState {
    db: Arc<Mutex<rusqlite::Connection>>,
}

#[tokio::main]
async fn main() {
    let conn = match learnsys_core::db::connect() {
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
        .route("/api/cards/search", get(search_cards_handler))
        .route("/api/cards/new", get(new_cards_handler))
        .route("/api/cards/leeches", get(leech_cards_handler))
        .route("/api/quiz", get(quiz_handler))
        .route(
            "/api/cards/:id",
            get(get_card).put(update_card_handler).delete(delete_card),
        )
        .route("/api/cards/:id/review", post(review_card))
        .route("/api/topics", post(create_topic).get(list_topics))
        .route("/api/topics/:id", get(get_topic).put(update_topic))
        .route("/api/stats", get(stats))
        .route("/api/dashboard", get(dashboard))
        .route("/api/goals", post(create_goal).get(list_goals))
        .route("/api/goals/:id", get(get_goal))
        .route("/api/goals/:id/status", put(update_goal_status))
        .route("/api/pathways", post(create_pathway).get(list_pathways))
        .route("/api/pathways/:id", get(get_pathway))
        .route(
            "/api/pathways/:id/modules",
            post(add_pathway_module).get(list_pathway_mods),
        )
        .route("/api/pathways/:id/next", get(next_module))
        .route("/api/modules", post(create_module).get(list_modules))
        .route("/api/sessions/start", post(session_start))
        .route("/api/sessions/:id/end", post(session_end))
        .route("/api/sessions", get(list_sessions))
        .route("/api/modules/:id/mastery", get(module_mastery))
        .route("/api/modules/:id/status", put(update_module_status))
        .route("/api/resources", post(create_resource).get(list_resources))
        .route("/api/stats/heatmap", get(heatmap))
        .route("/api/goals/:id/progress", get(goal_progress))
        .route("/api/profile", get(get_profile).put(upsert_profile))
        .route("/api/settings", get(get_settings).put(put_settings))
        .route("/api/export", get(export_handler))
        .route("/api/export/markdown", get(export_markdown_handler))
        .route("/api/backup", post(backup_handler))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = "127.0.0.1:7878";
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|e| panic!("绑定 {addr} 失败: {e}"));
    eprintln!("learnsys-api listening on http://{addr}");
    axum::serve(listener, app).await.unwrap();
}

// ───────────────────── 健康检查 ─────────────────────

async fn health() -> Json<Value> {
    Json(json!({
        "name": "learnsys",
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

#[derive(Deserialize)]
struct UpdateCard {
    front: Option<String>,
    back: Option<String>,
    topic: Option<String>,
    tags: Option<Vec<String>>,
    code_block: Option<String>,
    image_urls: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
    topic: Option<String>,
}

#[derive(Deserialize)]
struct SettingsBody {
    new_per_day: Option<i64>,
}

#[derive(Deserialize)]
struct QuizQuery {
    n: Option<i64>,
    topic: Option<String>,
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
    card.topic = learnsys_core::repo::get_topic(db, &card.topic)
        .map(|t| t.name)
        .unwrap_or_else(|_| card.topic.clone());
}

/// 按名取主题；不存在则新建（返回主题）。
fn resolve_topic_by_name(db: &rusqlite::Connection, name: &str) -> Result<Topic, ApiError> {
    match repo::get_topic_by_name(db, name) {
        Ok(t) => Ok(t),
        Err(RepoError::NotFound(_)) => {
            let t = Topic::new(name);
            repo::upsert_topic(db, &t)?;
            Ok(t)
        }
        Err(e) => Err(e.into()),
    }
}

async fn create_card(
    State(s): State<AppState>,
    Json(body): Json<CreateCard>,
) -> Result<(StatusCode, Json<Card>), ApiError> {
    let db = s.db.lock().unwrap();
    let topic = resolve_topic_by_name(&db, &body.topic)?;
    let mut card = Card::new(topic.id, body.front, body.back);
    repo::insert_card(&db, &card)?;
    name_topic(&db, &mut card);
    Ok((StatusCode::CREATED, Json(card)))
}

async fn update_card_handler(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateCard>,
) -> Result<Json<Card>, ApiError> {
    let db = s.db.lock().unwrap();
    let topic_id = match &body.topic {
        Some(name) => Some(resolve_topic_by_name(&db, name)?.id),
        None => None,
    };
    let patch = CardPatch {
        front: body.front,
        back: body.back,
        topic: topic_id,
        tags: body.tags,
        code_block: body.code_block,
        image_urls: body.image_urls,
    };
    let mut card = repo::update_card(&db, &id, &patch)?;
    name_topic(&db, &mut card);
    Ok(Json(card))
}

async fn search_cards_handler(
    State(s): State<AppState>,
    Query(q): Query<SearchQuery>,
) -> Result<Json<Vec<Card>>, ApiError> {
    let db = s.db.lock().unwrap();
    let mut cards = repo::search_cards(&db, &q.q, q.topic.as_deref())?;
    for c in &mut cards {
        name_topic(&db, c);
    }
    Ok(Json(cards))
}

async fn new_cards_handler(State(s): State<AppState>) -> Result<Json<Vec<Card>>, ApiError> {
    let db = s.db.lock().unwrap();
    let mut cards = repo::new_cards(&db, Utc::now().date_naive())?;
    for c in &mut cards {
        name_topic(&db, c);
    }
    Ok(Json(cards))
}

async fn leech_cards_handler(State(s): State<AppState>) -> Result<Json<Vec<Card>>, ApiError> {
    let db = s.db.lock().unwrap();
    let mut cards = repo::leech_cards(&db)?;
    for c in &mut cards {
        name_topic(&db, c);
    }
    Ok(Json(cards))
}

async fn quiz_handler(
    State(s): State<AppState>,
    Query(q): Query<QuizQuery>,
) -> Result<Json<Vec<Card>>, ApiError> {
    let db = s.db.lock().unwrap();
    let n = q.n.unwrap_or(5);
    let mut cards = repo::quiz_cards(&db, Utc::now().date_naive(), n, q.topic.as_deref())?;
    for c in &mut cards {
        name_topic(&db, c);
    }
    Ok(Json(cards))
}

async fn get_settings(State(s): State<AppState>) -> Result<Json<Value>, ApiError> {
    let db = s.db.lock().unwrap();
    Ok(Json(json!({ "new_per_day": repo::new_per_day(&db) })))
}

async fn put_settings(
    State(s): State<AppState>,
    Json(body): Json<SettingsBody>,
) -> Result<StatusCode, ApiError> {
    let db = s.db.lock().unwrap();
    if let Some(n) = body.new_per_day {
        repo::set_setting(&db, "new_per_day", &n.to_string())?;
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn export_handler(State(s): State<AppState>) -> Result<Json<repo::Export>, ApiError> {
    let db = s.db.lock().unwrap();
    Ok(Json(repo::export_all(&db)?))
}

async fn export_markdown_handler(State(s): State<AppState>) -> Result<String, ApiError> {
    let db = s.db.lock().unwrap();
    Ok(repo::export_markdown(&db)?)
}

async fn backup_handler(State(s): State<AppState>) -> Result<Json<Value>, ApiError> {
    let db = s.db.lock().unwrap();
    let dir = learnsys_core::db::db_path()
        .parent()
        .map(|p| p.join("backups"))
        .unwrap_or_else(|| std::path::PathBuf::from("backups"));
    std::fs::create_dir_all(&dir).ok();
    let ts = Utc::now().format("%Y%m%d-%H%M%S");
    let dest = dir.join(format!("learnsys-{ts}.db"));
    let dest_s = dest.to_string_lossy().to_string();
    repo::backup(&db, &dest_s)?;
    Ok(Json(json!({ "backup": dest_s })))
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

// ─────────────────── lms: goals ───────────────────

#[derive(Deserialize)]
struct CreateGoal {
    title: String,
    description: Option<String>,
    success_criteria: Option<String>,
    topic: Option<String>,
}

async fn create_goal(
    State(s): State<AppState>,
    Json(body): Json<CreateGoal>,
) -> Result<(StatusCode, Json<Goal>), ApiError> {
    let db = s.db.lock().unwrap();
    let mut g = Goal::new(body.title);
    g.description = body.description.unwrap_or_default();
    g.success_criteria = body.success_criteria.unwrap_or_default();
    g.topic = body.topic;
    repo::insert_goal(&db, &g)?;
    Ok((StatusCode::CREATED, Json(g)))
}

async fn list_goals(State(s): State<AppState>) -> Result<Json<Vec<Goal>>, ApiError> {
    let db = s.db.lock().unwrap();
    Ok(Json(repo::list_goals(&db)?))
}

async fn get_goal(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Goal>, ApiError> {
    let db = s.db.lock().unwrap();
    Ok(Json(repo::get_goal(&db, &id)?))
}

#[derive(Deserialize)]
struct UpdateGoalStatus {
    status: String,
    achieved_at: Option<String>,
}

async fn update_goal_status(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateGoalStatus>,
) -> Result<StatusCode, ApiError> {
    let db = s.db.lock().unwrap();
    let achieved = body
        .achieved_at
        .and_then(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok());
    repo::update_goal_status(&db, &id, GoalStatus::parse(&body.status), achieved)?;
    Ok(StatusCode::NO_CONTENT)
}

// ─────────────────── lms: pathways ───────────────────

#[derive(Deserialize)]
struct CreatePathway {
    name: String,
    goal_id: String,
    methodology: Option<String>,
    description: Option<String>,
}

async fn create_pathway(
    State(s): State<AppState>,
    Json(body): Json<CreatePathway>,
) -> Result<(StatusCode, Json<Pathway>), ApiError> {
    let db = s.db.lock().unwrap();
    let mut p = Pathway::new(body.name, body.goal_id);
    p.methodology = body.methodology.unwrap_or_default();
    p.description = body.description.unwrap_or_default();
    repo::insert_pathway(&db, &p)?;
    Ok((StatusCode::CREATED, Json(p)))
}

async fn get_pathway(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Pathway>, ApiError> {
    let db = s.db.lock().unwrap();
    Ok(Json(repo::get_pathway(&db, &id)?))
}

#[derive(Deserialize)]
struct PathwayQuery {
    goal: Option<String>,
}

async fn list_pathways(
    State(s): State<AppState>,
    Query(q): Query<PathwayQuery>,
) -> Result<Json<Vec<Pathway>>, ApiError> {
    let db = s.db.lock().unwrap();
    match q.goal {
        Some(gid) => Ok(Json(repo::list_pathways_by_goal(&db, &gid)?)),
        None => Err(ApiError {
            status: StatusCode::BAD_REQUEST,
            code: "missing_param",
            message: "需要 ?goal=".into(),
        }),
    }
}

// ─────────────────── lms: modules ───────────────────

#[derive(Deserialize)]
struct CreateModule {
    title: String,
    topic: Option<String>,
    description: Option<String>,
}

async fn create_module(
    State(s): State<AppState>,
    Json(body): Json<CreateModule>,
) -> Result<(StatusCode, Json<Module>), ApiError> {
    let db = s.db.lock().unwrap();
    let mut m = Module::new(body.title);
    m.topic = body.topic;
    m.description = body.description.unwrap_or_default();
    repo::insert_module(&db, &m)?;
    Ok((StatusCode::CREATED, Json(m)))
}

async fn list_modules(
    State(s): State<AppState>,
    Query(q): Query<TopicQuery>,
) -> Result<Json<Vec<Module>>, ApiError> {
    let db = s.db.lock().unwrap();
    Ok(Json(repo::list_modules(&db, q.topic.as_deref())?))
}

// ─────────────── lms: pathway ↔ modules ─────────────

#[derive(Deserialize)]
struct AddPathwayModule {
    module_id: String,
    sort_order: i64,
    depends_on: Option<Vec<String>>,
}

async fn add_pathway_module(
    State(s): State<AppState>,
    Path(pathway_id): Path<String>,
    Json(body): Json<AddPathwayModule>,
) -> Result<(StatusCode, Json<PathwayModule>), ApiError> {
    let db = s.db.lock().unwrap();
    let mut pm = PathwayModule::new(&pathway_id, body.module_id, body.sort_order);
    pm.depends_on = body.depends_on.unwrap_or_default();
    repo::insert_pathway_module(&db, &pm)?;
    Ok((StatusCode::CREATED, Json(pm)))
}

async fn list_pathway_mods(
    State(s): State<AppState>,
    Path(pathway_id): Path<String>,
) -> Result<Json<Vec<PathwayModule>>, ApiError> {
    let db = s.db.lock().unwrap();
    Ok(Json(repo::list_pathway_modules(&db, &pathway_id)?))
}

async fn next_module(
    State(s): State<AppState>,
    Path(pathway_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let db = s.db.lock().unwrap();
    match repo::next_module(&db, &pathway_id)? {
        Some((m, idx, total)) => Ok(Json(json!({"module": m, "position": idx, "total": total}))),
        None => Ok(Json(json!({"done": true}))),
    }
}

// ─────────────────── lms: sessions ──────────────────

#[derive(Deserialize)]
struct StartSession {
    goal_id: Option<String>,
    pathway_id: Option<String>,
}

async fn session_start(
    State(s): State<AppState>,
    Json(body): Json<StartSession>,
) -> Result<(StatusCode, Json<Session>), ApiError> {
    let db = s.db.lock().unwrap();
    let sess = repo::start_session(&db, body.goal_id.as_deref(), body.pathway_id.as_deref())?;
    Ok((StatusCode::CREATED, Json(sess)))
}

#[derive(Deserialize)]
struct EndSession {
    summary: Option<String>,
    new_cards: Option<i64>,
    reviewed: Option<i64>,
}

async fn session_end(
    State(s): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<EndSession>,
) -> Result<StatusCode, ApiError> {
    let db = s.db.lock().unwrap();
    repo::end_session(
        &db,
        id,
        &body.summary.unwrap_or_default(),
        body.new_cards.unwrap_or(0),
        body.reviewed.unwrap_or(0),
    )?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct SessionQuery {
    limit: Option<i64>,
}

async fn list_sessions(
    State(s): State<AppState>,
    Query(q): Query<SessionQuery>,
) -> Result<Json<Vec<Session>>, ApiError> {
    let db = s.db.lock().unwrap();
    Ok(Json(repo::list_sessions(&db, q.limit.unwrap_or(20))?))
}

async fn module_mastery(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<repo::ModuleMastery>, ApiError> {
    let db = s.db.lock().unwrap();
    Ok(Json(repo::module_mastery(&db, &id)?))
}

#[derive(Deserialize)]
struct UpdateModuleStatus {
    status: String,
}

async fn update_module_status(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateModuleStatus>,
) -> Result<StatusCode, ApiError> {
    let db = s.db.lock().unwrap();
    repo::update_module_status(&db, &id, ModuleStatus::parse(&body.status))?;
    Ok(StatusCode::NO_CONTENT)
}

// ─────────────── resources ───────────────

#[derive(Deserialize)]
struct CreateResource {
    title: String,
    url: Option<String>,
    notes: Option<String>,
    module_id: Option<String>,
    card_id: Option<String>,
}

async fn create_resource(
    State(s): State<AppState>,
    Json(body): Json<CreateResource>,
) -> Result<(StatusCode, Json<Resource>), ApiError> {
    let db = s.db.lock().unwrap();
    let mut r = Resource::new(body.title);
    r.url = body.url.unwrap_or_default();
    r.notes = body.notes.unwrap_or_default();
    r.module_id = body.module_id;
    r.card_id = body.card_id;
    repo::insert_resource(&db, &r)?;
    Ok((StatusCode::CREATED, Json(r)))
}

#[derive(Deserialize)]
struct ResourceQuery {
    module_id: Option<String>,
    card_id: Option<String>,
}

async fn list_resources(
    State(s): State<AppState>,
    Query(q): Query<ResourceQuery>,
) -> Result<Json<Vec<Resource>>, ApiError> {
    let db = s.db.lock().unwrap();
    Ok(Json(repo::list_resources(
        &db,
        q.module_id.as_deref(),
        q.card_id.as_deref(),
    )?))
}

// ─────────────── heatmap + goal progress ───────────────

#[derive(Deserialize)]
struct HeatmapQuery {
    days: Option<i64>,
}

async fn heatmap(
    State(s): State<AppState>,
    Query(q): Query<HeatmapQuery>,
) -> Result<Json<Vec<repo::HeatmapDay>>, ApiError> {
    let db = s.db.lock().unwrap();
    Ok(Json(repo::heatmap(&db, q.days.unwrap_or(90))?))
}

async fn goal_progress(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<repo::GoalProgress>, ApiError> {
    let db = s.db.lock().unwrap();
    Ok(Json(repo::goal_progress(&db, &id)?))
}

// ─────────────────── profile ───────────────────

async fn get_profile(State(s): State<AppState>) -> Result<Json<LearnerProfile>, ApiError> {
    let db = s.db.lock().unwrap();
    Ok(Json(repo::get_profile(&db)?))
}

async fn upsert_profile(
    State(s): State<AppState>,
    Json(body): Json<LearnerProfile>,
) -> Result<StatusCode, ApiError> {
    let db = s.db.lock().unwrap();
    let mut p = body;
    p.id = 1;
    p.updated = Utc::now();
    repo::upsert_profile(&db, &p)?;
    Ok(StatusCode::NO_CONTENT)
}
