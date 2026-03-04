/// Generate a generic Dockerfile content for any language runtime.
///
/// This is completely language-agnostic — the base image is determined
/// by the `docker_image` field in the runtime configuration.
///
/// The generated container simply sets up `/app` as the working directory.
/// Compilation and execution are handled via `docker exec` from the host.
pub fn generic_dockerfile_content(docker_image: &str) -> String {
    format!("FROM {}\nWORKDIR /app\n", docker_image)
}
