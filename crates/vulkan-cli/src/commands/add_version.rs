use std::{env, error::Error};

use vulkan_core::registry;

pub fn handle(args: &mut env::Args) -> Result<(), Box<dyn Error>> {
    let language = match args.next() {
        Some(arg) => arg,
        _ => return Err("Language not specified".into()),
    };
    let version = match args.next() {
        Some(arg) => arg,
        _ => return Err("Version not specified".into()),
    };

    let mut registry = registry::load_registry_from_file();
    let runtime = registry
        .find_runtime(&language)
        .ok_or_else(|| format!("Language '{}' not found in registry", language))?;

    if runtime.versions.contains(&version) {
        return Err(format!(
            "Version '{}' already exists for language '{}'",
            version, language
        )
        .into());
    }

    // Add the new version to the existing runtime config
    let mut updated_runtime = runtime.clone();
    updated_runtime.versions.push(version.clone());

    // Remove the old runtime config and add the updated one
    registry.remove_runtime(&language);
    registry.add_runtime(updated_runtime);

    registry::save_registry(&registry)?;
    println!("Added version '{}' to language '{}'", version, language);
    Ok(())
}