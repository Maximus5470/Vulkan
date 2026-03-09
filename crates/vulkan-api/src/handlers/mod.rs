use std::{sync::Arc, time::SystemTime};

use redis::aio::ConnectionManager;
use vulkan_core::RuntimeRegistry;

pub mod job;
pub mod health;
pub mod execute;
pub mod runtimes;
pub mod metrics;

#[derive(Clone)]
pub struct AppState{
    pub uptime: SystemTime,
    pub redis: ConnectionManager,
    pub runtimes: Arc<RuntimeRegistry>
}