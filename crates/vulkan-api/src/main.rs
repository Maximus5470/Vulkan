use std::{env, sync::Arc, time::SystemTime};

use axum::Router;
use redis::{Client, aio::ConnectionManager};
use tokio::net::TcpListener;
use vulkan_core::registry::load_registry_from_file;
use vulkan_server::Mlfq;
use dotenv;

use crate::handlers::AppState;

mod handlers;
mod routes;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost".into());
    let redis_conn = Client::open(redis_url).expect("Failed to create Redis client");
    let state = AppState {
        uptime: SystemTime::now(),
        redis: ConnectionManager::new(redis_conn).await.expect("Failed to create Redis connection manager"),
        runtimes: Arc::new(load_registry_from_file()),
        scheduler: Arc::new(Mlfq::new()),
    };

    let app = Router::new()
        .merge(routes::routes(state));

    let port = env::var("PORT").unwrap_or_else(|_| "8080".into());
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();
    println!("Starting server on port {}", port);
    axum::serve(listener, app).await.unwrap();
}