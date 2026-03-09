"""Tests for shared domain types."""

import pytest

from obsidian_mcp.types import Operation, PatchParams, TargetType


class TestOperation:
    def test_values(self):
        assert Operation.APPEND == "append"
        assert Operation.PREPEND == "prepend"
        assert Operation.REPLACE == "replace"

    def test_string_representation(self):
        assert str(Operation.APPEND) == "append"
        assert str(Operation.PREPEND) == "prepend"
        assert str(Operation.REPLACE) == "replace"

    def test_from_string(self):
        assert Operation("append") is Operation.APPEND
        assert Operation("prepend") is Operation.PREPEND
        assert Operation("replace") is Operation.REPLACE

    def test_invalid_value_raises(self):
        with pytest.raises(ValueError):
            Operation("invalid")


class TestTargetType:
    def test_values(self):
        assert TargetType.HEADING == "heading"
        assert TargetType.BLOCK == "block"
        assert TargetType.FRONTMATTER == "frontmatter"

    def test_string_representation(self):
        assert str(TargetType.HEADING) == "heading"
        assert str(TargetType.BLOCK) == "block"
        assert str(TargetType.FRONTMATTER) == "frontmatter"

    def test_from_string(self):
        assert TargetType("heading") is TargetType.HEADING
        assert TargetType("block") is TargetType.BLOCK
        assert TargetType("frontmatter") is TargetType.FRONTMATTER

    def test_invalid_value_raises(self):
        with pytest.raises(ValueError):
            TargetType("invalid")


class TestPatchParams:
    def test_required_fields(self):
        params = PatchParams(
            operation=Operation.APPEND,
            target_type=TargetType.HEADING,
            target="Section 1",
        )
        assert params.operation == Operation.APPEND
        assert params.target_type == TargetType.HEADING
        assert params.target == "Section 1"

    def test_optional_fields_default_none(self):
        params = PatchParams(
            operation=Operation.REPLACE,
            target_type=TargetType.BLOCK,
            target="block-id",
        )
        assert params.target_delimiter is None
        assert params.trim_target_whitespace is None
        assert params.create_target_if_missing is None

    def test_optional_fields_set(self):
        params = PatchParams(
            operation=Operation.PREPEND,
            target_type=TargetType.FRONTMATTER,
            target="tags",
            target_delimiter="::",
            trim_target_whitespace=True,
            create_target_if_missing=True,
        )
        assert params.target_delimiter == "::"
        assert params.trim_target_whitespace is True
        assert params.create_target_if_missing is True
