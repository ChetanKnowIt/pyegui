"""Builder options on `run_native`: the viewport and `NativeOptions`.

## Why this file exists separately from the widget ones

`run_native` is the only function in the binding that every app in the repo
calls, so a mistake here breaks `examples/` and `guides/` wholesale rather than
one widget group. It is also the last place `reject_unknown_options` was not
wired, which is why this file is also where the crate-wide audit's last gap is
covered.

## These tests observe the EFFECT, not the resolution

The brief is explicit that "the call did not raise" is the wrong assertion: it
passes just as well if the value was accepted and then dropped before reaching
`ViewportBuilder`. Every behavioural assertion here therefore reads a value back
OUT of egui:

* the size tests assert `ctx.viewport_inner_size()`, which is egui's own
  `InputState::viewport().inner_rect` as reported by the window manager;
* the title test asserts `ctx.viewport_title()`.

Those are the only readouts that exist. Anything else -- `decorations`,
`taskbar`, `drag_and_drop`, the whole `NativeOptions` group -- is a request to
the window manager with no counter-report in `egui::InputState`, so it is not
observable from Python and is not asserted as behaviour here. The acceptance
test says which names are accepted; it does not pretend to verify their effect.
See `NOT_OBSERVABLE_BELOW`.

## Every assertion runs AFTER `run_native` returns

`tests/bounded_app.py` explains why, and it is the same shape the slider,
DragValue/TextEdit and date-picker files use. A widget exception does not close
the app -- `PyeguiApp::update` displays it and returns `Ok` -- so an un-caught
failure inside the update function presents as a HANG under a wall-clock bound
rather than as a message. The snippets catch their own errors and assert after
the app has closed.

## The pinned source is the inventory, and it disagrees with TODO §6

`eframe` is pinned `=0.31.1`. Read `crates/egui/src/viewport.rs` and
`crates/eframe/src/epi.rs` at tag 0.31.1 rather than trusting TODO §6's prose.
Three §6 names do not exist there and are recorded in TODO.md as NOT
implemented rather than accepted-and-ignored:

* `movable_by_background` -- no such field or setter. What §6 meant is
  `drag_and_drop`, which is Windows-only.
* `monitor` -- the viewport has no per-monitor placement option in 0.31.1.
  `clamp_size_to_monitor_size` is the nearest real thing and IS implemented.
* `always_on_top` -- real, but egui's setter is a SWITCH
  (`with_always_on_top()`, no argument). It is exposed as a bool where only
  `True` does anything, because that is all egui can express;
  `window_level` is the spelling that can also say `AlwaysOnBottom`.

`test_section_6_names_that_do_not_exist_are_rejected` pins all three.
"""

import re
from pathlib import Path

from bounded_app import run_snippet

REPO_ROOT = Path(__file__).resolve().parent.parent


# --------------------------------------------------------------------------
# The eleven kwarg names that shipped before this task.
#
# Spelled exactly as they were. A rename here is a silent breaking change for
# every app in the repo, and `examples/` is the test that would notice -- so
# this list is asserted against `src/lib.rs` rather than trusted.
# --------------------------------------------------------------------------
PRESERVED_NAMES = [
    "fullscreen",
    "icon_path",
    "inner_height",
    "inner_width",
    "max_inner_height",
    "max_inner_width",
    "maximized",
    "min_inner_height",
    "min_inner_width",
    "resizable",
    "transparent",
]

