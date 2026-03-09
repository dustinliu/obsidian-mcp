"""Error types for Obsidian MCP server."""


class AppError(Exception):
    """Base exception for all Obsidian MCP errors."""


class HttpError(AppError):
    """Wraps httpx transport errors (connection, timeout, TLS)."""

    def __init__(self, error: Exception) -> None:
        self.error = error
        super().__init__(f"HTTP request failed: {error}")


class ApiError(AppError):
    """Non-2xx HTTP response from the Obsidian API."""

    def __init__(self, status: int, body: str) -> None:
        self.status = status
        self.body = body
        super().__init__(f"Obsidian API error ({status}): {body}")


class JsonError(AppError):
    """JSON deserialization failure."""

    def __init__(self, error: Exception) -> None:
        self.error = error
        super().__init__(f"JSON error: {error}")
