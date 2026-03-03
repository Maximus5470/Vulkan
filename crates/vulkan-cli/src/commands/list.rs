use std::error::Error;

use crate::commands::load_registry;

pub fn handle() -> Result<(), Box<dyn Error>>{
    let registry = load_registry()?;

    for lang in registry.list_runtimes().iter() {
        println!("Language: {}, versions: {:?}", lang.language, lang.versions);
    }

    Ok(())
}