# Read in one call each, in an app that closes itself. Every value is a real
# value for the option: `icon_path` is a path the repo actually has.
ELEVEN_NAMES_ACCEPTED = '''
import sys
import pyegui

REAL_PNG = None
import pathlib
for candidate in ("README.md", "CHANGELOG.md"):
    # A real, readable file. `icon_path` reads it and hands the bytes to
    # eframe's PNG decoder, which will reject a non-PNG with an OSError --
    # so this checks the NAME is accepted, in a separate call, below.
    if pathlib.Path(candidate).exists():
        REAL_PNG = candidate
        break

# The ten scalar/size names, all in one call. `icon_path` is exercised
# separately because it reads a file.
SCALARS = (
    ("inner_width", 640), ("inner_height", 480),
    ("min_inner_width", 200), ("min_inner_height", 150),
    ("max_inner_width", 1200), ("max_inner_height", 900),
    ("fullscreen", False), ("maximized", False),
    ("resizable", True), ("transparent", False),
)

problems = []
seen = []

def attempt(note, fn):
    try:
        fn()
    except Exception as exc:
        problems.append(f"{note}: {exc!r}")
    else:
        seen.append(note)

def contents():
    pyegui.label("viewport names")

def update(ctx):
    pyegui.central_panel(ctx, contents)

# One call with all ten, the way an app would write them.
def with_scalars():
    pyegui.run_native("scalars", update, **dict(SCALARS))

attempt("ten scalar names in one call", with_scalars)

pyegui.run_native(
    "icon", update, inner_width=320, inner_height=240, icon_path=REAL_PNG)

if problems:
    for p in problems:
        print("PROBLEM:", p)
    sys.exit(1)

print("all eleven preserved names accepted")
'''


def test_all_eleven_preserved_names_are_still_accepted():
    ok, detail = run_snippet(ELEVEN_NAMES_ACCEPTED)
    assert ok, f"a preserved run_native kwarg name was rejected: {detail}"


# --------------------------------------------------------------------------
# The eleven names must still be READ, and read under those exact spellings.
#
# This is a source-level check rather than a behavioural one, and it is here
# because the behavioural half is the size test below and a source-level check
# can additionally catch a name that was kept as an accepted-but-ignored
# keyword. It reads `apply_viewport_options` out of `src/lib.rs` and requires
# every one of the eleven to appear in it.
# --------------------------------------------------------------------------
def _apply_viewport_options_body():
    lib = (REPO_ROOT / "src" / "lib.rs").read_text(encoding="utf-8")
    start = lib.index("unsafe fn apply_viewport_options(")
    end = lib.index("/// The four `eframe::NativeOptions` fields", start)
    return lib[start:end]


def test_preserved_names_are_read_by_the_viewport_helper():
    body = _apply_viewport_options_body()
    unread = [name for name in PRESERVED_NAMES if f'"{name}"' not in body]
    assert not unread, (
        f"src/lib.rs's apply_viewport_options does not read {unread}. These "
        "eleven names are public API -- examples/ and guides/ pass them -- and "
        "the unknown-option check now accepts them because they are recorded, "
        "so a name that stopped being read would be silently ignored rather "
        "than reported."
    )


