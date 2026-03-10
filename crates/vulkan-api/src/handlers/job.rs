use axum::{Json, extract::{Path, State}};
use redis::{ErrorKind, RedisError, ServerErrorKind};
use tokio::task;
use uuid::Uuid;
use vulkan_core::{JobResult, JobStatus};

use crate::handlers::AppState;

pub async fn handle(
    State(state): State<AppState>,
    Path(job_id): Path<String>)
    -> Json<JobResult>
{
    let job_id = match Uuid::parse_str(&job_id) {
        Ok(id) => id,
        Err(_) => return Json(JobResult {
            job_id: Uuid::nil(),
            status: JobStatus::Failed,
            stdout: None,
            stderr: "Invalid job ID format".to_string(),
            execution_time_ms: 0,
            testcases: vec![],
        }),
    };

    let job_id_str = job_id.to_string();
    let pool = state.redis_pool.clone();
    let scheduler = state.scheduler.clone();

    let result = task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| {
            RedisError::from((ErrorKind::Server(ServerErrorKind::ResponseError), "Pool error", e.to_string()))
        })?;
        scheduler.get_result(&mut conn, job_id_str.as_str())
    })
    .await;

    match result {
        Ok(Ok(Some(res))) => Json(res),
        Ok(Ok(None)) => Json(JobResult {
            job_id,
            status: JobStatus::Failed,
            stdout: None,
            stderr: "Job ID not found".to_string(),
            execution_time_ms: 0,
            testcases: vec![],
        }),
        Ok(Err(e)) => Json(JobResult {
            job_id,
            status: JobStatus::Failed,
            stdout: None,
            stderr: format!("Redis error: {}", e),
            execution_time_ms: 0,
            testcases: vec![],
        }),
        Err(e) => Json(JobResult {
            job_id,
            status: JobStatus::Failed,
            stdout: None,
            stderr: format!("Task error: {}", e),
            execution_time_ms: 0,
            testcases: vec![],
        }),
    }
}