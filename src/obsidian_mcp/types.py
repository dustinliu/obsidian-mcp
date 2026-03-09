"""Shared domain types for Obsidian MCP server."""

from dataclasses import dataclass
from enum import StrEnum


class Operation(StrEnum):
    """Patch operation type."""

    APPEND = "append"
    PREPEND = "prepend"
    REPLACE = "replace"


class TargetType(StrEnum):
    """Patch target type."""

    HEADING = "heading"
    BLOCK = "block"
    FRONTMATTER = "frontmatter"


@dataclass
class PatchParams:
    """Parameters for an Obsidian REST API v3 PATCH request."""

    operation: Operation
    target_type: TargetType
    target: str
    target_delimiter: str | None = None
    trim_target_whitespace: bool | None = None
    create_target_if_missing: bool | None = None
