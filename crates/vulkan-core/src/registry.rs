use std::{error::Error, fs, path::Path};

use crate::RuntimeRegistry;

const CONFIG_PATH: &str = "crates/config/runtime.json";

pub fn load_registry_from_file() -> RuntimeRegistry {
    let config_path = Path::new(CONFIG_PATH);

    if !config_path.exists() {
        return RuntimeRegistry::new();
    }

    let content = fs::read_to_string(config_path).expect("Failed to read runtime.json");
    if content.trim().is_empty() {
        return RuntimeRegistry::new();
    }

    let runtimes = serde_json::from_str(&content).expect("Failed to parse runtime.json");
    RuntimeRegistry { runtimes }
}

pub fn save_registry(registry: &RuntimeRegistry) -> Result<(), Box<dyn Error>> {
    let output = serde_json::to_string_pretty(&registry.runtimes)?;
    fs::write(Path::new(CONFIG_PATH), output)?;
    Ok(())
}
