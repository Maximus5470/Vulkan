use std::{error::Error, fs, path::Path};
use vulkan_core::models::RuntimeRegistry;

pub mod add;
pub mod remove_lang;
pub mod list;
pub mod remove_version;

const CONFIG_PATH: &str = "crates/config/runtime.json";

pub fn load_registry() -> Result<RuntimeRegistry, Box<dyn Error>> {
	let config_path = Path::new(CONFIG_PATH);

	if !config_path.exists() {
		return Ok(RuntimeRegistry::new());
	}

	let content = fs::read_to_string(config_path)?;
	if content.trim().is_empty() {
		return Ok(RuntimeRegistry::new());
	}

	let runtimes = serde_json::from_str(&content)?;
	Ok(RuntimeRegistry { runtimes })
}

pub fn save_registry(registry: &RuntimeRegistry) -> Result<(), Box<dyn Error>> {
	let output = serde_json::to_string_pretty(&registry.runtimes)?;
	fs::write(Path::new(CONFIG_PATH), output)?;
	Ok(())
}