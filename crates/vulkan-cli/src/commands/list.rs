use std::error::Error;

use vulkan_core::registry;

pub fn handle() -> Result<(), Box<dyn Error>> {
    let registry = registry::load_registry_from_file();

    if registry.list_runtimes().is_empty() {
        println!("No runtimes registered. Use 'add-language' to add one.");
        return Ok(());
    }

    println!(
        "{:<12} {:<15} {:<15} {:<30} {:<30} {}",
        "Language", "Versions", "Source File", "Compile Cmd", "Run Cmd", "Docker Image"
    );

    for lang in registry.list_runtimes().iter() {
        let compile_str = match &lang.compile_cmd {
            Some(cmd) => cmd.join(" "),
            None => "(none)".to_string(),
        };
        println!(
            "{:<12} {:<15} {:<15} {:<30} {:<30} {}\n",
            lang.language,
            lang.versions.join(", "),
            lang.source_file,
            compile_str,
            lang.run_cmd.join(" "),
            lang.docker_image,
        );
    }

    Ok(())
}
