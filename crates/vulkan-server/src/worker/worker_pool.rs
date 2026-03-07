use dotenvy;
use std::{
    env,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
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
        
        let scheduler = Arc::new(Mutex::new(MLFQ::new(
            Client::open(
                env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".into()),
            )
            .unwrap()
            .get_connection()
            .unwrap()
        )));

        let mut workers = Vec::with_capacity(size);
        for i in 0..size {
            workers.push(Arc::new(Mutex::new(Worker {
                id: i,
                bias: Priority::Medium,
                status: WorkerStatus::Idle,
            })));
        }
        Self { workers, handles: Vec::new(), scheduler }
    }

    pub fn start(&mut self) {
        dotenvy::dotenv().ok();
        let scheduler_clone = Arc::clone(&self.scheduler);
        
        for worker_arc in &self.workers {
            let scheduler = Arc::clone(&scheduler_clone);
            let worker = Arc::clone(worker_arc);
            let bias = worker.lock().unwrap().bias;

            let handle = thread::spawn(move || {
                loop {
                    let mut sched = scheduler.lock().unwrap();
                    if let Some(job) = sched.fetch_job(bias).unwrap() {
                        drop(sched);
                        worker.lock().unwrap().process_job(job);
                    } else {
                        drop(sched);
                        thread::sleep(Duration::from_millis(500));
                    }
                }
            });

            self.handles.push(handle);
        }
    }
}