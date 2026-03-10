use std::{sync::Arc, time::SystemTime};

use redis::aio::ConnectionManager;
use vulkan_core::RuntimeRegistry;
use vulkan_server::Mlfq;

pub mod job;
pub mod health;
pub mod execute;
pub mod runtimes;
pub mod metrics;

pub type RedisPool = r2d2::Pool<redis::Client>;

#[derive(Clone)]
pub struct AppState{
    pub uptime: SystemTime,
    pub redis: ConnectionManager,
    pub redis_pool: RedisPool,
    pub runtimes: Arc<RuntimeRegistry>,
    pub scheduler: Arc<Mlfq>,
}