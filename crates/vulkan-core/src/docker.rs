use std::{
    collections::HashSet, error::Error, fs::{self, File}, io::Write, path::PathBuf, process::Command
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

    let output = Command::new("docker")
        .args(["rmi", &image])
        .output()?;

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

pub fn update_images(
    registry: &RuntimeRegistry,
) -> Result<(), Box<dyn Error>> {
    let existing_images = list_images()?;

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
                    // If the image was removed successfully, we can also remove the corresponding Dockerfile directory
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