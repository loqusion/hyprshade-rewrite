from typing import Any

import pytest

from hyprshade.template.mustache import NULLISH_COALESCE_LAMBDA_NAME, render


def nc(text: str) -> str:
    return f"{{{{#{NULLISH_COALESCE_LAMBDA_NAME}}}}}{text}{{{{/{NULLISH_COALESCE_LAMBDA_NAME}}}}}"


def test_chevron_enabled():
    from importlib.util import find_spec

    assert find_spec("chevron") is not None


def test_basic():
    assert render("Hello, world!") == "Hello, world!"


def test_variable():
    assert render("Hello, {{name}}!", {"name": "world"}) == "Hello, world!"


def test_duplicate_data_keys():
    with pytest.raises(
        ValueError, match=f"Invalid data keys: {NULLISH_COALESCE_LAMBDA_NAME}"
    ):
        render("Hello, {{name}}!", {"name": "world", NULLISH_COALESCE_LAMBDA_NAME: 3})


class TestNullishCoalesce:
    def test_nullish_coalesce(self):
        template = f"Hello, {nc('{{name}} ? world')}!"

        assert render(template) == "Hello, world!"
        assert render(template, {"name": "planet"}) == "Hello, planet!"

    @pytest.mark.parametrize("falsy", [0, 0.0])
    def test_falsy_values(self, falsy: Any):
        template = f"Hello, {nc('{{name}} ? world')}!"

        assert render(template, {"name": falsy}) == f"Hello, {falsy}!"

    @pytest.mark.parametrize("nullish", [None, ""])
    def test_nullish_values(self, nullish: Any):
        template = f"Hello, {nc('{{name}} ? world')}!"

        assert render(template, {"name": nullish}) == "Hello, world!"

    def test_no_default_value(self):
        with pytest.raises(ValueError, match="requires a default value"):
            render(f"Hello, {nc('{{name}}')}!")

    def test_multiple_operators(self):
        with pytest.raises(ValueError, match="must occur only once"):
            render(f"Hello, {nc('{{name}} ? world ? planet')}!")
