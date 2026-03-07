mod worker;
mod scheduler;
use std::env;

use crate::worker::WorkerPool;

fn main() {
    dotenvy::dotenv().ok();

    println!("Starting Vulkan Server...");
    let worker_count = env::var("WORKER_POOL_SIZE")
        .map_or(1, |value| value.parse::<usize>().unwrap_or(1));
    let mut worker_pool = WorkerPool::new(worker_count);
    println!("Worker pool initialized with {} workers.", worker_pool.workers.len());
    worker_pool.start();
}