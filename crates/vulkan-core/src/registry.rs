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
}
