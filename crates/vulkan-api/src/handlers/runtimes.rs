use axum::{Json, extract::State};
use serde::Serialize;

use crate::handlers::AppState;

#[derive(Serialize)]
pub struct RuntimeView {
    language: String,
    version: Vec<String>,
}

pub async fn handle(State(state): State<AppState>) -> Json<Vec<RuntimeView>> {
    let runtimes = state.runtimes.list_runtimes();
    let view = runtimes.into_iter().map(|r| RuntimeView {
        language: r.language.clone(),
        version: r.versions.clone(),
    }).collect();
    Json(view)
}