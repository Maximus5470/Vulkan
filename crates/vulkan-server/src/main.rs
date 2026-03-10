mod scheduler;
use std::{env, sync::Arc};

use vulkan_core::registry::load_registry_from_file;

use crate::worker::WorkerPool;
mod worker;

fn main() {
    dotenvy::dotenv().ok();

    println!("Starting Vulkan Server...");
    let worker_count = env::var("WORKER_POOL_SIZE")
        .map_or(1, |value| value.parse::<usize>().unwrap_or(1));
    let registry = Arc::new(load_registry_from_file());
    let mut worker_pool = WorkerPool::new(worker_count, Arc::clone(&registry));
    println!("Worker pool initialized with {} workers.", worker_pool.workers.len());
    worker_pool.start();

    for handle in worker_pool.handles {
        if let Err(e) = handle.join() {
            eprintln!("Worker thread panicked: {:?}", e);
        }
    }
}