"""Tests for error types."""

import json

from obsidian_mcp.errors import ApiError, AppError, HttpError, JsonError


class TestHttpError:
    def test_display_format(self):
        err = HttpError(ConnectionError("connection refused"))
        assert str(err) == "HTTP request failed: connection refused"

    def test_inherits_app_error(self):
        err = HttpError(ConnectionError("fail"))
        assert isinstance(err, AppError)

    def test_wraps_original_error(self):
        original = ConnectionError("timeout")
        err = HttpError(original)
        assert err.error is original


class TestApiError:
    def test_display_format(self):
        err = ApiError(404, "Not Found")
        assert str(err) == "Obsidian API error (404): Not Found"

    def test_inherits_app_error(self):
        err = ApiError(500, "Internal")
        assert isinstance(err, AppError)

    def test_status_and_body_attributes(self):
        err = ApiError(403, "Forbidden")
        assert err.status == 403
        assert err.body == "Forbidden"


class TestJsonError:
    def test_display_format(self):
        try:
            json.loads("not json")
        except json.JSONDecodeError as e:
            err = JsonError(e)
            assert str(err).startswith("JSON error: ")

    def test_inherits_app_error(self):
        err = JsonError(ValueError("bad json"))
        assert isinstance(err, AppError)

    def test_wraps_original_error(self):
        original = ValueError("parse error")
        err = JsonError(original)
        assert err.error is original
