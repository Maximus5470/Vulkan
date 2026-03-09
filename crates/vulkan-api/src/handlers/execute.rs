use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
};
use redis::Client;
use vulkan_core::{Job, SubmitJobRequest};

use crate::handlers::AppState;

pub async fn handle(
    State(state): State<AppState>,
    Json(payload): Json<SubmitJobRequest>,
) -> impl IntoResponse {
    let job_id = uuid::Uuid::new_v4().to_string();
    let parsed_job_id = match uuid::Uuid::parse_str(&job_id) {
        Ok(id) => id,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to create job id: {}", e),
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

    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".into());
    let redis_conn = match Client::open(redis_url) {
        Ok(client) => client,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to create Redis client: {}", e),
            )
                .into_response();
        }
    };

    let mut conn = match redis_conn.get_connection() {
        Ok(connection) => connection,
        Err(e) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                format!("Failed to get Redis connection: {}", e),
            )
                .into_response();
        }
    };

    match state.scheduler.push_job(&mut conn, &job) {
        Ok(_) => (StatusCode::ACCEPTED, format!("Job submitted with ID: {}", job_id)).into_response(),
        Err(e) => (
            StatusCode::SERVICE_UNAVAILABLE,
            format!("Failed to enqueue job {}: {}", job_id, e),
        )
            .into_response(),
    }
}