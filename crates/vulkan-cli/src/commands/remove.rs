use std::{env, error::Error};

use crate::commands::{load_registry, save_registry};

pub fn handle(args: &mut env::Args) -> Result<(), Box<dyn Error>>{
    let language = match args.next(){
        Some(language) => language,
        None => {
            eprintln!("Language not specified");
            return Err("Language not specified".into());
        }
    };

    let mut registry = load_registry()?;
    registry.remove_runtime(&language);
    save_registry(&registry)?;

    println!("Successfully removed language {}", language);
    Ok(())
}