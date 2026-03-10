pub fn generic_dockerfile_content(docker_image: &str) -> String {
    format!("FROM {}\nWORKDIR /app\nRUN useradd -m -u 10001 vulkan\nUSER vulkan", docker_image)
}
