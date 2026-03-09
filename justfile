# Run the server (stdio by default; use -- --transport http for HTTP)
[group('build')]
run *ARGS:
    uv run obsidian-mcp {{ARGS}}

# Format code
[group('quality')]
fmt:
    uv run ruff format .

# Check formatting
[group('quality')]
fmt-check:
    uv run ruff format --check .

# Lint
[group('quality')]
lint: fmt-check
    uv run ruff check .

# Fix lint issues
[group('quality')]
lint-fix:
    uv run ruff check --fix .

# Run unit tests
[group('test')]
unit-test:
    uv run pytest tests/ -v --ignore=tests/test_e2e.py

# Run tests with output
[group('test')]
test-verbose:
    uv run pytest tests/ -v -s --ignore=tests/test_e2e.py

# Run e2e tests (requires OBSIDIAN_API_KEY)
[group('test')]
e2e:
    uv run pytest tests/test_e2e.py -v -s

# Run tests with ≥85% line coverage threshold
[group('test')]
coverage:
    uv run pytest tests/ --ignore=tests/test_e2e.py --cov=obsidian_mcp --cov-report=term --cov-fail-under=85

# Generate HTML coverage report
[group('test')]
coverage-report:
    uv run pytest tests/ --ignore=tests/test_e2e.py --cov=obsidian_mcp --cov-report=html

# Clean build artifacts
[group('build')]
clean:
    rm -rf .venv .pytest_cache .ruff_cache htmlcov .coverage

# lint + test + coverage
[group('composite')]
__check: unit-test lint coverage

# Deploy to ~/.local/bin
[group('deploy')]
deploy: __check
    uv tool install --force .
