pub mod worker;
pub mod worker_pool;

use std::{sync::{Arc, Mutex}, thread};
use vulkan_core::Priority;

#[derive(Debug, Clone, Copy)]
pub enum WorkerStatus{
    Idle,
    Busy,
    Offline,
}

pub struct WorkerPool{
    pub workers: Vec<Arc<Mutex<Worker>>>,
    pub handles: Vec<thread::JoinHandle<()>>,
}

#[derive(Debug)]
pub struct Worker{
    pub id: usize,
    pub bias: Priority,
    pub status: WorkerStatus,
}