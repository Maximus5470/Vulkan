use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LanguageConfig {
    pub language: String,
    pub versions: Vec<String>,
    pub source_file: String,
    pub compile_cmd: Option<Vec<String>>,
    pub run_cmd: Vec<String>,
    /// Docker base image to use for this language (e.g., "python:3.11-slim", "eclipse-temurin:25-jdk")
    pub docker_image: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeRegistry {
    pub runtimes: Vec<LanguageConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestCase {
    pub testcase_id: String,
    pub input: String,
    pub expected_output: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestcaseResult {
    pub testcase_id: String,
    pub input: String,
    pub expected_output: String,
    pub actual_output: String,
    pub passed: bool,
    pub exec_time_ms: u64,
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
    pub testcases: Vec<TestcaseResult>,
    pub stderr: String,
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
    pub version: String,
    pub code: String,
    pub testcases: Vec<TestCase>,
    pub attempts: u32,
    pub created_at: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum JobStatus {
    Success,
    Failed,
}
