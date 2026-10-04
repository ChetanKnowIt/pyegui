"""Make `tests/` importable so the widget-option test files can share a harness.

`tests/bounded_app.py` holds the one copy of the bounded-app runner. Importing
it as `from bounded_app import run_snippet` (rather than
`tests.bounded_app`, which would need `tests/__init__.py`) relies on `tests/`
being on `sys.path`. pytest's default `prepend` import mode puts the first
directory without an `__init__.py` -- which is `tests/` -- on `sys.path` for
us, but `tests/test_scenario_names_agree.py` already inserts it explicitly, so
this makes the one assumption explicit rather than implicit and inconsistent.
"""

import sys
from pathlib import Path

TESTS_DIR = Path(__file__).resolve().parent
if str(TESTS_DIR) not in sys.path:
    sys.path.insert(0, str(TESTS_DIR))
