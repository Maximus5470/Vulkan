use std::env;
use std::fmt;

use redis::Connection;
use redis::Script;
use redis::ServerErrorKind;
use redis::{Commands, ErrorKind, RedisError, RedisResult};
use vulkan_core::{Job, JobResult, JobSubmission, Priority};

const DEFAULT_HIGH_LIMIT: usize = 100;
const DEFAULT_MEDIUM_LIMIT: usize = 500;
const DEFAULT_LOW_LIMIT: usize = 1000;

const HIGH_QUEUE: &str = "vulkan:queue:high";
const MEDIUM_QUEUE: &str = "vulkan:queue:medium";
const LOW_QUEUE: &str = "vulkan:queue:low";
const JOBS_HASH: &str = "vulkan:jobs";
const RESULTS_HASH: &str = "vulkan:results";

pub struct Mlq {
    pub high_limit: usize,
    pub medium_limit: usize,
    pub low_limit: usize,
}

impl fmt::Debug for Mlq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MLQ")
            .field("high_limit", &self.high_limit)
            .field("medium_limit", &self.medium_limit)
            .field("low_limit", &self.low_limit)
            .finish()
    }
}

impl Mlq {
    pub fn new() -> Self {
        Self {
            high_limit: read_limit("HIGH_QUEUE_LIMIT", DEFAULT_HIGH_LIMIT),
            medium_limit: read_limit("MEDIUM_QUEUE_LIMIT", DEFAULT_MEDIUM_LIMIT),
            low_limit: read_limit("LOW_QUEUE_LIMIT", DEFAULT_LOW_LIMIT),
        }
    }

       #[allow(dead_code)]
    fn determine_priority(submission_type: JobSubmission, testcase_count: usize) -> Priority {
        let testcase_limit = env::var("TESTCASE_COUNT_LIMIT")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(1000);
        match submission_type {
            JobSubmission::Run => Priority::High,
            JobSubmission::Submit => {
                if testcase_count < testcase_limit {
                    Priority::Medium
                } else {
                    Priority::Low
                }
            }
        }
    }
    #[allow(dead_code)]
    fn push_job_with_priority(&self, conn: &mut Connection, job_id: &str, job_json: &str, priority: Priority) -> RedisResult<i64> {
        let script = Script::new(include_str!("scheduler_push.lua"));

        let result = script
            .key(HIGH_QUEUE)
            .key(MEDIUM_QUEUE)
            .key(LOW_QUEUE)
            .key(JOBS_HASH)
            .arg(self.high_limit)
            .arg(self.medium_limit)
            .arg(self.low_limit)
            .arg(job_id)
            .arg(format!("{:?}", priority))
            .arg(job_json)
            .invoke(conn)?;

        if result == 0 {
            return Err(RedisError::from((
                ErrorKind::Server(ServerErrorKind::ResponseError),
                "All queues full",
            )));
        }

        Ok(result)
    }

    #[allow(dead_code)]
    pub fn push_job(&self, conn: &mut Connection, job: &Job) -> RedisResult<i64> {
        let job_id = job.job_id.to_string();
        let priority = Self::determine_priority(job.submission_type, job.testcases.len());

        let job_json = serde_json::to_string(job).map_err(|e| {
            RedisError::from((ErrorKind::Server(ServerErrorKind::ResponseError), "Failed to serialize job", e.to_string()))
        })?;

        // Atomically validate rate limits, push to queue, and store job data via Lua script
        self.push_job_with_priority(conn, &job_id, &job_json, priority)
    }

    pub fn fetch_job(&self, conn: &mut Connection, bias: Priority) -> RedisResult<Option<Job>> {
        let queues = match bias {
            Priority::High => [HIGH_QUEUE, MEDIUM_QUEUE, LOW_QUEUE],
            Priority::Medium => [MEDIUM_QUEUE, HIGH_QUEUE, LOW_QUEUE],
            Priority::Low => [LOW_QUEUE, MEDIUM_QUEUE, HIGH_QUEUE],
        };

        let result: Option<(String, String)> = redis::cmd("BRPOP")
            .arg(queues[0])
            .arg(queues[1])
            .arg(queues[2])
            .arg(0)
            .query(conn)?;

        if let Some((_, job_id)) = result {
            // Handle stale queue entries (orphaned IDs with no matching job data)
            let job_json: Option<String> = conn.hget(JOBS_HASH, &job_id)?;
            
            match job_json {
                Some(json) => {
                    let job: Job = serde_json::from_str(&json).map_err(|e| {
                        RedisError::from((
                            ErrorKind::Server(ServerErrorKind::ResponseError),
                            "Failed to deserialize job",
                            e.to_string(),
                        ))
                    })?;
                    return Ok(Some(job));
                }
                None => {
                    // Dead-letter: Log stale job ID and continue to next job
                    eprintln!("WARNING: Orphaned job ID in queue (no JOBS_HASH entry): {}. Skipping.", job_id);
                    // TODO: Optionally push to dead-letter queue for investigation
                    return Ok(None);
                }
            }
        }

        Ok(None)
    }

    pub fn push_result(&self, conn: &mut Connection, job: &JobResult, result: &str) -> RedisResult<()> {
        let key = format!("{}:{}", RESULTS_HASH, job.job_id);
        conn.set_ex(key, result, 300)
    }

   #[allow(dead_code)]
    pub fn get_result(&self, conn: &mut Connection, job_id: &str) -> RedisResult<Option<JobResult>> {
        let key = format!("{}:{}", RESULTS_HASH, job_id);
        let result_json: Option<String> = conn.get(key)?;
        if let Some(json) = result_json {
            let result: JobResult = serde_json::from_str(&json).map_err(|e| {
                RedisError::from((
                    ErrorKind::Server(ServerErrorKind::ResponseError),
                    "Failed to deserialize job result",
                    e.to_string(),
                ))
            })?;
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }
}

fn read_limit(key: &str, default: usize) -> usize {
    env::var(key)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(default)
}