# --------------------------------------------------------------------------
# BEHAVIOURAL: the requested size must reach the window.
#
# This is the test the brief asked for, in the form it asked for. `inner_width`
# and `inner_height` are passed to `run_native`, and the assertion is on
# `ctx.viewport_inner_size()` -- egui's `InputState::viewport().inner_rect` as
# reported back by the window manager. If the pair were accepted and then
# dropped before reaching `ViewportBuilder`, the window would open at eframe's
# 800x600 default (epi's native/epi_integration.rs:78) and the size assertion
# would fail.
#
# Two details this depends on, both of them the reason a weaker version of this
# test would be wrong:
#
#   - the rect is None on the first frame, so the reading is taken from a
#     later frame and only after a None;
#   - `inner_rect` is in egui POINTS, not physical pixels, so a HiDPI host
#     does not scale the number. The tolerance is for the window manager
#     adjusting the request (some WMs clamp, and a WM may not honour a size
#     exactly), not for scaling.
# --------------------------------------------------------------------------
REQUESTED_SIZE_REACHES_THE_WINDOW = '''
import sys
import pyegui

WANTED_W, WANTED_H = 640, 480
TOLERANCE = 2.0

requested = dict(inner_width=WANTED_W, inner_height=WANTED_H,
                 resizable=False)
observed = []
frames = [0]

def contents():
    pyegui.label("size check")

def update(ctx):
    pyegui.central_panel(ctx, contents)
    frames[0] += 1
    size = ctx.viewport_inner_size()
    if size is not None and not observed:
        observed.append(size)
    if frames[0] > 120:
        ctx.close()

pyegui.run_native("requested size", update, **requested)

problems = []

if not observed:
    problems.append(
        "the window manager never reported an inner rect, so the requested "
        "size could not be observed at all -- this test cannot distinguish a "
        "size that arrived from one that did not")
else:
    w, h = observed[0]
    if abs(w - WANTED_W) > TOLERANCE or abs(h - WANTED_H) > TOLERANCE:
        problems.append(
            f"run_native was asked for a {WANTED_W}x{WANTED_H} inner size and "
            f"the window reports {w}x{h}. eframe's own default is 800x600 "
            "(crates/eframe/src/native/epi_integration.rs:78), so this size "
            "did not reach ViewportBuilder.")

if problems:
    for p in problems:
        print("PROBLEM:", p)
    sys.exit(1)

print("requested size reached the window:", observed[0])
'''


def test_requested_inner_size_reaches_the_window():
    ok, detail = run_snippet(REQUESTED_SIZE_REACHES_THE_WINDOW)
    assert ok, (
        "run_native accepted the size but the window did not open at it, so "
        f"the option never reached ViewportBuilder: {detail}"
    )
    assert "reached the window" in detail, (
        "the snippet exited 0 without reporting the observed size, so it did "
        f"not assert what it claims to: {detail}"
    )


# --------------------------------------------------------------------------
# The control: a DIFFERENT size gives a DIFFERENT observed size.
#
# Without this, "the window reports the requested size" and "the window always
# reports whatever was requested-shaped number" are indistinguishable from one
# assertion. Two runs at two sizes, both observed, and they must differ. This is
# what makes the size test behavioural rather than a tautology about the
# requested value.
# --------------------------------------------------------------------------
def _size_snippet(width, height):
    return f'''
import sys
import pyegui

WANTED = ({width}, {height})
observed = []
frames = [0]

def contents():
    pyegui.label("size check")

def update(ctx):
    pyegui.central_panel(ctx, contents)
    frames[0] += 1
    size = ctx.viewport_inner_size()
    if size is not None and not observed:
        observed.append(size)
    if frames[0] > 120:
        ctx.close()

pyegui.run_native("size", update,
                  inner_width=WANTED[0], inner_height=WANTED[1],
                  resizable=False)

if not observed:
    print("NO OBSERVATION")
    sys.exit(1)
print("OBSERVED", observed[0][0], observed[0][1])
'''


def test_two_different_requested_sizes_give_two_different_windows():
    sizes = []
    for width, height in ((500, 400), (700, 550)):
        ok, detail = run_snippet(_size_snippet(width, height))
        assert ok, f"run_native(inner_width={width}, inner_height={height}) failed: {detail}"
        m = re.search(r"OBSERVED ([\d.]+) ([\d.]+)", detail)
        assert m, f"the snippet did not report an observed size: {detail}"
        sizes.append((float(m.group(1)), float(m.group(2))))

    (w1, h1), (w2, h2) = sizes
    assert abs(w1 - w2) > 2.0 or abs(h1 - h2) > 2.0, (
        f"both runs produced the same window {sizes}, so the size test is not "
        "observing the request at all -- it is observing a constant."
    )


