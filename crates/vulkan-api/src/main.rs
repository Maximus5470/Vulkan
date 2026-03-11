use std::{env, sync::Arc, time::SystemTime};

use axum::Router;
use r2d2;
use redis::{Client, aio::ConnectionManager};
use tokio::net::TcpListener;
use vulkan_core::registry::load_registry_from_file;
use vulkan_server::Mlq;
use dotenv;

use crate::handlers::AppState;

mod handlers;
mod routes;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".into());
    let redis_conn = Client::open(redis_url).expect("Failed to create Redis client");
    let redis_pool = r2d2::Pool::builder()
        .max_size(10)
        .build_unchecked(redis_conn.clone());
    let state = AppState {
        uptime: SystemTime::now(),
        redis: ConnectionManager::new(redis_conn).await.expect("Failed to create Redis connection manager"),
        redis_pool,
        runtimes: Arc::new(load_registry_from_file()),
        scheduler: Arc::new(Mlq::new()),
    };

    let app = Router::new()
        .merge(routes::routes(state));

    let port = env::var("PORT").unwrap_or_else(|_| "8000".into());
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();
    println!("Starting server on port {}", port);
    axum::serve(listener, app).await.unwrap();
}