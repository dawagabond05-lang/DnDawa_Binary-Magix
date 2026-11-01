"""Imports, metadata and both entry-point paths (console script and ``python -m``)."""

import contextlib
import importlib
import io
import os
import subprocess
import sys
import tomllib
import unittest
from pathlib import Path

from tests import SRC

ROOT = SRC.parent
MODULES = ("cli", "models", "dungeon", "combat", "game", "persistence", "ui")


def pyproject() -> dict:
    with open(ROOT / "pyproject.toml", "rb") as fh:
        return tomllib.load(fh)


class ImportTests(unittest.TestCase):
    def test_package_and_modules_import(self):
        package = importlib.import_module("dndawa")
        self.assertEqual(Path(package.__file__).resolve().parent, SRC / "dndawa")
        for name in MODULES:
            with self.subTest(module=name):
                importlib.import_module(f"dndawa.{name}")

    def test_version_matches_pyproject(self):
        import dndawa

        self.assertEqual(dndawa.__version__, pyproject()["project"]["version"])

    def test_no_runtime_dependencies(self):
        self.assertEqual(pyproject()["project"]["dependencies"], [])


class CliTests(unittest.TestCase):
    def run_main(self, argv):
        from dndawa.cli import main

        out = io.StringIO()
        with contextlib.redirect_stdout(out):
            code = main(argv)
        return code, out.getvalue()

    def test_main_shows_title_and_exits_zero(self):
        code, out = self.run_main([])
        self.assertEqual(code, 0)
        self.assertIn("DnDawa_Binary-Magix", out)
        self.assertIn("not implemented yet", out)

    def test_version_flag_exits_zero(self):
        import dndawa
        from dndawa.cli import main

        out = io.StringIO()
        with contextlib.redirect_stdout(out), self.assertRaises(SystemExit) as caught:
            main(["--version"])
        self.assertEqual(caught.exception.code, 0)
        self.assertIn(dndawa.__version__, out.getvalue())


class EntryPointTests(unittest.TestCase):
    def test_console_script_target_is_cli_main(self):
        target = pyproject()["project"]["scripts"]["dndawa"]
        self.assertEqual(target, "dndawa.cli:main")
        module_name, _, attr = target.partition(":")
        main = getattr(importlib.import_module(module_name), attr)
        with contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(main([]), 0)

    def test_python_dash_m_runs_and_exits_zero(self):
        env = dict(os.environ, PYTHONPATH=str(SRC))
        done = subprocess.run(
            [sys.executable, "-m", "dndawa"],
            capture_output=True,
            text=True,
            env=env,
            cwd=ROOT,
            timeout=30,
            check=False,
        )
        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertIn("DnDawa_Binary-Magix", done.stdout)


if __name__ == "__main__":
    unittest.main()