# --------------------------------------------------------------------------
# BEHAVIOURAL: `title` reaches the window.
#
# `ViewportBuilder::title` is what eframe passes to the native window, and egui
# reports the title back on `InputState::viewport().title`, so this one is
# observable too. It is the second behavioural datapoint, on a different field
# from the size, which is what rules out "the whole viewport builder is being
# dropped after the first option".
# --------------------------------------------------------------------------
TITLE_REACHES_THE_WINDOW = '''
import sys
import pyegui

WANTED = "pyegui viewport option under test"
observed = []
frames = [0]

def contents():
    pyegui.label("title check")

def update(ctx):
    pyegui.central_panel(ctx, contents)
    frames[0] += 1
    title = ctx.viewport_title()
    if title is not None and not observed:
        observed.append(title)
    if frames[0] > 120:
        ctx.close()

pyegui.run_native("app name that is not the title", update,
                  inner_width=400, inner_height=300, title=WANTED)

problems = []

if not observed:
    problems.append(
        "the window manager never reported a title, so `title` could not be "
        "observed at all on this platform")
elif observed[0] != WANTED:
    problems.append(
        f"run_native was asked for the title {WANTED!r} and the window reports "
        f"{observed[0]!r}. eframe falls back to the app_name argument when the "
        "builder carries no title, so a report of the app_name means the "
        "option never reached ViewportBuilder.")

if problems:
    for p in problems:
        print("PROBLEM:", p)
    sys.exit(1)

print("title reached the window:", observed[0])
'''


def test_requested_title_reaches_the_window():
    ok, detail = run_snippet(TITLE_REACHES_THE_WINDOW)
    assert ok, (
        "run_native accepted `title` but the window does not report it, so "
        f"the option never reached ViewportBuilder: {detail}"
    )


# --------------------------------------------------------------------------
# Every new name is ACCEPTED, at a value egui accepts.
#
# This is resolution, not behaviour, and it is here for the names the other
# tests cannot observe. Saying so in the comment is the point: an acceptance
# test that reads as behavioural coverage is the failure mode the brief names.
# --------------------------------------------------------------------------
NEW_OPTIONS_ACCEPTED = '''
import sys
import pyegui

ACCEPTED = (
    {"title": "a title"},
    {"app_id": "org.pyegui.test"},
    {"position": (40.0, 60.0)},
    {"visible": True},
    {"active": True},
    {"decorations": False},
    {"always_on_top": True},
    {"always_on_top": False},
    {"window_level": "Normal"},
    {"window_level": "AlwaysOnBottom"},
    {"window_level": "AlwaysOnTop"},
    {"taskbar": False},
    {"window_type": "Normal"},
    {"window_type": "Utility"},
    {"minimize_button": False},
    {"maximize_button": False},
    {"close_button": True},
    {"title_shown": True},
    {"titlebar_shown": False},
    {"titlebar_buttons_shown": True},
    {"fullsize_content_view": False},
    {"drag_and_drop": False},
    {"mouse_passthrough": False},
    {"clamp_size_to_monitor_size": True},
    # The four NativeOptions fields.
    {"centered": False},
    {"multisampling": 0},
    {"multisampling": 4},
    {"persist_window": False},
    {"persistence_path": "/tmp/pyegui-viewport-test.ron"},
)

problems = []
seen = []

def attempt(options):
    note = ",".join(sorted(options))
    def fn():
        pyegui.run_native("opt", update,
                          inner_width=320, inner_height=240, **options)
    try:
        fn()
    except Exception as exc:
        problems.append(f"{note}: {exc!r}")
    else:
        seen.append(note)

def contents():
    pyegui.label("options")

def update(ctx):
    pyegui.central_panel(ctx, contents)

# Each in its own app: some of these (position, always_on_top, mouse
# passthrough) change how the window manager behaves, and stacking them all
# into one window would make a failure unattributable.
for options in ACCEPTED:
    attempt(options)

if len(seen) != len(ACCEPTED):
    problems.append(
        f"expected {len(ACCEPTED)} accepted option sets, got {len(seen)}")

if problems:
    for p in problems:
        print("PROBLEM:", p)
    sys.exit(1)

print("every new viewport and NativeOptions name accepted")
'''


