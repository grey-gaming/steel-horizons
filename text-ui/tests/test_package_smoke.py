"""Package smoke test for the Steel Horizons text UI.

A real, minimal test so the pytest gate is honest: it imports the package and
checks the advertised entry point, instead of the previous collect-only guard
that silently swallowed a non-zero exit.
"""

from __future__ import annotations

import steel_horizons_text_ui


def test_package_importable() -> None:
    """The text-ui package imports and reports its version."""
    assert steel_horizons_text_ui.__version__ == "0.1.0"


def test_main_module_has_entry_point() -> None:
    """The __main__ module exposes a `main()` callable for the console script."""
    import steel_horizons_text_ui.__main__ as main_mod

    assert callable(getattr(main_mod, "main", None))
