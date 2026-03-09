use axum::{extract::State};
use prometheus::{Encoder, IntGaugeVec, Opts, TextEncoder, register_int_gauge_vec};
use redis::AsyncCommands;

use crate::handlers::AppState;
use lazy_static::lazy_static;

lazy_static! {
    static ref QUEUE_LENGTH: IntGaugeVec = register_int_gauge_vec!(
        Opts::new("vulkan_queue_length", "Length of the job queues"),
        &["priority"]
    ).unwrap();
}

pub async fn handle(State(state): State<AppState>) -> String {
    let mut conn = state.redis.clone();

    let high_length: i64 = conn.llen("vulkan:queue:high").await.unwrap_or(0);
    let medium_length: i64 = conn.llen("vulkan:queue:medium").await.unwrap_or(0);
    let low_length: i64 = conn.llen("vulkan:queue:low").await.unwrap_or(0);

    QUEUE_LENGTH.with_label_values(&["high"]).set(high_length);
    QUEUE_LENGTH.with_label_values(&["medium"]).set(medium_length);
    QUEUE_LENGTH.with_label_values(&["low"]).set(low_length);

    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}