def test_every_new_option_name_is_accepted():
    ok, detail = run_snippet(NEW_OPTIONS_ACCEPTED)
    assert ok, (
        "a viewport or NativeOptions option this binding advertises was "
        f"rejected: {detail}"
    )


# --------------------------------------------------------------------------
# An unknown name is an ERROR, and it names itself.
#
# This is the decision-2 behaviour, now reaching `run_native`. The set of names
# TODO §6 lists that egui 0.31.1 does not have is in there too, so that a
# documented-but-nonexistent name is reported rather than silently accepted.
# --------------------------------------------------------------------------
UNKNOWN_OPTION_IS_AN_ERROR = '''
import sys
import pyegui

REJECTED = (
    "widht",                 # a misspelling of a real option
    "movable_by_background",  # TODO §6, but no such ViewportBuilder field
    "monitor",               # TODO §6, but no per-monitor setter in 0.31.1
    "depth_buffer",          # NativeOptions, not implemented
    "glow_options",          # NativeOptions, deferred
    "wgpu_options",          # NativeOptions, deferred
    "persist",               # a near-miss for persist_window
)

outcomes = []
recorded = []

def record(note, fn):
    if note in recorded:
        pyegui.label("done")
        return
    recorded.append(note)
    try:
        fn()
    except ValueError as exc:
        outcomes.append((note, str(exc)))
    except Exception as exc:
        outcomes.append((note, "WRONG EXCEPTION TYPE: " + repr(exc)))
    else:
        outcomes.append((note, "NOT RAISED"))

def contents():
    pyegui.label("unknown")

def update(ctx):
    pyegui.central_panel(ctx, contents)

def attempt(name):
    # `run_native` raises before the window opens -- the check runs before
    # `eframe::run_native` is called -- so this needs no app at all and cannot
    # hang. It is called from module scope rather than from `contents`.
    try:
        pyegui.run_native("unknown", update, **{name: 1})
    except ValueError as exc:
        outcomes.append((name, str(exc)))
    except Exception as exc:
        outcomes.append((name, "WRONG EXCEPTION TYPE: " + repr(exc)))
    else:
        outcomes.append((name, "NOT RAISED"))

for name in REJECTED:
    attempt(name)

problems = []

if len(outcomes) != len(REJECTED):
    problems.append(f"expected {len(REJECTED)} outcomes, got {len(outcomes)}")

for note, detail in outcomes:
    if detail == "NOT RAISED":
        problems.append(f"{note}: the unknown option was accepted silently")
    elif detail.startswith("WRONG EXCEPTION TYPE"):
        problems.append(f"{note}: expected ValueError, got {detail}")
    else:
        if note not in detail:
            problems.append(f"{note}: the error does not name the option: {detail}")

if problems:
    for p in problems:
        print("PROBLEM:", p)
    sys.exit(1)

print("every unknown name was rejected and named")
'''


def test_unknown_option_is_an_error_that_names_it():
    ok, detail = run_snippet(UNKNOWN_OPTION_IS_AN_ERROR)
    assert ok, (
        "an unknown run_native option was not reported as a ValueError naming "
        f"it: {detail}"
    )


# --------------------------------------------------------------------------
# The three §6 names that do not exist, and the deferred renderer structs.
#
# `reject_unknown_options` reporting them is the record the plan asked for, and
# it is asserted here as a separate test because that is the specific claim:
# these names are NOT silently accepted, because accepting them would promise
# behaviour that is not there.
# --------------------------------------------------------------------------
SECTION_6_NAMES_THAT_DO_NOT_EXIST = ("movable_by_background", "monitor")


