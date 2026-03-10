# Technology Stack

**Analysis Date:** 2026-03-10

## Languages

**Primary:**
- Rust (Edition 2024) - All application source code and server implementation

## Runtime

**Environment:**
- Tokio 1.x (async runtime)
- Linux/macOS/Windows compatible via standard Rust toolchain

**Package Manager:**
- Cargo
- Lockfile: Present (`Cargo.lock`)

## Frameworks

**Core:**
- rmcp 0.12 - MCP (Model Context Protocol) server SDK with macros for tool routing
- Axum 0.8 - HTTP web framework for streamable HTTP transport support

**Testing:**
- wiremock 0.6 - HTTP mocking for unit tests (mocks Obsidian REST API)
- serial_test 3.4.0 - Serialization control for concurrent tests
- tokio (test harness) - Built-in async test support

**Build/Dev:**
- clap 4.x - CLI argument parsing with `derive` and `env` features
- cargo-llvm-cov - Code coverage measurement tool
- cargo-release - Automated release management
- just - Task orchestration runner (via justfile)

## Key Dependencies

**Critical:**
- reqwest 0.12 (with json, rustls-tls features) - HTTP client for Obsidian API calls; accepts self-signed TLS certificates
- serde 1.x - Serialization/deserialization framework
- serde_json 1.x - JSON parsing and generation
- schemars 1.x - JSON Schema generation from Rust types for MCP tool argument definitions
- thiserror 2.x - Error type derivation with Display implementation

**Infrastructure:**
- tokio-util 0.7 - Async utilities including CancellationToken for graceful shutdown
- tracing 0.1 - Structured logging framework
- tracing-subscriber 0.3 (with env-filter) - Log filtering and output configuration
- anyhow 1.x - Error context wrapper for main() error handling

**Development:**
- dotenvy 0.15 - Load `.env` files for test configuration

## Configuration

**Environment:**
- CLI arguments (clap) with environment variable fallbacks:
  - `OBSIDIAN_API_URL`: Base URL for Obsidian Local REST API (default: `https://127.0.0.1:27124`)
  - `OBSIDIAN_API_KEY`: Bearer token for REST API authentication (required, no default)
  - `MCP_TRANSPORT`: Transport mode - "stdio" or "http" (default: "stdio")
  - `MCP_PORT`: HTTP server listen port (default: "3000")
  - `MCP_HOST`: HTTP server listen host (default: "127.0.0.1")
  - `RUST_LOG`: Tracing log level filter (e.g., `RUST_LOG=obsidian_mcp=debug`)
- `.env` file support via dotenvy (present at `/workspace/obsidian-mcp/.env`)

**Build:**
- `Cargo.toml`: Package manifest and dependency specifications
- `release.toml`: Cargo-release configuration
- `justfile`: Build and test task definitions

## Platform Requirements

**Development:**
- Rust toolchain (Edition 2024)
- Cargo package manager
- Just task runner (for automated builds/tests)
- OBSIDIAN_API_KEY environment variable for testing

**Production:**
- Linux/macOS/Windows environment
- Network connectivity to Obsidian Local REST API (default: `https://127.0.0.1:27124`)
- MCP client capable of stdio or HTTP/streamable transport (e.g., Claude Desktop)

---

*Stack analysis: 2026-03-10*
