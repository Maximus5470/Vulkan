use std::error::Error;

use vulkan_core::{Job, JobResult, docker::execute_job, registry::load_registry_from_file};

use crate::worker::Worker;

impl Worker {
    pub fn process_job(&self, job: Job) -> Result<JobResult, Box<dyn Error>> {
        let registry = load_registry_from_file();
        println!("Worker {} processing job {}", self.id, job.job_id);
        execute_job(&job, &registry)
    }
}