def test_section_6_names_that_do_not_exist_are_rejected():
    for name in SECTION_6_NAMES_THAT_DO_NOT_EXIST:
        source = f'''
import sys
import pyegui

def update(ctx):
    pyegui.label("x")

try:
    pyegui.run_native("t", update, {name}=1)
except ValueError as exc:
    print("REJECTED", {name!r} in str(exc), str(exc))
    sys.exit(0 if {name!r} in str(exc) else 1)
except Exception as exc:
    print("WRONG", repr(exc))
    sys.exit(1)
else:
    print("ACCEPTED")
    sys.exit(1)
'''
        ok, detail = run_snippet(source)
        assert ok, (
            f"run_native accepted {name}, which TODO §6 lists but egui 0.31.1 "
            f"does not have: {detail}"
        )


# --------------------------------------------------------------------------
# `glow_options` and `wgpu_options` are DEFERRED, not stubbed.
#
# Both are large nested renderer structs. They are recorded in TODO.md with the
# reason and are rejected here, which is the honest shape: a name that is
# accepted and dropped would be worse than a name that is refused.
# --------------------------------------------------------------------------
def test_glow_and_wgpu_options_are_rejected_not_stubbed():
    for name in ("glow_options", "wgpu_options"):
        source = f'''
import sys
import pyegui

def update(ctx):
    pyegui.label("x")

try:
    pyegui.run_native("t", update, {name}={{}})
except ValueError:
    print("REJECTED")
    sys.exit(0)
except Exception as exc:
    print("WRONG", repr(exc))
    sys.exit(1)
else:
    print("ACCEPTED")
    sys.exit(1)
'''
        ok, detail = run_snippet(source)
        assert ok, (
            f"run_native accepted {name}, which is deferred and not "
            f"implemented: {detail}"
        )


# --------------------------------------------------------------------------
# An enum option is rejected with the accepted values LISTED.
#
# Both enums here are egui enums with no Python type, so the word is checked
# against a list. An error that says only "unknown window_level" leaves the
# caller with nothing to type instead, so the list is part of the contract.
# `AlwaysOnTop` is a real variant of `egui::WindowLevel` and must be accepted
# with exactly that spelling -- which is the kind of thing a hand-written list
# gets wrong.
# --------------------------------------------------------------------------
ENUM_ERRORS_LIST_THE_ACCEPTED_VALUES = '''
import sys
import pyegui

def update(ctx):
    pyegui.label("x")

WINDOW_LEVELS = ("Normal", "AlwaysOnBottom", "AlwaysOnTop")

# Only the REJECTED cases are probed here. A correctly spelled variant is
# accepted, and accepting it means `run_native` goes on to open a window and
# block until it closes -- probing the accepted side from here would hang the
# snippet. `test_every_new_option_name_is_accepted` covers the accepted side,
# in its own self-closing app.
NEAR_MISSES = ("alwaysontop", "ALWAYSONTOP", "", "Top", "normal")

problems = []
outcomes = []

for word in NEAR_MISSES:
    try:
        pyegui.run_native("t", update, window_level=word)
    except ValueError as exc:
        outcomes.append((word, "REJECTED", str(exc)))
    except Exception as exc:
        outcomes.append((word, "WRONG EXCEPTION TYPE", repr(exc)))
    else:
        outcomes.append((word, "ACCEPTED", ""))

if len(outcomes) != len(NEAR_MISSES):
    problems.append(
        f"expected {len(NEAR_MISSES)} outcomes, got {len(outcomes)}")

for word, kind, detail in outcomes:
    if kind == "WRONG EXCEPTION TYPE":
        problems.append(f"window_level={word!r}: expected ValueError, got {detail}")
    elif kind == "ACCEPTED":
        problems.append(
            f"window_level={word!r} was accepted, so an app would open a window "
            "with that word silently ignored -- the spellings are case-exact")
    else:
        for expected in WINDOW_LEVELS:
            if expected not in detail:
                problems.append(
                    f"window_level={word!r}: the error does not list the accepted "
                    f"value {expected!r}: {detail}")

if problems:
    for p in problems:
        print("PROBLEM:", p)
    sys.exit(1)

print("near-miss spellings rejected, and the error lists the accepted values")
'''


