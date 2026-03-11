# Vulkan

**Vulkan** is a high-performance, sandboxed **remote code execution (RCE) engine** written in Rust. It accepts code submissions via an HTTP API, queues them using a **Multi-Level Queue (MLQ)** backed by Redis, and executes them inside isolated Docker containers — all without giving user code any network or filesystem access.

---

## Table of Contents

- [Architecture Overview](#architecture-overview)
- [Crates](#crates)
  - [vulkan-core](#vulkan-core)
  - [vulkan-api](#vulkan-api)
  - [vulkan-server](#vulkan-server)
  - [vulkan-cli](#vulkan-cli)
- [API Reference](#api-reference)
- [Runtime Configuration (`runtime.json`)](#runtime-configuration-runtimejson)
- [Environment Variables (`.env`)](#environment-variables-env)
- [Setup & Running](#setup--running)
  - [Prerequisites](#prerequisites)
  - [1. Clone and Build](#1-clone-and-build)
  - [2. Register Languages](#2-register-languages)
  - [3. Build Docker Images](#3-build-docker-images)
  - [4. Start the Worker Server](#4-start-the-worker-server)
  - [5. Start the API Server](#5-start-the-api-server)
- [CLI Reference](#cli-reference)
- [Security Model](#security-model)
- [Project Structure](#project-structure)

---

## Architecture Overview

```
Client (HTTP)
     │
     ▼
┌─────────────────┐
│   vulkan-api    │  ← Axum HTTP server
│  (REST API)     │  Accepts job submissions, queries results
└────────┬────────┘
         │  push_job / get_result (Redis)
         ▼
┌─────────────────┐
│     Redis       │  ← MLQ queues + job/result hashes
│  (3 queues)     │  high / medium / low priority
└────────┬────────┘
         │  fetch_job (blocking BRPOP)
         ▼
┌─────────────────┐
│  vulkan-server  │  ← Worker pool (synchronous threads)
│  (Worker Pool)  │  Each worker has a priority bias
└────────┬────────┘
         │  execute_job
         ▼
┌─────────────────┐
│  vulkan-core    │  ← Docker execution engine
│  (Execution)    │  Isolated container per job
└─────────────────┘
```

Jobs are pushed to one of three Redis queues based on submission type and test-case count. Workers continuously poll their preferred queue and execute jobs in sandboxed containers. Results are stored in Redis (TTL: 5 minutes) and retrieved asynchronously by the client.

---

## Crates

This is a Cargo workspace with four crates:

### `vulkan-core`

Shared library used by all other crates. Contains:

| Module | Description |
|--------|-------------|
| `lib.rs` | Core data types: `Job`, `JobResult`, `SubmitJobRequest`, `TestCase`, `TestcaseResult`, `LanguageConfig`, `RuntimeRegistry`, `Priority`, `JobStatus`, `JobSubmission` |
| `docker.rs` | Docker container lifecycle management, code execution engine, image building/updating |
| `registry.rs` | Reads/writes `crates/config/runtime.json` |
| `dockerfile_content.rs` | Generates generic Dockerfile content for a given base image |

### `vulkan-api`

An **async Axum HTTP server** that acts as the API gateway.

**Handlers:**

| File | Route | Method | Description |
|------|-------|--------|-------------|
| `health.rs` | `/health` | `GET` | Returns server uptime and status |
| `execute.rs` | `/execute` | `POST` | Accepts a job submission, enqueues into MLQ |
| `job.rs` | `/job/{id}` | `GET` | Polls for a job's result by UUID |
| `runtimes.rs` | `/runtimes` | `GET` | Lists all registered language runtimes |
| `metrics.rs` | `/metrics` | `GET` | Prometheus-format queue length metrics |

### `vulkan-server`

A **synchronous worker process** that continuously dequeues and executes jobs.

- Spawns a configurable pool of OS threads (`WORKER_POOL_SIZE`)
- Each worker has a **priority bias** (High / Medium / Low), meaning it prefers its assigned queue but will fall back to others when its queue is empty
- Worker counts per priority tier are independently configurable via env vars
- Uses `BRPOP` (blocking Redis pop) to wait efficiently for new jobs
- Results are stored back to Redis after execution

### `vulkan-cli`

A command-line tool for managing the language runtime registry.

---

## API Reference

### `POST /execute`

Submits code for execution. Returns a job ID.

**Request Body (JSON):**

```json
{
  "language": "python",
  "version": "3.11",
  "code": "print('Hello, World!')",
  "submission_type": "Run",
  "testcases": []
}
```

| Field | Type | Description |
|-------|------|-------------|
| `language` | `string` | Language name (must match registry) |
| `version` | `string` | Language version (must match registry) |
| `code` | `string` | Source code to execute |
| `submission_type` | `"Run"` \| `"Submit"` | `Run` = execute once; `Submit` = run against test cases |
| `testcases` | `array` | Required for `Submit` mode (can be empty for `Run`) |

**Testcase object:**

```json
{
  "testcase_id": "tc1",
  "input": "5\n3",
  "expected_output": "8"
}
```

**Response:** `202 Accepted` with body: `Job submitted with ID: <uuid> (Xms)`

---

### `GET /job/{id}`

Retrieves the result of a previously submitted job.

**Response (JSON):**

```json
{
  "job_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "Success",
  "stdout": "Hello, World!",
  "stderr": "",
  "execution_time_ms": 124,
  "testcases": []
}
```

> Results are cached in Redis for **5 minutes**. Poll until the job ID resolves.

---

### `GET /runtimes`

Returns the list of all registered language runtimes from `runtime.json`.

---

### `GET /metrics`

Returns Prometheus-format metrics including queue lengths per priority tier:

```
vulkan_queue_length{priority="high"} 0
vulkan_queue_length{priority="medium"} 3
vulkan_queue_length{priority="low"} 1
```

---

### `GET /health`

Returns server uptime in seconds and a status indicator.

---

## Runtime Configuration (`runtime.json`)

Located at `crates/config/runtime.json`. This file defines all supported languages and their execution configuration. It is read at startup by both `vulkan-api` and `vulkan-server`, and is managed by the `vulkan-cli`.

**Format:**

```json
[
  {
    "language": "python",
    "versions": ["3.11", "3.9"],
    "source_file": "main.py",
    "compile_cmd": null,
    "run_cmd": ["python", "/app/main.py"],
    "docker_image": "python:{version}-slim"
  },
  {
    "language": "java",
    "versions": ["25"],
    "source_file": "Main.java",
    "compile_cmd": ["javac", "/app/Main.java"],
    "run_cmd": ["java", "-cp", "/app", "Main"],
    "docker_image": "eclipse-temurin:{version}-jdk"
  }
]
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `language` | `string` | ✅ | Unique language identifier (case-insensitive) |
| `versions` | `string[]` | ✅ | Supported version strings (e.g. `["3.11", "3.9"]`) |
| `source_file` | `string` | ✅ | Filename to write the submitted code to inside the container |
| `compile_cmd` | `string[]` \| `null` | ❌ | Compilation command. `null` for interpreted languages |
| `run_cmd` | `string[]` | ✅ | Command to execute the code |
| `docker_image` | `string` | ✅ | Base Docker image. Use `{version}` as a placeholder that is replaced at runtime |

> **Note:** The `{version}` placeholder in `docker_image` is automatically substituted with the specific version being processed. For example, `python:{version}-slim` becomes `python:3.11-slim`.

---

## Environment Variables (`.env`)

Create a `.env` file in the project root. All variables are optional and have defaults.

| Variable | Default | Description |
|----------|---------|-------------|
| `REDIS_URL` | `redis://127.0.0.1:6379` | Full Redis connection URL used by both API and worker |
| `PORT` | `8000` | Port the `vulkan-api` HTTP server listens on |
| **Worker Pool** | | |
| `WORKER_POOL_SIZE` | `1` | Total number of worker threads in `vulkan-server` |
| `HIGH_QUEUE_WORKER_COUNT` | `0` | Number of workers with a **High** priority bias |
| `MEDIUM_QUEUE_WORKER_COUNT` | Equals `WORKER_POOL_SIZE` | Number of workers with a **Medium** priority bias |
| `LOW_QUEUE_WORKER_COUNT` | `0` | Number of workers with a **Low** priority bias |
| **Queue Limits** | | |
| `HIGH_QUEUE_LIMIT` | `100` | Maximum number of jobs allowed in the high-priority queue |
| `MEDIUM_QUEUE_LIMIT` | `500` | Maximum number of jobs allowed in the medium-priority queue |
| `LOW_QUEUE_LIMIT` | `1000` | Maximum number of jobs allowed in the low-priority queue |
| **Job Settings** | | |
| `TESTCASE_COUNT_LIMIT` | `1000` | Test case threshold for `Submit` jobs. Jobs with fewer test cases go to the **Medium** queue; those with more go to **Low** |

### Priority Assignment Logic

| Submission Type | Condition | Queue |
|----------------|-----------|-------|
| `Run` | Always | High |
| `Submit` | `testcase_count < TESTCASE_COUNT_LIMIT` | Medium |
| `Submit` | `testcase_count >= TESTCASE_COUNT_LIMIT` | Low |

### Example `.env`

```env
REDIS_URL="<redis_url>"
PORT=8000

WORKER_POOL_SIZE=4
HIGH_QUEUE_WORKER_COUNT=2
MEDIUM_QUEUE_WORKER_COUNT=1
LOW_QUEUE_WORKER_COUNT=1

HIGH_QUEUE_LIMIT=100
MEDIUM_QUEUE_LIMIT=500
LOW_QUEUE_LIMIT=1000

TESTCASE_COUNT_LIMIT=10
```

---

## Setup & Running

### Prerequisites

- **Rust** (stable, 1.75+) — [rustup.rs](https://rustup.rs)
- **Docker** — must be running and accessible from the command line
- **Redis** — running on the URL specified in `REDIS_URL`

### 1. Clone and Build

```bash
git clone https://github.com/your-org/vulkan.git
cd vulkan

# Build everything in release mode
cargo build --release
```

Binaries will be at:
- `target/release/vulkan-api`
- `target/release/vulkan-server`
- `target/release/vulkan-cli`

### 2. Register Languages

Use the CLI to add language runtimes to `crates/config/runtime.json`:

```bash
# Add Python (interpreted — no compile step)
./target/release/vulkan-cli add-language \
  --language python \
  --versions 3.11 3.9 \
  --source-file main.py \
  --run "python /app/main.py" \
  --docker-image "python:{version}-slim"

# Add Java (compiled)
./target/release/vulkan-cli add-language \
  --language java \
  --versions 25 \
  --source-file Main.java \
  --compile "javac /app/Main.java" \
  --run "java -cp /app Main" \
  --docker-image "eclipse-temurin:{version}-jdk"
```

### 3. Build Docker Images

This pulls base images and builds local tagged images (e.g. `vulkan-python:3.11`) for every registered language/version:

```bash
./target/release/vulkan-cli update_images
```

> Run this command from the **project root** directory so Dockerfiles are written to `./dockerfiles/`.

### 4. Start the Worker Server

```bash
./target/release/vulkan-server
```

The server reads `WORKER_POOL_SIZE` from `.env` and spawns the specified number of worker threads. Each thread blocks on Redis and processes jobs as they arrive.

### 5. Start the API Server

```bash
./target/release/vulkan-api
```

The API server listens on `0.0.0.0:<PORT>`. You can verify it's running:

```bash
curl http://localhost:8000/health
curl http://localhost:8000/runtimes
```

---

## CLI Reference

The `vulkan-cli` binary manages the runtime registry stored in `crates/config/runtime.json`.

| Command | Aliases | Description |
|---------|---------|-------------|
| `add-language` | `add` | Add a new language or update an existing one |
| `add_version` | — | Add a new version to an existing language |
| `remove-language` | `remove-lang` | Remove an entire language from the registry |
| `remove_version` | — | Remove a specific version from a language |
| `list` | — | List all registered languages and their versions |
| `clear_redis` | — | Flush all Vulkan queues and job data from Redis |
| `update_images` | — | Build/sync Docker images to match the registry |

### `add-language` / `add`

```
vulkan-cli add-language \
  --language <name>           Required. Language identifier
  --versions <v1> <v2> ...    Required. One or more version strings
  --source-file <filename>    Required (new language). Source file name inside container
  --run "<cmd>"               Required (new language). Run command
  --compile "<cmd>"           Optional. Compile command (omit for interpreted languages)
  --docker-image "<image>"    Required (new language). Base Docker image (supports {version})
```

---

## Security Model

Each code execution job runs inside a freshly created Docker container with strict resource and capability limits:

| Constraint | Value |
|------------|-------|
| Network access | **None** (`--network none`) |
| Memory limit | 256 MB |
| Swap | Disabled (memory-swap = memory) |
| CPU limit | 1.0 CPU |
| File descriptors | 256 |
| Process limit | 64 PIDs, 64 nproc |
| CPU time (ulimit) | 10 seconds |
| Wall-clock timeout | 10 seconds (enforced by Rust) |
| Linux capabilities | **All dropped** (`--cap-drop ALL`) |
| Privilege escalation | Disabled (`--security-opt no-new-privileges`) |

Containers are destroyed immediately after job execution. Source files are written to a temporary directory that is also deleted post-execution.

---

## Project Structure

```
Vulkan/
├── .env                          # Environment configuration
├── Cargo.toml                    # Workspace manifest
├── crates/
│   ├── config/
│   │   └── runtime.json          # Language runtime registry
│   ├── vulkan-core/              # Shared library (models, docker, registry)
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── docker.rs
│   │       ├── registry.rs
│   │       └── dockerfile_content.rs
│   ├── vulkan-api/               # HTTP API server (Axum)
│   │   └── src/
│   │       ├── main.rs
│   │       ├── routes.rs
│   │       └── handlers/
│   │           ├── execute.rs
│   │           ├── job.rs
│   │           ├── health.rs
│   │           ├── runtimes.rs
│   │           └── metrics.rs
│   ├── vulkan-server/            # Worker process
│   │   └── src/
│   │       ├── main.rs
│   │       ├── scheduler.rs      # MLQ implementation
│   │       ├── scheduler_push.lua
│   │       └── worker/
│   │           ├── mod.rs
│   │           ├── worker.rs
│   │           └── worker_pool.rs
│   └── vulkan-cli/               # CLI management tool
│       └── src/
│           ├── main.rs
│           ├── cli.rs
│           └── commands/
│               ├── add.rs
│               ├── add_version.rs
│               ├── remove_lang.rs
│               ├── remove_version.rs
│               ├── list.rs
│               └── clear_redis.rs
├── dockerfiles/                  # Auto-generated per language/version
│   └── <language>/<version>/Dockerfile
└── tests/                        # Integration tests
```
