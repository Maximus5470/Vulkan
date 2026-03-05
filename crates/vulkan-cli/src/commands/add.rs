use std::{env, error::Error};
use vulkan_core::LanguageConfig;

use vulkan_core::registry;

/// Usage:
///   vulkan add-language \
///     --language java \
///     --versions 25 \
///     --source-file Main.java \
///     --compile "javac /app/Main.java" \
///     --run "java -cp /app Main" \
///     --docker-image "eclipse-temurin:25-jdk"
///
/// The `--compile` flag is optional (omit for interpreted languages).
pub fn handle(args: &mut env::Args) -> Result<(), Box<dyn Error>> {
    let mut language: Option<String> = None;
    let mut versions: Vec<String> = vec![];
    let mut source_file: Option<String> = None;
    let mut compile_cmd: Option<Vec<String>> = None;
    let mut run_cmd: Option<Vec<String>> = None;
    let mut docker_image: Option<String> = None;

    let all_args: Vec<String> = args.collect();
    let mut i = 0;
    while i < all_args.len() {
        match all_args[i].as_str() {
            "--language" => {
                i += 1;
                if i < all_args.len() {
                    language = Some(all_args[i].clone());
                }
            }
            "--versions" => {
                i += 1;
                while i < all_args.len() && !all_args[i].starts_with("--") {
                    versions.push(all_args[i].clone());
                    i += 1;
                }
                i -= 1;
            }
            "--source-file" => {
                i += 1;
                if i < all_args.len() {
                    source_file = Some(all_args[i].clone());
                }
            }
            "--compile" => {
                i += 1;
                if i < all_args.len() {
                    compile_cmd = Some(
                        all_args[i]
                            .split_whitespace()
                            .map(|s| s.to_string())
                            .collect(),
                    );
                }
            }
            "--run" => {
                i += 1;
                if i < all_args.len() {
                    run_cmd = Some(
                        all_args[i]
                            .split_whitespace()
                            .map(|s| s.to_string())
                            .collect(),
                    );
                }
            }
            "--docker-image" => {
                i += 1;
                if i < all_args.len() {
                    docker_image = Some(all_args[i].clone());
                }
            }
            _ => {
                if language.is_none() {
                    language = Some(all_args[i].clone());
                } else {
                    versions.push(all_args[i].clone());
                }
            }
        }
        i += 1;
    }

    let language = language.ok_or("Language not specified. Use --language <name>")?;

    if versions.is_empty() {
        return Err(
            "At least one version required. Use --versions <version1> <version2> ...".into(),
        );
    }

    let mut registry = registry::load_registry_from_file();

    // If language already exists, just add/merge config
    if let Some(existing) = registry
        .runtimes
        .iter_mut()
        .find(|c| c.language.eq_ignore_ascii_case(&language))
    {
        existing.versions.extend(versions.clone());
        existing.versions.sort();
        existing.versions.dedup();

        // Update optional fields if provided
        if let Some(sf) = source_file {
            existing.source_file = sf;
        }
        if let Some(cc) = compile_cmd {
            existing.compile_cmd = Some(cc);
        }
        if let Some(rc) = run_cmd {
            existing.run_cmd = rc;
        }
        if let Some(di) = docker_image {
            existing.docker_image = di;
        }

        registry::save_registry(&registry)?;
        println!(
            "Updated language '{}' with versions {:?}",
            language, versions
        );
    } else {
        // New language — all fields required except compile_cmd
        let source_file =
            source_file.ok_or("--source-file is required when adding a new language")?;
        let run_cmd = run_cmd.ok_or("--run is required when adding a new language")?;
        let docker_image =
            docker_image.ok_or("--docker-image is required when adding a new language")?;

        let lang_config = LanguageConfig {
            language: language.clone(),
            versions: versions.clone(),
            source_file,
            compile_cmd,
            run_cmd,
            docker_image: docker_image,
        };

        registry.add_runtime(lang_config);
        registry::save_registry(&registry)?;
        println!(
            "Successfully added language '{}' with versions {:?}",
            language, versions
        );
    }

    Ok(())
}
