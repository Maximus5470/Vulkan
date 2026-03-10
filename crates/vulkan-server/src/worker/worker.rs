use std::error::Error;

use vulkan_core::{Job, JobResult, docker::execute_job};

use crate::worker::Worker;

impl Worker {
    pub fn process_job(&self, job: Job) -> Result<JobResult, Box<dyn Error>> {
        println!("Worker {}:[{}] processing job {}", self.id, self.bias, job.job_id);
        execute_job(&job, &self.registry)
    }
}