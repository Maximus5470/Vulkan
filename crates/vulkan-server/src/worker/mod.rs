use std::{sync::{Arc, Mutex}, thread};
use vulkan_core::{Priority, RuntimeRegistry};

pub mod worker_pool;
pub mod worker;

#[derive(Debug, Clone, Copy)]
pub enum WorkerStatus{
    Idle,
    Busy,
    Offline,
}

pub struct WorkerPool{
    pub workers: Vec<Arc<Mutex<Worker>>>,
    pub handles: Vec<thread::JoinHandle<()>>,
    pub registry: Arc<RuntimeRegistry>
}

#[derive(Debug)]
pub struct Worker{
    pub id: usize,
    pub bias: Priority,
    pub status: WorkerStatus,
    pub registry: Arc<RuntimeRegistry>
}