Vulkan/
├── Cargo.toml (workspace)
├── crates/
│   ├── vulkan-core/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── config.rs
│   │       ├── error.rs
│   │       ├── runtime/
│   │       │   ├── mod.rs
│   │       │   ├── models.rs
│   │       │   ├── registry.rs
│   │       │   └── validation.rs
│   │       └── docker/
│   │           └── mod.rs
│   │
│   ├── vulkan-cli/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs
│   │
│   └── vulkan-server/
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           └── routes/
│               └── runtimes.rs
│
├── examples/
├── tests/
└── target/

2. vulkan-core Implementation

This crate contains domain logic only.
No CLI code. No HTTP code.

2.1 runtime/models.rs
Purpose

Defines runtime-related domain entities.

Implement
pub struct RuntimeRegistry {
    pub languages: Vec<Language>,
}

pub struct Language {
    pub name: String,
    pub versions: Vec<String>,
}

Add Serialize, Deserialize, Debug, Clone.

2.2 runtime/registry.rs
Purpose

Manages runtime state.

Implement Methods
Loading & Saving
pub fn load_from_file(path: &Path) -> Result<RuntimeRegistry, RuntimeError>
pub fn save_to_file(&self, path: &Path) -> Result<(), RuntimeError>
Runtime Operations
pub fn add_language(&mut self, name: &str) -> Result<(), RuntimeError>
pub fn remove_language(&mut self, name: &str) -> Result<(), RuntimeError>

pub fn add_version(&mut self, language: &str, version: &str) -> Result<(), RuntimeError>
pub fn remove_version(&mut self, language: &str, version: &str) -> Result<(), RuntimeError>

pub fn language_exists(&self, name: &str) -> bool
pub fn version_exists(&self, language: &str, version: &str) -> bool
2.3 runtime/validation.rs
Purpose

Validates input before mutation.

Implement
pub fn validate_language_name(name: &str) -> Result<(), RuntimeError>
pub fn validate_version(version: &str) -> Result<(), RuntimeError>
pub fn validate_docker_image(language: &str, version: &str) -> Result<(), RuntimeError>

validate_docker_image should:

Run docker manifest inspect

Or docker pull

Return error if image not found

2.4 error.rs
Purpose

Centralized error handling.

Implement
pub enum RuntimeError {
    LanguageAlreadyExists,
    LanguageNotFound,
    VersionAlreadyExists,
    VersionNotFound,
    InvalidLanguageName,
    InvalidVersion,
    DockerImageNotFound,
    IoError(std::io::Error),
    SerializationError(serde_json::Error),
}

Implement From<std::io::Error> and From<serde_json::Error>.

2.5 config.rs
Purpose

Handles registry file location.

Implement
pub struct Config {
    pub registry_path: PathBuf,
}

impl Config {
    pub fn load() -> Self
}

Default location:

~/.vulkan/runtimes.json

Create directory if not exists.

2.6 docker/mod.rs
Purpose

Abstraction for Docker validation.

Implement Trait
pub trait DockerValidator {
    fn validate_image(&self, image: &str) -> Result<(), RuntimeError>;
}

Later:

CLI will use this for runtime validation

Server will use this for execution

3. vulkan-cli Implementation
Dependencies

clap

vulkan-core

3.1 main.rs
Implement Commands
vulkan runtime add <language> --versions <v1> <v2>
vulkan runtime remove <language> <version>
vulkan runtime list
Required Logic

For add:

Load config

Load registry

Validate language + versions

Validate docker images

Add entries

Save registry

For remove:

Load registry

Remove version

If no versions left → optionally remove language

Save registry

For list:

Load registry

Print languages + versions

4. vulkan-server Implementation
Dependencies

axum (or actix)

serde

vulkan-core

4.1 routes/runtimes.rs
Implement Route
GET /runtimes

Handler:

pub async fn get_runtimes() -> Json<RuntimeRegistry>

Logic:

Load config

Load registry

Return JSON

4.2 main.rs

Initialize router

Register /runtimes

Start server

5. Implementation Order (Strict)
Step 1

Create vulkan-core crate.

Step 2

Implement:

models

error

config

registry load/save

Test manually with simple unit tests.

Step 3

Implement CLI:

runtime add

runtime remove

runtime list

Ensure runtimes.json is created correctly.

Step 4

Add Docker image validation.

Step 5

Add server with /runtimes.

6. What NOT To Implement Yet

Do NOT implement:

Code execution

Timeouts

Memory limits

Worker pools

Async job queues

Kafka

Distributed nodes

You are still in:

Runtime Registry Phase

7. Definition of Completion (Current Milestone)

You are done with Phase 1 when:

vulkan runtime add python --versions 3.11 works

Registry persists correctly

GET /runtimes returns accurate data

Docker images are validated during add

No hardcoded languages exist anywhere