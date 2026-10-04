"""Shared harness for the builder-option test files.

## Why this exists, and why it is one module and not three copies

Every widget-options task in this plan needs the same thing: a way to call a
widget inside a real egui update function, catch whatever comes out, and only
assert AFTER `run_native` has returned. That shape is not stylistic. An
exception raised inside a widget's draw does not close the app:
`PyeguiApp::update` does
`if let Err(err) = self.update_func.call1((ctx_r,)) { err.display(py); }` and
then returns `Ok`, and `run_nested_update_func` does the same with `call0()`.
Nothing calls `ctx.close()`, so the app keeps repainting and the failure
presents as a HANG under a wall-clock bound rather than as a message naming
the offending option.

The bounded runner below is what turns that hang into a readable assertion:

1. `record`-style helpers call the widget inside the update function and
   *catch* whatever comes out, appending it to a list. Nothing propagates, so
   the app never hangs and always closes normally.
2. `run_native` returns once the deadline closes the window.
3. The snippet asserts on the collected outcomes and `sys.exit(1)` if any
   expectation failed, so pytest sees a failing process with output.

A widget that silently ignored `calender=True` would append `"NOT RAISED"`,
which fails step 3 with a readable message.

`BOUNDED_RUNNER` and `run_snippet` started life duplicated in
`test_slider_options.py` and `test_drag_textedit_options.py`. Rather than let a
third copy appear for the date picker, both now import from here. Keeping the
definition in one place matters for correctness, not just tidiness: the two
copies had already drifted (the Task 3 file filters `xkbcommon:` noise, the
Task 2 file did not), and a third variant would have been a third set of
behaviour nobody compared.

`run_snippet` is deliberately NOT `tests/smoke.py`: smoke screenshots the
window and needs an X server that can export the root window. The bound is
kept -- the snippet closes its own window, and an app that never closes is a
hang, which is a failure.
"""

import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent

# Prepended to every snippet: close the window after a fixed number of frames,
# so a snippet is a bounded app rather than something that hangs on a typo.
BOUNDED_RUNNER = '''
import pyegui as _pyegui
import time as _time

_frames = [0]
_deadline = _time.monotonic() + 1.0
_original_run_native = _pyegui.run_native


def _bounded(app_name, update_func, **kwargs):
    def update(ctx):
        update_func(ctx)
        _frames[0] += 1
        ctx.request_repaint()
        if _time.monotonic() >= _deadline:
            ctx.close()
    return _original_run_native(app_name, update, **kwargs)


_pyegui.run_native = _bounded
'''


def run_snippet(source, timeout=180):
    """Execute `source` as a self-closing app; return (ok, detail)."""
    runner = BOUNDED_RUNNER + source
    proc = subprocess.run(
        [sys.executable, "-c", runner],
        capture_output=True,
        text=True,
        timeout=timeout,
    )
    lines_out = (proc.stdout + "\n" + proc.stderr).strip().splitlines()
    interesting = [ln for ln in lines_out if not ln.startswith("xkbcommon:")]
    return proc.returncode == 0, "\n".join(interesting[-14:]) or "no output"
