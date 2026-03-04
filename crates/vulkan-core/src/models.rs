use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LanguageConfig {
    pub language: String,
    pub versions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeRegistry {
    pub runtimes: Vec<LanguageConfig>,
}

pub struct LanguageTemplate {
    pub base_image: String,
    pub workdir: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestCase {
    pub testcase_id: String,
    pub input: String,
    pub expected_output: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubmitJobRequest {
    pub language: String,
    pub code: String,
    pub testcases: Vec<TestCase>,
    pub user_timeout_ms: u64,
    pub memory_limit_mb: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JobResult {
    pub job_id: Uuid,
    pub status: JobStatus,
    pub execution_time_ms: u64,
    pub memory_used_mb: u64,
    pub failure_reason: Option<JobFailureReason>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Eq, PartialEq)]
pub enum Priority {
    High,
    Medium,
    Low,
}

#[derive(Debug)]
pub struct Job {
    pub job_id: Uuid,
    pub user_id: String,

    pub language: String,
    pub code: String,
    pub testcases: Vec<TestCase>,
    pub attempts: u32,

    pub created_at: u64,

    // Constraints
    pub user_timeout_ms: u64,
    pub memory_limit_mb: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum JobStatus {
    Success,
    Failed,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum JobFailureReason {
    TimeLimitExceeded,
    MemoryLimitExceeded,
    CompilationError(String),
    RuntimeError(String),
    WrongAnswer { testcase_id: String },
    InternalError(String),
}
