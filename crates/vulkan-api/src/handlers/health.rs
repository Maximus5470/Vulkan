use axum::{Json, extract::State};
use serde::Serialize;

use super::AppState;

#[derive(Serialize)]
pub struct Health {
    uptime_seconds: u64,
    redis_connected: bool,
}

pub async fn handle(State(state): State<AppState>) -> Json<Health> {
    let uptime_seconds = state.uptime
        .elapsed()
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let redis_connected = {
        let mut conn = state.redis.clone();
        redis::cmd("PING")
            .query_async::<String>(&mut conn)
            .await
            .is_ok()
    };

    Json(Health {
        uptime_seconds,
        redis_connected,
    })
}