//! recall-api —— axum 服务入口（骨架阶段）。
//!
//! 当前仅一个健康检查端点。后续 Task 在此 Router 上挂 cards/topics/stats。

use axum::{routing::get, Json, Router};
use serde_json::{json, Value};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(health));

    let addr = "127.0.0.1:7878";
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|e| panic!("绑定 {addr} 失败: {e}"));
    eprintln!("recall-api listening on http://{addr}");
    axum::serve(listener, app).await.unwrap();
}

/// 健康检查 —— 返回平台元信息。
async fn health() -> Json<Value> {
    Json(json!({
        "name": "recall",
        "status": "ok",
        "tagline": "headless learning data platform",
        "note": "AI calls the API; the platform has no AI of its own."
    }))
}
