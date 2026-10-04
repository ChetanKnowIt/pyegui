"""Prove the consumed-key tracker never mutates the caller's options dict.

The option helpers record what they read in an `OptNames` set rather than
removing the key from the dict. That choice is only worth anything if the
alternative really would break someone: a caller who builds one options dict and
passes it to two widgets must have both calls see every key. `remove_item` would
have made the second call see an empty dict, which `validate_options` then
reports as silently dropping the options.

`reject_unknown_options` is not called by any widget yet (the six §6 targets
arrive in later tasks), so the delegated-path test here cannot yet catch a
supported option being falsely rejected. It is here as the guard that has to be
satisfied the moment `frame` and `window` do call the check -- a helper that
reads options without threading `used` through makes a genuine option look
unknown, and this is the test that would say so.

Both tests run their snippet through `smoke.py --run-source`, the same harness
the examples job uses, because these widgets only draw inside a real egui frame.
A snippet that fails to raise exits 0; a snippet whose options got swallowed
raises and exits non-zero.
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


# One dict, two calls. `window` reads its options through
# `apply_window_options`, which is where the helpers record names -- so if the
# recording were done by consuming the keys, the second window would get an
# options dict with `default_size` already gone. `validate_options` would still
# pass (it only checks for *unknown* keys), which is exactly why the silent
# version of this bug needs a test rather than an existing gate.
REUSED_DICT = '''
import pyegui

shared = {"default_size": (320.0, 240.0)}

def first():
    pyegui.label("first")

def second():
    pyegui.label("second")

def update(ctx):
    pyegui.window(ctx, "one", "one", first, **shared)
    pyegui.window(ctx, "two", "two", second, **shared)
    pyegui.central_panel(ctx, lambda: pyegui.heading("reused dict"))

pyegui.run_native("reused", update)
'''

# `text_edit_singleline` reads its one option straight off the dict, so it is
# the simplest witness that the helpers' presence did not change dict handling
# for a widget that shares the same kwargs shape.
REUSED_TEXT_KWARGS = '''
import pyegui

shared = {"hint_text": "name"}

def update(ctx):
    a = pyegui.Str("a")
    b = pyegui.Str("b")
    pyegui.text_edit_singleline(a, **shared)
    # Must still see hint_text. A consumed key here would leave this field with
    # no placeholder at all.
    pyegui.text_edit_singleline(b, **shared)
    pyegui.central_panel(ctx, lambda: pyegui.heading("reused kwargs"))

pyegui.run_native("reused kwargs", update)
'''

# Delegated option paths: both `frame` and `window` read their options in a
# helper (`apply_frame_options` / `apply_window_options`) rather than in the
# pyfunction itself. Each is passed an option it genuinely supports, so the day
# these two start calling `reject_unknown_options`, a helper that forgets to
# pass `used` through fails here instead of in a user's app.
DELEGATED_OPTIONS = '''
import pyegui

def contents():
    pyegui.label("inside a frame")
    pyegui.label("inside a window")

def update(ctx):
    pyegui.frame(contents, inner_margin=8, corner_radius=4)
    pyegui.window(ctx, "Settings", "settings", contents,
                  default_size=(320.0, 240.0), resizable=True)
    pyegui.central_panel(ctx, lambda: pyegui.heading("delegated options"))

pyegui.run_native("delegated", update)
'''


def test_one_options_dict_survives_two_windows():
    ok, detail = run_snippet(REUSED_DICT)
    assert ok, (
        "passing one options dict to two windows did not work -- the first "
        f"window appears to have consumed its keys: {detail}"
    )


def test_one_kwargs_dict_survives_two_text_edits():
    ok, detail = run_snippet(REUSED_TEXT_KWARGS)
    assert ok, (
        "text_edit_singleline appears to consume its kwargs: the second call "
        f"did not see hint_text: {detail}"
    )


def test_delegated_option_paths_do_not_raise():
    ok, detail = run_snippet(DELEGATED_OPTIONS)
    assert ok, (
        "frame and window read their options in a helper; passing an option "
        f"each genuinely supports raised: {detail}"
    )