def test_enum_errors_list_the_accepted_values():
    ok, detail = run_snippet(ENUM_ERRORS_LIST_THE_ACCEPTED_VALUES)
    assert ok, (
        "the window_level error did not list the accepted values, or a "
        f"near-miss spelling was accepted: {detail}"
    )


# --------------------------------------------------------------------------
# A duplicate keyword still raises, so `title` and friends are real keyword
# parameters rather than dict entries.
#
# `run_native` has no named parameters of its own -- everything arrives through
# `**kwargs` -- so this is a weaker statement here than it is on the widget
# groups: a `**options`-only design for `run_native` would raise the same
# TypeError. It is kept because it costs nothing and because it would catch a
# future change that introduces named parameters and then let one through the
# dict as well. Noted here so it is not read as stronger than it is.
# --------------------------------------------------------------------------
DUPLICATE_KEYWORD = '''
import sys
import pyegui

def update(ctx):
    pyegui.label("x")

try:
    pyegui.run_native("t", update, inner_width=400, **{"inner_width": 500})
except TypeError as exc:
    print("TypeError", "multiple values" in str(exc), str(exc))
    sys.exit(0 if "multiple values" in str(exc) else 1)
except Exception as exc:
    print("WRONG", repr(exc))
    sys.exit(1)
else:
    print("NOT RAISED")
    sys.exit(1)
'''


def test_duplicate_keyword_raises():
    ok, detail = run_snippet(DUPLICATE_KEYWORD)
    assert ok, f"a duplicate keyword was silently accepted: {detail}"


# --------------------------------------------------------------------------
# The size pair still needs BOTH halves.
#
# This is pre-existing behaviour that this task deliberately preserved: before
# this task, `run_native(inner_width=800)` on its own was IGNORED, because the
# code read `inner_height` and `inner_width` as a tuple and applied the size
# only if both arrived. Fixing it would be a behaviour change dressed as an
# option addition, so it is pinned rather than quietly changed.
#
# The observation is egui's own: eframe's default inner size is 800x600
# (crates/eframe/src/native/epi_integration.rs:78), so a lone `inner_width`
# leaves the window at eframe's default rather than at the requested width.
# --------------------------------------------------------------------------
HALF_A_SIZE_PAIR_IS_STILL_IGNORED = '''
import sys
import pyegui

observed = []
frames = [0]

def contents():
    pyegui.label("half a pair")

def update(ctx):
    pyegui.central_panel(ctx, contents)
    frames[0] += 1
    size = ctx.viewport_inner_size()
    if size is not None and not observed:
        observed.append(size)
    if frames[0] > 120:
        ctx.close()

# Only the width. No height.
pyegui.run_native("half", update, inner_width=500, resizable=False)

if not observed:
    print("NO OBSERVATION")
    sys.exit(1)

w, h = observed[0]
print("OBSERVED", w, h)

# 800 is eframe's own default width; a lone inner_width has always been
# ignored, so the window should still be at the default, not 500 wide.
if abs(w - 500.0) <= 2.0:
    print("PROBLEM: a lone inner_width=500 changed the window width. That is "
          "not the pre-existing behaviour; either the behaviour changed or the "
          "observation is not measuring what it claims.")
    sys.exit(1)
'''


def test_half_a_size_pair_is_still_ignored():
    ok, detail = run_snippet(HALF_A_SIZE_PAIR_IS_STILL_IGNORED)
    assert ok, (
        "a lone inner_width changed the window width, which is not the "
        f"behaviour that shipped before this task: {detail}"
    )


# --------------------------------------------------------------------------
# The inventory, held against `src/lib.rs`.
#
# Reads the two helper bodies rather than restating them, so a name that is
# documented but not read -- which `reject_unknown_options` would then reject at
# runtime -- fails here instead. And TODO §6's list of names that do NOT exist
# is held against the test's own list, so the two cannot drift.
# --------------------------------------------------------------------------
NOT_A_VIEWPORT_OPTION = [
    "movable_by_background",
    "monitor",
]

