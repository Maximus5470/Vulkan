use dotenvy;
use std::{
    env,
    sync::Arc,
    thread,
};

use redis::Client;
use vulkan_core::{Priority, RuntimeRegistry};

use crate::{
    scheduler::Mlfq,
    worker::{Worker, WorkerPool, WorkerStatus},
};

impl WorkerPool {
    pub fn new(size: usize, registry: Arc<RuntimeRegistry>) -> Self {
        dotenvy::dotenv().ok();
        assert!(size > 0, "Worker pool size must be greater than 0");

        let high_count = env::var("HIGH_QUEUE_WORKER_COUNT")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(0);
        let medium_count = env::var("MEDIUM_QUEUE_WORKER_COUNT")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(size);
        let low_count = env::var("LOW_QUEUE_WORKER_COUNT")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(0);

        let total_from_split = high_count + medium_count + low_count;
        let (high_count, medium_count, low_count) = if total_from_split == 0 {
            // Backward-compatible fallback when queue-specific values are absent.
            (0, size, 0)
        } else {
            if total_from_split != size {
                eprintln!(
                    "WORKER_POOL_SIZE ({}) does not match HIGH/MEDIUM/LOW split ({}). Using split-defined total.",
                    size, total_from_split
                );
            }
            (high_count, medium_count, low_count)
        };

        let mut workers = Vec::with_capacity(high_count + medium_count + low_count);
        let mut id = 0usize;

        for _ in 0..high_count {
            workers.push(Worker {
                id,
                bias: Priority::High,
                status: WorkerStatus::Idle,
                registry: Arc::clone(&registry),
            });
            id += 1;
        }

        for _ in 0..medium_count {
            workers.push(Worker {
                id,
                bias: Priority::Medium,
                status: WorkerStatus::Idle,
                registry: Arc::clone(&registry),
            });
            id += 1;
        }

        for _ in 0..low_count {
            workers.push(Worker {
                id,
                bias: Priority::Low,
                status: WorkerStatus::Idle,
                registry: Arc::clone(&registry),
            });
            id += 1;
        }

        Self { workers, handles: Vec::new(), registry }
    }

    pub fn start(&mut self) {
        dotenvy::dotenv().ok();
        let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".into());
        let workers = std::mem::take(&mut self.workers);

        for mut worker in workers {
            let bias = worker.bias;

            let mut conn = Client::open(redis_url.clone())
                .unwrap()
                .get_connection()
                .unwrap();
            let scheduler = Mlfq::new();

            let handle = thread::spawn(move || {
                loop {
                    match scheduler.fetch_job(&mut conn, bias) {
                        Ok(Some(job)) => {
                            worker.status = WorkerStatus::Busy;
                            let result = worker.process_job(job);
                            match result {
                                Ok(job_result) => {
                                    scheduler.push_result(&mut conn, &job_result, &serde_json::to_string(&job_result).unwrap()).unwrap();
                                }
                                Err(e) => {
                                    eprintln!("Job processing error: {}", e);
                                }
                            }

                            worker.status = WorkerStatus::Idle;
                            
                        }
                        Ok(None) => {}
                        Err(e) => {
                            worker.status = WorkerStatus::Offline;
                            eprintln!("Worker error: {}", e);
                            break;
                        }
                    }
                }
            });

            self.handles.push(handle);
        }
    }
}