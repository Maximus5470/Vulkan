use std::time::Instant;

use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
};
use redis::{ErrorKind, ServerErrorKind};
use tokio::task;
use vulkan_core::{Job, SubmitJobRequest};

use crate::handlers::AppState;

pub async fn handle(
    State(state): State<AppState>,
    Json(payload): Json<SubmitJobRequest>,
) -> impl IntoResponse {
    let start = Instant::now();
    let job_id = uuid::Uuid::new_v4().to_string();
    let parsed_job_id = match uuid::Uuid::parse_str(&job_id) {
        Ok(id) => id,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to create job id: {} ({}ms)", e, start.elapsed().as_millis()),
            )
                .into_response();
        }
    };

    let job = Job {
        job_id: parsed_job_id,
        code: payload.code,
        language: payload.language,
        version: payload.version,
        submission_type: payload.submission_type,
        testcases: payload.testcases,
    };

    let pool = state.redis_pool.clone();
    let scheduler = state.scheduler.clone();

    match task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| {
            redis::RedisError::from((ErrorKind::Server(ServerErrorKind::ResponseError), "Pool error", e.to_string()))
        })?;
        scheduler.push_job(&mut conn, &job)
    })
    .await
    {
        Ok(Ok(_)) => (
            StatusCode::ACCEPTED,
            format!("Job submitted with ID: {} ({}ms)", job_id, start.elapsed().as_millis()),
        )
            .into_response(),
        Ok(Err(e)) => (
            StatusCode::SERVICE_UNAVAILABLE,
            format!("Failed to enqueue job {}: {} ({}ms)", job_id, e, start.elapsed().as_millis()),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Task panicked: {} ({}ms)", e, start.elapsed().as_millis()),
        )
            .into_response(),
    }
}