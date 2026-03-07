use std::env;
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

use redis::Script;
use redis::{Commands, ErrorKind, RedisError, RedisResult};
use vulkan_core::models::{Priority, JobSubmission, Job};

const DEFAULT_HIGH_LIMIT: usize = 100;
const DEFAULT_MEDIUM_LIMIT: usize = 500;
const DEFAULT_LOW_LIMIT: usize = 1000;

const HIGH_QUEUE: &str = "vulkan:queue:high"; 
const MEDIUM_QUEUE: &str = "vulkan:queue:medium";
const LOW_QUEUE: &str = "vulkan:queue:low";
const JOBS_HASH: &str = "vulkan:jobs";

pub struct MLFQ{
    pub redis: redis::Connection,
    pub high_limit: usize,
    pub medium_limit: usize,
    pub low_limit: usize,
}

impl fmt::Debug for MLFQ {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MLFQ")
            .field("high_limit", &self.high_limit)
            .field("medium_limit", &self.medium_limit)
            .field("low_limit", &self.low_limit)
            .finish()
    }
}

fn current_time() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as f64
}

impl MLFQ {
    pub fn new(redis: redis::Connection) -> Self {
        Self {
            redis,
            high_limit: read_limit("HIGH_QUEUE_LIMIT", DEFAULT_HIGH_LIMIT),
            medium_limit: read_limit("MEDIUM_QUEUE_LIMIT", DEFAULT_MEDIUM_LIMIT),
            low_limit: read_limit("LOW_QUEUE_LIMIT", DEFAULT_LOW_LIMIT),
        }
    }

    fn determine_priority(submission_type: JobSubmission, testcase_count: usize) -> Priority {
        match submission_type {
            JobSubmission::Run => Priority::High,
            JobSubmission::Submit => {
                if testcase_count < 1000 {
                    Priority::Medium
                } else {
                    Priority::Low
                }
            }
        }
    }

    fn push_job_with_priority(&mut self, job_id: &str, priority: Priority) -> RedisResult<i64> {
        let script = Script::new(include_str!("scheduler_push.lua"));

    let time = current_time();

    let result = script
        .key(HIGH_QUEUE)
        .key(MEDIUM_QUEUE)
        .key(LOW_QUEUE)
        .arg(self.high_limit)
        .arg(self.medium_limit)
        .arg(self.low_limit)
        .arg(job_id)
        .arg(time)
        .arg(format!("{:?}", priority))
        .invoke(&mut self.redis)?;

    if result == 0 {
        return Err(RedisError::from((
            ErrorKind::ResponseError,
            "All queues full"
        )));
    }
    
    Ok(result)
    }

    pub fn push_job(&mut self, job: &Job) -> RedisResult<i64> {
        let job_id = job.job_id.to_string();
        let priority = Self::determine_priority(job.submission_type, job.testcases.len());

        let job_json = serde_json::to_string(job).map_err(|e| {
            RedisError::from((ErrorKind::IoError, "Failed to serialize job", e.to_string()))
        })?;
        self.redis.hset::<_, _, _, ()>(JOBS_HASH, &job_id, &job_json)?;

        self.push_job_with_priority(&job_id, priority)
    }

    pub fn fetch_job(&mut self, bias: Priority) -> RedisResult<Option<Job>> {
        let queues = match bias {
            Priority::High => [HIGH_QUEUE, MEDIUM_QUEUE, LOW_QUEUE],
            Priority::Medium => [MEDIUM_QUEUE, HIGH_QUEUE, LOW_QUEUE],
            Priority::Low => [LOW_QUEUE, MEDIUM_QUEUE, HIGH_QUEUE],
        };

        for queue in queues {
            let result: Vec<(String, f64)> = self.redis.zpopmin(queue, 1)?;
            if let Some((job_id, _)) = result.into_iter().next() {
                let job_json: String = self.redis.hget(JOBS_HASH, &job_id)?;
                let job: Job = serde_json::from_str(&job_json).map_err(|e| {
                    RedisError::from((ErrorKind::IoError, "Failed to deserialize job", e.to_string()))
                })?;
                return Ok(Some(job));
            }
        }

        Ok(None)
    }

    pub fn requeue_job(&mut self, job_id: &str, next_priority: Priority) -> RedisResult<i64> {
        self.push_job_with_priority(job_id, next_priority)
    }

    fn queue_len(&mut self, queue: &str) -> RedisResult<usize> {
        self.redis.zcard(queue)
    }
}

fn read_limit(key: &str, default: usize) -> usize {
    env::var(key)
    .ok()
    .and_then(|value| value.parse::<usize>().ok())
    .unwrap_or(default)
}