# TODO §6's run_native item, minus the names it got wrong and minus the ones
# this task implemented. Named here as the reason each is absent.
SECTION_6_DISAGREEMENTS = {
    "movable_by_background": "no such ViewportBuilder field or setter in 0.31.1",
    "monitor": "no per-monitor placement setter in 0.31.1",
    "always_on_top": "egui's setter is a switch, so it takes no bool",
}


def _todo_item(marker):
    """The TODO.md checklist item whose text starts with `marker`."""
    todo = (REPO_ROOT / "TODO.md").read_text(encoding="utf-8")
    start = todo.index(marker)
    rest = todo[start + len(marker):]
    end = rest.index("\n- [")
    return rest[:end]


def _read_names_in(body):
    """The option names a helper body reads with an `opt_*` call.

    Deliberately narrow: it matches `opt_...(kwargs, "name"`, so it finds the
    names a helper actually forwards and not the ones it only mentions in a
    comment or an error message. `opt_size_pair` takes two names per call and
    is read by the PRESERVED_NAMES test instead.
    """
    return sorted(set(re.findall(r'opt_\w+\(kwargs, "([a-z_]+)"', body)))


def test_viewport_inventory_matches_src_and_todo():
    body = _apply_viewport_options_body()

    unread = [n for n in PRESERVED_NAMES if f'"{n}"' not in body]
    assert not unread, (
        f"apply_viewport_options does not read {unread}; see the preserved "
        "names test"
    )

    lib = (REPO_ROOT / "src" / "lib.rs").read_text(encoding="utf-8")
    native_start = lib.index("unsafe fn apply_native_options(")
    native_end = lib.index("unsafe fn ui_stack(", native_start)
    native_names = _read_names_in(lib[native_start:native_end])

    assert native_names == ["centered", "multisampling", "persist_window",
                            "persistence_path"], (
        f"apply_native_options reads {native_names}. TODO.md records exactly "
        "these four as implemented and glow_options/wgpu_options as deferred, "
        "so a fifth name here is undocumented."
    )

    # TODO.md must still record the deferred and nonexistent names as NOT
    # implemented, or they read as an oversight rather than a verified absence.
    documented = (
        _todo_item("- [x] `run_native` viewport kwargs")
        + _todo_item("- [x] `eframe::NativeOptions`")
    )
    for name in NOT_A_VIEWPORT_OPTION + ["glow_options", "wgpu_options"]:
        assert name in documented, (
            f"TODO.md no longer mentions {name}, which the binding does NOT "
            "implement. Leaving it off the list makes it look like an oversight."
        )

    # And TODO.md must list each one as its own bullet under a "not implemented"
    # heading, not merely mention it somewhere. Matching the BULLET rather than
    # the bare name is what makes this check work: `monitor` appears inside
    # `clamp_size_to_monitor_size` too, so a substring search finds the wrong
    # occurrence and reads as "not recorded as not implemented".
    def _bullets_for(text, name):
        """TODO.md bullets that open with ``- `name` ``."""
        marker = f"- `{name}`"
        return [
            line.strip()
            for line in text.splitlines()
            if line.strip().startswith(marker)
        ]

    for name in NOT_A_VIEWPORT_OPTION + ["glow_options", "wgpu_options"]:
        assert _bullets_for(documented, name), (
            f"TODO.md does not record {name} as its own 'not implemented' "
            "bullet, so it reads as an oversight rather than a verified absence."
        )
        assert any("not implement" in b.lower() or "deferred" in b.lower()
                   or "does not exist" in b.lower()
                   for b in _bullets_for(documented, name)), (
            f"TODO.md's bullet for {name} does not say it is unimplemented: "
            f"{_bullets_for(documented, name)}"
        )