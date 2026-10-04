"""Guard for the consumed-key tracker that `reject_unknown_options` depends on.

## What the tracker is, and why `get_item` was kept

`opt_*` helpers record the option names they read in an `OptNames` set instead
of removing the key from the dict, so that a caller who builds one options dict
and passes it to two widgets still has both calls see every key. All 25 widget
signatures take a shared `Option<&Bound<'_, PyDict>>`, which is why that
compiles at all: `remove_item` needs `&mut PyDict`, so consuming keys would have
meant changing every one of those 25 signatures to a distinct mutable borrow.

**That compile-time shape, not non-mutation, is the justification.** The two
things are usually confused, so to be explicit:

- **No Python test can witness non-mutation.** CPython's `f(**d)` always
  materialises a fresh dict for the callee's `**kwargs`, and pyo3 0.24's
  `handle_varkeyword` builds its own dict with
  `get_or_insert_with(|| PyDict::new(...))`. So the callee never receives the
  caller's dict object at all; clearing `**kwargs` on exit could not reach `d`.
  An earlier version of this file asserted the opposite with two
  dict-reuse snippets -- those tests passed whether the helpers called
  `get_item` or `remove_item`, so they were deleted rather than fixed.
- **The direct witness would be a Rust unit test** of `opt_bool`/`opt_f32`:
  build a dict, call the helper twice for one key, assert the key is still
  present and the second call returns the value rather than `None`. That test
  fails under `remove_item` and passes today, so it is the shape worth having
  if this repo ever gains a Rust test target. It is *not* here because `cargo
  test` is not part of any CI job here -- `check` runs `cargo check
  --all-targets`, which compiles test code but never executes it, so the test
  would ship unrun and unverified. Asserting an unrun test passes is worse
  than saying plainly that the property is not covered at runtime.

## What the remaining test does and does not prove

`reject_unknown_options` is not called by any widget yet (the six §6 targets
arrive in later tasks), so nothing can currently observe a false rejection.
The test below is the guard that has to be satisfied the moment `frame` and
`window` do call the check: it passes each of them an option it genuinely
supports, read inside a helper (`apply_frame_options` / `apply_window_options`)
rather than in the pyfunction. A helper that reads options without threading
`used` through makes a genuine option look unknown, and that is what fails
here.

It runs its snippet through `smoke.py --run-source`, the same harness the
examples job uses, because these widgets only draw inside a real egui frame.
A snippet that fails to raise exits 0.

One property of that harness to carry forward: an exception raised inside a
widget's draw never returns to the harness's `ctx.close()`, so it shows up as a
120s timeout rather than as an error message naming the option. Widgets that
need a `Ui` must therefore be drawn inside a container callback, or the test
hangs instead of failing.
"""

import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
SMOKE = REPO_ROOT / "tests" / "smoke.py"


def run_snippet(source):
    """Execute `source` as a bounded app; return (ok, detail)."""
    proc = subprocess.run(
        [sys.executable, str(SMOKE), "--run-source"],
        input=source,
        capture_output=True,
        text=True,
        timeout=120,
    )
    detail = (proc.stderr or proc.stdout).strip().splitlines()
    return proc.returncode == 0, detail[-1] if detail else "no output"


# Delegated option paths: both `frame` and `window` read their options in a
# helper (`apply_frame_options` / `apply_window_options`) rather than in the
# pyfunction. Each is passed an option it genuinely supports, so the day these
# two start calling `reject_unknown_options`, a helper that forgets to pass
# `used` through fails here instead of in a user's app.
#
# `frame` is not a top-level container, so it needs a Ui to draw into; both
# live under `central_panel`'s callback.
DELEGATED_OPTIONS = '''
import pyegui

def contents():
    pyegui.label("inside a frame")
    pyegui.label("inside a window")

def main():
    # frame is not a top-level container, so it needs a Ui to draw into.
    pyegui.frame(contents, inner_margin=8, corner_radius=4)

def update(ctx):
    pyegui.window(ctx, "Settings", "settings", contents,
                  default_size=(320.0, 240.0), resizable=True)
    pyegui.central_panel(ctx, main)

pyegui.run_native("delegated", update)
'''


def test_delegated_option_paths_do_not_raise():
    ok, detail = run_snippet(DELEGATED_OPTIONS)
    assert ok, (
        "frame and window read their options in a helper; passing an option "
        f"each genuinely supports raised: {detail}"
    )