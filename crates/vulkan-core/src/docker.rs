use std::{
    collections::HashSet,
    env,
    error::Error,
    fs::{self, File},
    io::Write,
    path::PathBuf,
    process::Command,
    time::Instant,
};

use crate::{
    dockerfile_content::generic_dockerfile_content,
    models::{Job, JobResult, JobStatus, RuntimeRegistry, TestcaseResult},
};

// ───────────────────────────── Helpers ─────────────────────────────

fn validate_input(value: &str) -> Result<(), Box<dyn Error>> {
    if value.contains("..") || value.contains('/') || value.contains('\\') {
        return Err("Invalid path component".into());
    }
    Ok(())
}

fn image_name(language: &str, version: &str) -> String {
    format!("vulkan-{}:{}", language, version)
}

fn dockerfile_dir(language: &str, version: &str) -> PathBuf {
    PathBuf::from("dockerfiles").join(language).join(version)
}

fn ensure_dockerfile(
    language: &str,
    version: &str,
    dockerfile_contents: &str,
) -> Result<(), Box<dyn Error>> {
    validate_input(language)?;
    validate_input(version)?;

    let dir = dockerfile_dir(language, version);
    let dockerfile_path = dir.join("Dockerfile");

    fs::create_dir_all(&dir)?;

    if !dockerfile_path.exists() {
        let mut file = File::create(&dockerfile_path)?;
        file.write_all(dockerfile_contents.as_bytes())?;
    }

    Ok(())
}

