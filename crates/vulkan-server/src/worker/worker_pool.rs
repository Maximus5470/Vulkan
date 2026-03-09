use dotenvy;
use std::{
    env,
    sync::{Arc, Mutex},
    thread,
};

use redis::{Client, Connection};
use vulkan_core::Priority;

use crate::{
    scheduler::Mlfq,
    worker::{Worker, WorkerPool, WorkerStatus},
};

impl WorkerPool {
    pub fn new(size: usize) -> Self {
        dotenvy::dotenv().ok();
        assert!(size > 0, "Worker pool size must be greater than 0");
        let mut workers: Vec<Arc<Mutex<Worker>>> = Vec::with_capacity(size);
        for i in 0..size {
            workers.push(Arc::new(Mutex::new(Worker {
                id: i,
                bias: Priority::Medium,
                status: WorkerStatus::Idle,
            })));
        }
        Self { workers, handles: Vec::new() }
    }

    pub fn start(&mut self) {
        dotenvy::dotenv().ok();
        let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".into());

        for worker_arc in &self.workers {
            let worker = Arc::clone(worker_arc);
            let bias = worker.lock().unwrap().bias;
            let redis_url = redis_url.clone();
            let scheduler = Mlfq::new();

            let handle = thread::spawn(move || {
                // Retry connecting until successful
                let make_conn = |url: &str| -> Connection {
                    loop {
                        match Client::open(url).and_then(|c| c.get_connection()) {
                            Ok(conn) => {
                                eprintln!("Worker connected to Redis");
                                return conn;
                            }
                            Err(e) => {
                                eprintln!("Failed to connect to Redis, retrying in 2s: {}", e);
                            }
                        }
                    }
                };

                let mut conn = make_conn(&redis_url);

                loop {
                    match scheduler.fetch_job(&mut conn, bias) {
                        Ok(Some(job)) => {
                            worker.lock().unwrap().status = WorkerStatus::Busy;

                            let result = {
                                let w = worker.lock().unwrap();
                                w.process_job(job)
                            };

                            match result {
                                Ok(job_result) => {
                                    let payload = match serde_json::to_string(&job_result) {
                                        Ok(p) => p,
                                        Err(e) => {
                                            eprintln!("Serialization error: {}", e);
                                            worker.lock().unwrap().status = WorkerStatus::Idle;
                                            continue;
                                        }
                                    };
                                    if let Err(e) = scheduler.push_result(&mut conn, &job_result, &payload) {
                                        eprintln!("Failed to push result: {}", e);
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Job processing error: {}", e);
                                }
                            }

                            worker.lock().unwrap().status = WorkerStatus::Idle;
                        }
                        Ok(None) => {
                            thread::sleep(std::time::Duration::from_millis(1));
                        }
                        Err(e) => {
                            eprintln!("Worker error: {}, reconnecting...", e);
                            worker.lock().unwrap().status = WorkerStatus::Offline;
                            thread::sleep(std::time::Duration::from_secs(2));
                            conn = make_conn(&redis_url);  // reconnect
                            worker.lock().unwrap().status = WorkerStatus::Idle;
                        }
                    }
                }
            });

            self.handles.push(handle);
        }
    }
}