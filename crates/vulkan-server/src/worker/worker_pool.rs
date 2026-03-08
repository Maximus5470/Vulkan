use dotenvy;
use std::{
    env,
    sync::{Arc, Mutex},
    thread,
};

use redis::Client;
use vulkan_core::models::Priority;

use crate::{
    scheduler::MLFQ,
    worker::{Worker, WorkerPool, WorkerStatus},
};

impl WorkerPool {
    pub fn new(size: usize) -> Self {
        dotenvy::dotenv().ok();

        let mut workers = Vec::with_capacity(size);
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

            let conn = Client::open(redis_url.clone())
                .unwrap()
                .get_connection()
                .unwrap();
            let mut scheduler = MLFQ::new(conn);

            let handle = thread::spawn(move || {
                loop {
                    match scheduler.fetch_job(bias) {
                        Ok(Some(job)) => {
                            let mut w = worker.lock().unwrap();
                            w.status = WorkerStatus::Busy;
                            let result= w.process_job(job);
                            match result {
                                Ok(job_result) => {
                                    scheduler.push_result(&job_result, &serde_json::to_string(&job_result).unwrap()).unwrap();
                                }
                                Err(e) => {
                                    eprintln!("Job processing error: {}", e);
                                }
                            }

                            w.status = WorkerStatus::Idle;
                            
                        }
                        Ok(None) => {}
                        Err(e) => {
                            worker.lock().unwrap().status = WorkerStatus::Offline;
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