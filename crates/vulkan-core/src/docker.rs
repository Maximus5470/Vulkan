use std::{
    error::Error, fs::{self, File}, io::Write, path::PathBuf, process::Command
};

use crate::{dockerfile_content::default_dockerfile_content, models::RuntimeRegistry};

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
    PathBuf::from("dockerfiles")
        .join(language)
        .join(version)
}



pub fn ensure_dockerfile(
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

pub fn image_exists(language: &str, version: &str) -> bool {
    let image = image_name(language, version);

    match Command::new("docker")
        .args(["image", "inspect", &image])
        .status()
    {
        Ok(status) => status.success(),
        Err(_) => false,
    }
}

fn build_image(
    language: &str,
    version: &str,
) -> Result<(), Box<dyn Error>> {

    validate_input(language)?;
    validate_input(version)?;

    let image = image_name(language, version);
    let dir = dockerfile_dir(language, version);

    if !dir.exists() {
        return Err(format!(
            "Dockerfile directory missing: {}",
            dir.display()
        ).into());
    }

    let status = Command::new("docker")
        .args(["build", "-t", &image, dir.to_str().unwrap()])
        .status()?;

    if !status.success() {
        return Err(format!("Failed to build Docker image: {}", image).into());
    }

    Ok(())
}

fn remove_image(
    language: &str,
    version: &str,
) -> Result<(), Box<dyn Error>> {

    let image = image_name(language, version);

    let status = Command::new("docker")
        .args(["rmi", &image])
        .status()?;

    if !status.success() {
        return Err(format!("Failed to remove Docker image: {}", image).into());
    }

    Ok(())
}

pub fn update_images(
    registry: &RuntimeRegistry,
) -> Result<(), Box<dyn Error>> {

    for runtime in &registry.runtimes {
        for version in &runtime.versions {
            if !image_exists(&runtime.language, version) {
                println!(
                    "Building Docker image for {} {}",
                    runtime.language, version
                );

                let dockerfile_content = default_dockerfile_content(&runtime.language, version);
                ensure_dockerfile(&runtime.language, version, &dockerfile_content)?;
                build_image(&runtime.language, version)?;
            }
        }
    }

    Ok(())
}