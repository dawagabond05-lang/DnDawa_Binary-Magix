"""Command-line entry point for DnDawa_Binary-Magix.

Gameplay is not implemented yet: ``main()`` shows the title and exits cleanly.
"""

from __future__ import annotations

import argparse
from collections.abc import Sequence

from dndawa import __version__

TITLE = "DnDawa_Binary-Magix"
NOT_READY = "Gameplay is not implemented yet. Nothing was started."


def build_parser() -> argparse.ArgumentParser:
    """Return the argument parser for the ``dndawa`` command."""
    parser = argparse.ArgumentParser(
        prog="dndawa",
        description=f"{TITLE}: a turn-based terminal dungeon crawler.",
    )
    parser.add_argument(
        "--version", action="version", version=f"%(prog)s {__version__}"
    )
    return parser


def main(argv: Sequence[str] | None = None) -> int:
    """Run the ``dndawa`` command and return its exit status."""
    build_parser().parse_args(argv)
    print(f"{TITLE} v{__version__}")
    print(NOT_READY)
    return 0
