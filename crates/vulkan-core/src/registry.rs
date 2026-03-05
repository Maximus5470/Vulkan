use std::{error::Error, fs, path::Path};

use crate::models::{LanguageConfig, RuntimeRegistry};

impl RuntimeRegistry {
    pub fn new() -> Self {
        RuntimeRegistry {
            runtimes: Vec::new(),
        }
    }

    pub fn add_runtime(&mut self, language_config: LanguageConfig) {
        self.runtimes.push(language_config);
    }

    pub fn remove_runtime(&mut self, language: &str) {
        self.runtimes.retain(|config| config.language != language);
    }

    pub fn list_runtimes(&self) -> &Vec<LanguageConfig> {
        &self.runtimes
    }

    pub fn find_runtime(&self, language: &str) -> Option<&LanguageConfig> {
        self.runtimes
            .iter()
            .find(|r| r.language.eq_ignore_ascii_case(language))
    }

    // Checks if a language + version combination exists in the registry
    pub fn validate_runtime(
        &self,
        language: &str,
        version: &str,
    ) -> Result<&LanguageConfig, String> {
        let runtime = self
            .find_runtime(language)
            .ok_or_else(|| format!("Language '{}' not found in registry", language))?;

        if !runtime.versions.iter().any(|v| v == version) {
            return Err(format!(
                "Version '{}' not found for language '{}'. Available versions: {:?}",
                version, language, runtime.versions
            ));
        }

        Ok(runtime)
    }
}

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
