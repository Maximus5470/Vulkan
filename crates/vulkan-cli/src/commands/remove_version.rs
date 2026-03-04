use std::{env, error::Error};

use crate::commands;

pub fn handle(args: &mut env::Args) -> Result<(), Box<dyn Error>> {
    let language = args.next().ok_or("Language not specified")?;
    let version = args.next().ok_or("Version not specified")?;

    let mut registry = commands::load_registry()?;

    if let Some(lang_config) = registry
        .runtimes
        .iter_mut()
        .find(|c| c.language.eq_ignore_ascii_case(&language))
    {
        if let Some(pos) = lang_config
            .versions
            .iter()
            .position(|v| v.eq_ignore_ascii_case(&version))
        {
            lang_config.versions.remove(pos);
        } else {
            eprintln!("Version {} not found for language {}", version, language);
            return Err("Version not found".into());
        }
    } else {
        eprintln!("Language {} not found", language);
        return Err("Language not found".into());
    }
    commands::save_registry(&registry)?;
    println!("Successfully removed version {} of language {}", version, language);
    Ok(())
}