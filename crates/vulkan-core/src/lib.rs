pub mod docker;
pub mod dockerfile_content;
pub mod registry;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LanguageConfig {
    pub language: String,
    pub versions: Vec<String>,
    pub source_file: String,
    pub compile_cmd: Option<Vec<String>>,
    pub run_cmd: Vec<String>,
    pub docker_image: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeRegistry {
    pub runtimes: Vec<LanguageConfig>,
}

impl RuntimeRegistry {
    pub fn new() -> Self {
        RuntimeRegistry {
            runtimes: Vec::new(),
        }
    }

    pub fn add_runtime(&mut self, language_config: LanguageConfig) {
        self.runtimes.push(language_config);
    }

    pub fn remove_runtime(&mut self, language: &str) {
        self.runtimes.retain(|config| config.language != language);
    }

    pub fn list_runtimes(&self) -> &Vec<LanguageConfig> {
        &self.runtimes
    }

    pub fn find_runtime(&self, language: &str) -> Option<&LanguageConfig> {
        self.runtimes
            .iter()
            .find(|r| r.language.eq_ignore_ascii_case(language))
    }

    // Checks if a language + version combination exists in the registry
    pub fn validate_runtime(
        &self,
        language: &str,
        version: &str,
    ) -> Result<&LanguageConfig, String> {
        let runtime = self
            .find_runtime(language)
            .ok_or_else(|| format!("Language '{}' not found in registry", language))?;

        if !runtime.versions.iter().any(|v| v == version) {
            return Err(format!(
                "Version '{}' not found for language '{}'. Available versions: {:?}",
                version, language, runtime.versions
            ));
        }

        Ok(runtime)
    }
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
    pub version: String,
    pub code: String,
    pub submission_type: JobSubmission,
    pub testcases: Vec<TestCase>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JobResult {
    pub job_id: Uuid,
    pub status: JobStatus,
    pub stderr: String,
    pub execution_time_ms: u64,
    pub testcases: Vec<TestcaseResult>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Eq, PartialEq)]
pub enum Priority {
    High,
    Medium,
    Low,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Job {
    pub job_id: Uuid,
    pub language: String,
    pub version: String,
    pub code: String,
    pub submission_type: JobSubmission,
    pub testcases: Vec<TestCase>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum JobStatus {
    Success,
    Failed,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq)]
pub enum JobSubmission{
    Run,
    Submit
}