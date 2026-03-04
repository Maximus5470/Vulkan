use std::{fs, path::Path};

use vulkan_core::{
    RuntimeRegistry,
    docker::execute_job,
    models::{Job, TestCase},
};

const RUNTIME_CONFIG_PATH: &str = "crates/config/runtime.json";

fn load_registry_from_file() -> RuntimeRegistry {
    let config_path = Path::new(RUNTIME_CONFIG_PATH);

    if !config_path.exists() {
        eprintln!(
            "Warning: {} not found, using empty registry",
            RUNTIME_CONFIG_PATH
        );
        return RuntimeRegistry::new();
    }

    let content = fs::read_to_string(config_path).expect("Failed to read runtime.json");
    if content.trim().is_empty() {
        return RuntimeRegistry::new();
    }

    let runtimes = serde_json::from_str(&content).expect("Failed to parse runtime.json");
    RuntimeRegistry { runtimes }
}

fn main() {
    // Load the registry dynamically from configuration
    let registry = load_registry_from_file();

    println!(
        "Loaded {} runtime(s) from {}",
        registry.runtimes.len(),
        RUNTIME_CONFIG_PATH
    );
    for rt in registry.list_runtimes() {
        println!(
            "  - {} (versions: {:?}, source: {}, docker: {})",
            rt.language, rt.versions, rt.source_file, rt.docker_image
        );
    }

    // Example job: Java "Hello Vulkan"
    let job = Job {
        job_id: uuid::Uuid::new_v4(),
        user_id: "test".to_string(),
        language: "java".to_string(),
        version: "25".to_string(),
        code: r#"class Main { public static void main(String[] args) { System.out.println("Hello Vulkan"); } }"#.to_string(),
        testcases: vec![
            TestCase {
                testcase_id: "1".to_string(),
                input: "".to_string(),
                expected_output: "Hello Vukan\n".to_string(),
            },
            TestCase {
                testcase_id: "2".to_string(),
                input: "".to_string(),
                expected_output: "Hello Vulkan\n".to_string(),
            },
        ],
        attempts: 0,
        created_at: 0,
    };

    match execute_job(&job, &registry) {
        Ok(job_result) => println!("{:#?}", job_result),
        Err(e) => eprintln!("Job execution failed: {}", e),
    };
}
