## MODIFIED Requirements

### Requirement: Client construction
`ObsidianClient` SHALL be constructed with `base_url` and `api_key` parameters. It SHALL use `httpx.AsyncClient` with `verify=False` to accept self-signed TLS certificates. The bearer token SHALL be pre-formatted as `"Bearer {api_key}"` at construction time.

#### Scenario: Client construction with self-signed cert
- **WHEN** `ObsidianClient` is created with a base_url and api_key
- **THEN** the underlying `httpx.AsyncClient` SHALL have TLS verification disabled
- **AND** the authorization header SHALL be `"Bearer {api_key}"`

### Requirement: HTTP method mapping
`ObsidianClient` SHALL map vault operations to HTTP methods: GET=read, PUT=create, POST=append, PATCH=partial update, DELETE=delete. All request/response patterns, headers, and URL structures SHALL remain identical to the Rust implementation.

#### Scenario: All 15 API methods preserved
- **WHEN** any of the 15 public API methods is called
- **THEN** the HTTP method, endpoint path, headers, and body format SHALL match the existing spec exactly

### Requirement: Error response handling
`ObsidianClient` SHALL check HTTP response status codes. Non-2xx responses SHALL raise `ApiError` with the status code and response body text.

#### Scenario: Non-2xx response
- **WHEN** the Obsidian API returns a non-2xx status code
- **THEN** an `ApiError` SHALL be raised with `status` and `body` attributes

### Requirement: Periodic URL construction
The periodic URL helper SHALL require all three date parts (year, month, day) to produce a dated URL `/periodic/{period}/{y}/{m}/{d}/`. Any partial combination SHALL fall back to `/periodic/{period}/`.

#### Scenario: Partial date falls back to period-only URL
- **WHEN** `periodic_url` is called with year but without month or day
- **THEN** the URL SHALL be `/periodic/{period}/`

### Requirement: Async context manager
`ObsidianClient` SHALL implement async context manager protocol (`__aenter__`/`__aexit__`) to properly close the underlying `httpx.AsyncClient`.

#### Scenario: Client cleanup
- **WHEN** `ObsidianClient` is used in an `async with` block and the block exits
- **THEN** the underlying httpx client SHALL be closed
