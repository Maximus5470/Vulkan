use axum::{Json, extract::{Path, State}};
use redis::Client;
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
            stderr: "Invalid job ID format".to_string(),
            execution_time_ms: 0,
            testcases: vec![],
        }),
    };

    let job_id_str = job_id.to_string();

    let client = match Client::open(std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".into())) {
        Ok(c) => c,
        Err(e) => {
            return Json(JobResult {
                job_id,
                status: JobStatus::Failed,
                stderr: format!("Failed to create Redis client: {}", e),
                execution_time_ms: 0,
                testcases: vec![],
            });
        }
    };
    
    let mut conn = match client.get_connection() {
        Ok(c) => c,
        Err(e) => {
            return Json(JobResult {
                job_id,
                status: JobStatus::Failed,
                stderr: format!("Failed to get Redis connection: {}", e),
                execution_time_ms: 0,
                testcases: vec![],
            });
        }
    };

    let result = state.scheduler.get_result(&mut conn, job_id_str.as_str()).ok().flatten();

    match result {
        Some(res) => Json(res),
        None => Json(JobResult {
            job_id,
            status: JobStatus::Failed,
            stderr: "Job ID not found".to_string(),
            execution_time_ms: 0,
            testcases: vec![],
        }),
    }
}