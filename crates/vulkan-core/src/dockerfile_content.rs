pub fn default_dockerfile_content(language: &str, version: &str) -> String {
    match language {
        "python" => python_content(version),
        "java" => java_content(version),
        _ => format!(
            "FROM {}\nWORKDIR /app\n",
            language
        ),
    }
}

pub fn python_content(version: &str) -> String {
    format!(
        "FROM python:{}-slim\nWORKDIR /app\n",
        version
    )
}

pub fn java_content(version: &str) -> String {
    format!(
        "FROM eclipse-temurin:{}-jre\nWORKDIR /app\n",
        version
    )
}