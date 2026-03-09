use axum::{Router, routing::{get, post}};

use crate::handlers::{AppState, execute, health, job, metrics, runtimes};

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health::handle))
        .route("/execute", post(execute::handle))
        .route("/job", get(job::handle))
        .route("/runtimes", get(runtimes::handle))
        .route("/metrics", get(metrics::handle))
        .with_state(state)
}