use vulkan_core::{
    docker::execute_job,
    models::{Job, JobSubmission, TestCase},
};

use vulkan_core::registry::load_registry_from_file;

fn main() {
    // Load the registry dynamically from configuration
    let registry = load_registry_from_file();

    // let job = Job {
    //     job_id: uuid::Uuid::new_v4(),
    //     user_id: "test".to_string(),
    //     language: "python".to_string(),
    //     version: "3.11".to_string(),
    //     code: "print(f'Hello, {input().strip()}!')".to_string(),
    //     testcases: vec![
    //         TestCase {
    //             testcase_id: "1".to_string(),
    //             input: "Vulkan".to_string(),
    //             expected_output: "Hello, Vulkan!\n".to_string(),
    //         },
    //         TestCase {
    //             testcase_id: "2".to_string(),
    //             input: "Antigravity".to_string(),
    //             expected_output: "Hello, Antigravity!\n".to_string(),
    //         },
    //     ],
    //     attempts: 0,
    //     created_at: 0,
    // };

    let job = Job {
        job_id: uuid::Uuid::new_v4(),
        user_id: "test".to_string(),
        language: "java".to_string(),
        version: "25".to_string(),
        submission_type: JobSubmission::Run,
        code: r#"class Main { public static void main(String[] args) { System.out.println("Hello Vulkan"); } }"#.to_string(),
        testcases: vec![
            TestCase {
                testcase_id: "1".to_string(),
                input: "".to_string(),
                expected_output: "Hello Vulkan\n".to_string(),
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
