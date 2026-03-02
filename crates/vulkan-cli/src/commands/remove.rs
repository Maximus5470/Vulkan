use std::{env, error::Error};

pub fn handle(args: &mut env::Args) -> Result<(), Box<dyn Error>>{
    let lang = match args.next(){
        Some(lang) => lang,
        None => {
            eprintln!("Language not specified");
            return Err("Language not specified".into());
        }
    };
    loop{
        let version = match args.next(){
        Some(version) => version,
        None => {
            eprintln!("Version not specified");
            return Err("Version not specified".into());
        }
    };

    println!("Removing language {} with version {}", lang, version);
    }
}