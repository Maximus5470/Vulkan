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

    /// Find a runtime configuration by language name (case-insensitive).
    /// Returns None if the language is not registered.
    pub fn find_runtime(&self, language: &str) -> Option<&LanguageConfig> {
        self.runtimes
            .iter()
            .find(|r| r.language.eq_ignore_ascii_case(language))
    }

    /// Validate that a language + version combination exists in the registry.
    /// Returns the matching runtime config, or an error describing what went wrong.
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
