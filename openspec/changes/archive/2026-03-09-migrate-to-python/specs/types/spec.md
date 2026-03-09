## MODIFIED Requirements

### Requirement: Operation enum
`Operation` SHALL be a Python `StrEnum` with values `"append"`, `"prepend"`, `"replace"`. It SHALL be usable directly as HTTP header values.

#### Scenario: Operation values
- **WHEN** `Operation` is used in an HTTP header
- **THEN** its string value SHALL be the lowercase variant name

#### Scenario: Invalid operation rejected
- **WHEN** an invalid string is used to construct an `Operation`
- **THEN** a `ValueError` SHALL be raised

### Requirement: TargetType enum
`TargetType` SHALL be a Python `StrEnum` with values `"heading"`, `"block"`, `"frontmatter"`. It SHALL be usable directly as HTTP header values.

#### Scenario: TargetType values
- **WHEN** `TargetType` is used in an HTTP header
- **THEN** its string value SHALL be the lowercase variant name

#### Scenario: Invalid target type rejected
- **WHEN** an invalid string is used to construct a `TargetType`
- **THEN** a `ValueError` SHALL be raised

### Requirement: PatchParams dataclass
`PatchParams` SHALL be a dataclass (or Pydantic model) with fields: `operation`, `target_type`, `target`, `target_delimiter` (optional), `trim_target_whitespace` (optional), `create_target_if_missing` (optional).

#### Scenario: PatchParams with optional fields
- **WHEN** `PatchParams` is created with only required fields
- **THEN** optional fields SHALL be `None`

### Requirement: AppError hierarchy
`AppError` SHALL be the base exception class. `HttpError` SHALL wrap `httpx` transport errors. `ApiError` SHALL carry `status` (int) and `body` (str) for non-2xx responses. `JsonError` SHALL wrap JSON deserialization errors.

#### Scenario: ApiError display
- **WHEN** an `ApiError` with status 404 and body "Not Found" is converted to string
- **THEN** the result SHALL be `"Obsidian API error (404): Not Found"`

#### Scenario: HttpError display
- **WHEN** an `HttpError` is converted to string
- **THEN** the result SHALL start with `"HTTP request failed: "`