fn image_exists(language: &str, version: &str) -> bool {
    let image = image_name(language, version);

    match Command::new("docker")
        .args(["image", "inspect", &image])
        .output()
    {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

fn build_image(language: &str, version: &str) -> Result<(), Box<dyn Error>> {
    validate_input(language)?;
    validate_input(version)?;

    let image = image_name(language, version);
    let dir = dockerfile_dir(language, version);

    if !dir.exists() {
        return Err(format!("Dockerfile directory missing: {}", dir.display()).into());
    }

    let dockerfile_relative = format!("dockerfiles/{}/{}/Dockerfile", language, version);
    let status = Command::new("docker")
        .args([
            "build",
            "-t",
            &image,
            "-f",
            &dockerfile_relative,
            "dockerfiles",
        ])
        .status()?;

    if !status.success() {
        return Err(format!("Failed to build Docker image: {}", image).into());
    }

    Ok(())
}

fn remove_image(language: &str, version: &str) -> Result<(), Box<dyn Error>> {
    let image = image_name(language, version);

    let output = Command::new("docker").args(["rmi", &image]).output()?;

    if !output.status.success() {
        return Err(format!("Failed to remove Docker image: {}", image).into());
    }

    Ok(())
}

fn list_images() -> Result<Vec<String>, Box<dyn Error>> {
    let output = Command::new("docker")
        .args(["images", "--format", "{{.Repository}}:{{.Tag}}"])
        .output()?;

    if !output.status.success() {
        return Err("Failed to list Docker images".into());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.lines().map(|line| line.to_string()).collect())
}

// ──────────────────────── Image Management ────────────────────────

/// Synchronize Docker images with the runtime registry.
///
/// - Build any missing images using the generic Dockerfile template
/// - Remove any `vulkan-*` images that are no longer in the registry
pub fn update_images(registry: &RuntimeRegistry) -> Result<(), Box<dyn Error>> {
    let existing_images = list_images()?;

    for runtime in &registry.runtimes {
        for version in &runtime.versions {
            if !image_exists(&runtime.language, version) {
                println!("Building Docker image for {} {}", runtime.language, version);

                // Use the docker_image from config to build a generic Dockerfile
                let dockerfile_content = generic_dockerfile_content(&runtime.docker_image);
                ensure_dockerfile(&runtime.language, version, &dockerfile_content)?;
                build_image(&runtime.language, version)?;
            }
        }
    }

    // Build a set of expected image names based on the registry
    let mut expected_images = HashSet::new();
    for runtime in &registry.runtimes {
        for version in &runtime.versions {
            let image = image_name(&runtime.language, version);
            expected_images.insert(image);
        }
    }

    // Remove images that are not in registry
    for existing_image in existing_images {
        if existing_image.starts_with("vulkan-") && !expected_images.contains(&existing_image) {
            println!("Removing Docker image: {}", existing_image);
            if let Some(lang_version) = existing_image.strip_prefix("vulkan-") {
                if let Some((language, version)) = lang_version.split_once(':') {
                    remove_image(language, version)?;
                    // Remove that image's dockerfile directory
                    let dir = dockerfile_dir(language, version);
                    if dir.exists() {
                        fs::remove_dir_all(dir)?;
                    }
                }
            }
        }
    }

    Ok(())
}

// ──────────────────────── Output Normalization ────────────────────

/// Normalize output for comparison:
/// - Trim leading/trailing whitespace
/// - Normalize line endings (\r\n → \n)
/// - Remove trailing empty lines
fn normalize_output(output: &str) -> String {
    output.replace("\r\n", "\n").trim().to_string()
}

// ──────────────────── Container Management ────────────────────────

/// Create and start a Docker container with the workspace mounted at /app.
/// Returns the container ID.
fn create_container(
    image: &str,
    workspace: &PathBuf,
    container_name: &str,
) -> Result<String, Box<dyn Error>> {
    let output = Command::new("docker")
        .args([
            "create",
            "--name",
            container_name,
            "-v",
            &format!("{}:/app", workspace.display()),
            "-w",
            "/app",
            image,
            "tail",
            "-f",
            "/dev/null", // keep container alive
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to create container: {}", stderr).into());
    }

    let container_id = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Start the container
    let start_output = Command::new("docker")
        .args(["start", &container_id])
        .output()?;

    if !start_output.status.success() {
        let stderr = String::from_utf8_lossy(&start_output.stderr);
        return Err(format!("Failed to start container: {}", stderr).into());
    }

    Ok(container_id)
}

/// Execute a command inside a running container.
/// Optionally pipe stdin input.
/// Returns (exit_code, stdout, stderr).
fn exec_in_container(
    container_id: &str,
    cmd: &[String],
    stdin_input: Option<&str>,
) -> Result<(i32, String, String), Box<dyn Error>> {
    let mut command = Command::new("docker");
    command.args(["exec", "-i", container_id]);
    command.args(cmd);

    if let Some(input) = stdin_input {
        use std::process::Stdio;
        command.stdin(Stdio::piped());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        let mut child = command.spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            let _ = stdin.write_all(input.as_bytes());
            // drop stdin to close it, signaling EOF
        }

        let output = child.wait_with_output()?;
        let exit_code = output.status.code().unwrap_or(-1);
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

        Ok((exit_code, stdout, stderr))
    } else {
        let output = command.output()?;
        let exit_code = output.status.code().unwrap_or(-1);
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

        Ok((exit_code, stdout, stderr))
    }
}

/// Destroy (force-remove) a Docker container.
fn destroy_container(container_id: &str) {
    let _ = Command::new("docker")
        .args(["rm", "-f", container_id])
        .output();
}

// ──────────────────────── Job Execution ──────────────────────────

/// Execute a job using the Piston-style runtime pipeline.
///
/// The executor is **completely language-agnostic**. All language behavior
/// (source file name, compile command, run command) comes from the
/// runtime configuration loaded from `runtime.json`.
///
/// ## Pipeline
/// 1. Validate runtime (language + version) against the registry
/// 2. Create a temporary workspace
/// 3. Write user code to the configured `source_file`
/// 4. Create and start a Docker container with workspace mounted at `/app`
/// 5. If `compile_cmd` exists, execute it inside the container
/// 6. For each testcase, execute `run_cmd` with stdin from testcase input
/// 7. Capture stdout/stderr and compare output (normalized)
/// 8. Destroy the container
/// 9. Return the final `JobResult`
pub fn execute_job(job: &Job, registry: &RuntimeRegistry) -> Result<JobResult, Box<dyn Error>> {
    let start = Instant::now();

    // ── Step 1: Validate runtime ──
    let runtime = registry
        .validate_runtime(&job.language, &job.version)
        .map_err(|e| -> Box<dyn Error> { e.into() })?;

    let image = image_name(&job.language, &job.version);

    if !image_exists(&job.language, &job.version) {
        return Err(format!(
            "Docker image not found for {} {}. Run `vulkan update_images` first.",
            job.language, job.version
        )
        .into());
    }

    // ── Step 2: Create temporary workspace ──
    let workspace = env::temp_dir()
        .join("vulkan_jobs")
        .join(job.job_id.to_string());
    fs::create_dir_all(&workspace)?;

    // ── Step 3: Write user code to source_file ──
    let file_path = workspace.join(&runtime.source_file);
    let normalized_code = job.code.replace("\r\n", "\n");
    fs::write(&file_path, &normalized_code)?;

    // ── Step 4: Create and start Docker container ──
    let container_name = format!("vulkan-job-{}", job.job_id);
    let container_id = match create_container(&image, &workspace, &container_name) {
        Ok(id) => id,
        Err(e) => {
            let _ = fs::remove_dir_all(&workspace);
            return Err(e);
        }
    };

    // Ensure cleanup on any failure path
    let result = execute_in_container(job, runtime, &container_id);

    // ── Step 10: Destroy the container ──
    destroy_container(&container_id);

    // Clean up workspace
    let _ = fs::remove_dir_all(&workspace);

    let duration = start.elapsed().as_millis() as u64;

    match result {
        Ok((testcase_results, global_stderr)) => {
            let all_passed =
                !testcase_results.is_empty() && testcase_results.iter().all(|tc| tc.passed);
            let status = if all_passed {
                JobStatus::Success
            } else {
                JobStatus::Failed
            };

            Ok(JobResult {
                job_id: job.job_id,
                status,
                execution_time_ms: duration,
                testcases: testcase_results,
                stderr: global_stderr,
            })
        }
        Err(e) => Ok(JobResult {
            job_id: job.job_id,
            status: JobStatus::Failed,
            execution_time_ms: duration,
            testcases: vec![],
            stderr: e.to_string(),
        }),
    }
}

/// Inner execution logic that runs inside the container.
/// Separated so that container cleanup always happens in `execute_job`.
fn execute_in_container(
    job: &Job,
    runtime: &crate::models::LanguageConfig,
    container_id: &str,
) -> Result<(Vec<TestcaseResult>, String), Box<dyn Error>> {
    let mut global_stderr = String::new();

    // ── Step 7: Compile if compile_cmd exists ──
    if let Some(compile_cmd) = &runtime.compile_cmd {
        let (exit_code, _stdout, stderr) = exec_in_container(container_id, compile_cmd, None)?;

        if exit_code != 0 {
            return Err(format!(
                "Compilation failed (exit code {}):\n{}",
                exit_code,
                stderr.trim()
            )
            .into());
        }
    }

    // ── Step 8: Execute each testcase ──
    let mut testcase_results = Vec::new();

    for testcase in &job.testcases {
        let tc_start = Instant::now();

        let stdin_input = if testcase.input.is_empty() {
            None
        } else {
            Some(testcase.input.as_str())
        };

        let (exit_code, stdout, stderr) =
            exec_in_container(container_id, &runtime.run_cmd, stdin_input)?;

        let tc_duration = tc_start.elapsed().as_millis() as u64;

        if !stderr.is_empty() && global_stderr.is_empty() {
            global_stderr = stderr.clone();
        }

        // ── Step 9: Compare output with expected (normalized) ──
        let normalized_actual = normalize_output(&stdout);
        let normalized_expected = normalize_output(&testcase.expected_output);
        let passed = exit_code == 0 && normalized_actual == normalized_expected;

        testcase_results.push(TestcaseResult {
            testcase_id: testcase.testcase_id.clone(),
            input: testcase.input.clone(),
            expected_output: testcase.expected_output.clone(),
            actual_output: stdout.clone(),
            passed,
            exec_time_ms: tc_duration,
        });
    }

    Ok((testcase_results, global_stderr))
}
