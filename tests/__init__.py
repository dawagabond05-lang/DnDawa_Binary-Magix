"""Tests for dndawa. Run from the repository root: ``python3 -m unittest``."""

import sys
from pathlib import Path

# Let the tests import the package from src/ without installing it first.
SRC = Path(__file__).resolve().parent.parent / "src"
if str(SRC) not in sys.path:
    sys.path.insert(0, str(SRC))
