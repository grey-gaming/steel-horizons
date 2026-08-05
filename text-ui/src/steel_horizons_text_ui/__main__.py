"""CLI entry point for steel-horizons-text-ui."""

from __future__ import annotations

import argparse


def main() -> None:
    parser = argparse.ArgumentParser(description="Steel Horizons text UI")
    parser.add_argument(
        "--version",
        action="version",
        version="steel-horizons-text-ui 0.1.0",
    )
    parser.parse_args()
    print("Steel Horizons text UI — scaffold placeholder")


if __name__ == "__main__":
    main()
