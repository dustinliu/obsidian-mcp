# Debug build
[group('build')]
build:
    cargo build

# Release build
[group('build')]
build-release: __check
    cargo build --release

# Run the server
[group('build')]
run *ARGS:
    cargo run {{ARGS}}

# Format code
[group('quality')]
fmt:
    cargo fmt

# Check formatting
[group('quality')]
fmt-check:
    cargo fmt --check

# Lint with warnings as errors
[group('quality')]
clippy:
    cargo clippy -- -D warnings

# fmt-check + clippy
[group('quality')]
lint: fmt-check clippy

# Run unit tests
[group('test')]
unit-test:
    cargo test --lib

# Run tests with output
[group('test')]
test-verbose:
    cargo test -- --nocapture

# Run e2e tests (requires OBSIDIAN_API_KEY)
[group('test')]
e2e:
    cargo test --test integration_test -- --test-threads=1 --nocapture

# Run tests with ≥85% line coverage threshold
[group('test')]
coverage:
    cargo llvm-cov --fail-under-lines 85

# Generate HTML coverage report
[group('test')]
coverage-report:
    cargo llvm-cov --html

# Clean build artifacts
[group('build')]
clean:
    cargo clean

# lint + test + build
[group('composite')]
__check: unit-test lint e2e coverage build

[group('deploy')]
deploy: build-release
    cp target/release/obsidian-mcp ~/.local/bin
