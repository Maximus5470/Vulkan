use std::{env, error::Error};
use vulkan_core::LanguageConfig;

use crate::commands::{load_registry, save_registry};

pub fn handle(args: &mut env::Args) -> Result<(), Box<dyn Error>>{

    let language = match args.next(){
        Some(language) => language,
        None => {
            eprintln!("Language not specified");
            return Err("Language not specified".into());
        }
    };

    let mut versions = vec![];

    loop {
        let version = match args.next() {
            Some(version) => version,
            None => break,
        };

        versions.push(version);
    }

    if versions.is_empty() {
        eprintln!("Version not specified");
        return Err("Version not specified".into());
    }

    let lang_config = LanguageConfig{
        language,
        versions
    };

    let language_display = lang_config.language.clone();
    let versions_display = lang_config.versions.clone();

    let mut registry = load_registry()?;

    if let Some(existing) = registry
        .runtimes
        .iter_mut()
        .find(|c| c.language.eq_ignore_ascii_case(&lang_config.language))
    {
        existing.versions.extend(lang_config.versions);
        existing.versions.sort();
        existing.versions.dedup();
    } else {
        registry.add_runtime(lang_config);
    }

    save_registry(&registry)?;

    println!("Successfully added language {} with versions {:?}", language_display, versions_display);
    Ok(())
}