pub fn generic_dockerfile_content(docker_image: &str) -> String {
    format!("FROM {}\nWORKDIR /app\n", docker_image)
}
