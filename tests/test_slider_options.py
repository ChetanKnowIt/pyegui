"""Builder options on the slider group, and the first `reject_unknown_options` call.

## Every assertion here runs AFTER `run_native` returns, not inside the callback

That is deliberate, and it is the shape that makes this file able to fail.

An exception raised inside a widget's draw does not close the app:
`PyeguiApp::update` does
`if let Err(err) = self.update_func.call1((ctx_r,)) { err.display(py); }` and
then returns `Ok`, and nothing calls `ctx.close()`. So an `assert` or an
un-caught `ValueError` inside the update function is swallowed, the app keeps
repainting, and under `smoke.py`'s 120s bound the test shows up as a TIMEOUT
rather than as a message naming the bad option. That is not a slow failure, it
is an unreadable one, and it is how two earlier snippets in this repo ended up
vacuous.

So the pattern throughout is:

1. `raise_around` calls the widget inside the update function and *catches*
   whatever comes out, appending it to a list. Nothing propagates, so the app
   never hangs and always closes normally.
2. `run_native` returns once the window closes.
3. Only then does the script assert on the collected outcomes and call
   `sys.exit(1)` if any expectation failed.

A widget that silently ignored `sufix` would append `"NOT RAISED"`, which
fails step 3 with a readable message. That is what makes these assertions
real rather than "it exited 0".

`tests/test_option_tracker.py` has the matching concern written down for
`frame`/`window`, which only get to use this file's lessons once their own task
lands.
"""

from bounded_app import run_snippet

# Every option the sliders accept, at values egui accepts. A run that completes
# proves each name was read and forwarded rather than rejected as unknown.
#
# Four of these are spelled differently in Python than a reader might guess,
# which is why each is pinned here rather than left to the docs:
#
# - `binary`/`octal`/`hexadecimal` take egui's own multi-argument shape:
#   `(min_width, twos_complement)`, plus `upper` for hexadecimal.
# - `handle_shape` accepts a word, or a `(word, aspect_ratio)` pair, because
#   egui's `HandleShape::Rect` carries a ratio.
# - `clamping` is an enum word, not a bool -- egui's setter takes a
#   `SliderClamping`.
# - `vertical` is a switch: egui's `vertical()` takes no argument.
ALL_SLIDER_OPTIONS = '''
import pyegui

f = pyegui.Float(5)
i = pyegui.Int(5)

def contents():
    pyegui.heading("slider options")
    pyegui.slider_float(f, 0, 50, "Gain",
                        suffix="dB", prefix="$", step_by=0.5,
                        logarithmic=False, clamping="edits", binary=(8, False))
    pyegui.slider_float(f, 0, 50, "Tail",
                        drag_value_speed=0.02,
                        vertical=True,
                        show_value=True,
                        trailing_fill=True,
                        text_color=(255, 0, 0),
                        fixed_decimals=2,
                        min_decimals=1,
                        max_decimals=3,
                        smallest_positive=1e-6,
                        largest_finite=1e6,
                        octal=(8, False),
                        hexadecimal=(16, False, True),
                        handle_shape="rect")
    pyegui.slider_float(f, 0, 50, "Narrow", handle_shape=("rect", 0.5))
    pyegui.slider_float(f, 0, 50, "Round", handle_shape="circle")
    pyegui.slider_float(f, 0, 50, "Flat", vertical=False, logarithmic=True)
    pyegui.slider_int(i, 0, 50, "Count", suffix=" pcs", step_by=5,
                      logarithmic=False, clamping="never",
                      fixed_decimals=0, show_value=True, octal=(4, False))

def update(ctx):
    pyegui.central_panel(ctx, contents)

pyegui.run_native("all options", update)
'''


def test_every_slider_option_is_accepted():
    ok, detail = run_snippet(ALL_SLIDER_OPTIONS)
    assert ok, (
        "a slider option the binding advertises was rejected or raised: "
        f"{detail}"
    )


# The positional signature must not shift. `slider_float(value, min, max, text)`
# is existing usage in examples/ and guides/; new parameters all go after
# `text`. pyo3 preserves positional order but nothing else guarantees it.
POSITION_UNCHANGED = '''
import pyegui

f = pyegui.Float(5)

def positional():
    pyegui.heading("positional")
    pyegui.slider_float(f, 0, 50, "Gain")

def keyword():
    pyegui.heading("keyword")
    pyegui.slider_float(f, min=0, max=50, text="Gain")

def update(ctx):
    pyegui.central_panel(ctx, positional)
    pyegui.central_panel(ctx, keyword)

pyegui.run_native("positional", update)
'''


def test_positional_signature_did_not_shift():
    ok, detail = run_snippet(POSITION_UNCHANGED)
    assert ok, f"slider_float's positional signature shifted: {detail}"


# A name cannot arrive twice. Python raises TypeError on the duplicate before
# pyo3 sees the call, which is what stops a named parameter from silently
# shadowing a **options key of the same name.
DUPLICATE_KEYWORD = '''
import sys
import pyegui

f = pyegui.Float(5)
raised = []

def contents():
    # `contents` runs once per frame, so the outcome is recorded once and the
    # rest of the frames are skipped. Without this the list grows to a hundred
    # entries and the diagnostic is unreadable.
    if raised:
        pyegui.label("draw something")
        return
    try:
        pyegui.slider_float(f, 0, 1, "x", suffix="ms", **{"suffix": "s"})
    except TypeError as exc:
        raised.append(type(exc).__name__)
    else:
        raised.append("NOT RAISED")
    pyegui.label("draw something")

def update(ctx):
    pyegui.central_panel(ctx, contents)

pyegui.run_native("duplicate", update)

if raised == ["TypeError"]:
    print("duplicate keyword raised TypeError as expected")
else:
    print("duplicate keyword outcome:", raised)
    sys.exit(1)
'''


def test_duplicate_keyword_raises():
    ok, detail = run_snippet(DUPLICATE_KEYWORD)
    assert ok, f"a duplicate keyword was silently accepted: {detail}"
    assert "NOT RAISED" in detail or "TypeError" in detail, (
        f"expected a TypeError from the duplicate keyword, got: {detail}"
    )


# The unknown-option check, and the enum-word errors, all caught in Python.
# These are the assertions Task 1 could not make: `reject_unknown_options` had
# no caller, so nothing could observe a false rejection.
UNKNOWN_OPTION = '''
import sys
import pyegui

f = pyegui.Float(5)
outcomes = []

def record(fn, note=""):
    # `contents` runs once per frame; record each case exactly once, then just
    # redraw. Appending every frame would bury the assertion in thousands of
    # duplicates and make the failure unreadable.
    if len(outcomes) >= 6:
        pyegui.label("done")
        return
    try:
        fn()
    except ValueError as exc:
        outcomes.append((note or "raised", str(exc)))
    except Exception as exc:
        outcomes.append((note or "other", type(exc).__name__ + ": " + str(exc)))
    else:
        outcomes.append((note or "NOT RAISED", ""))

def contents():
    pyegui.heading("unknown option")

    # A misspelling of `suffix`. Named parameters are separate from **options,
    # so `sufix` arrives in the dict and is not in `used`.
    record(lambda: pyegui.slider_float(f, 0, 1, "x", **{"sufix": "ms"}))

    # Genuine options must NOT be reported unknown. That is the property
    # `tests/test_option_tracker.py` guards for `frame`/`window`; this is the
    # same guard for the slider group, now that the check is live.
    record(lambda: pyegui.slider_float(f, 0, 1, "x", trailing_fill=True,
                                       fixed_decimals=3,
                                       handle_shape="circle"), "genuine")

    # The int slider names itself, not slider_float.
    record(lambda: pyegui.slider_int(pyegui.Int(1), 0, 1, "x",
                                     **{"sufix": "ms"}))

    # Two bad options at once: both are reported, sorted.
    record(lambda: pyegui.slider_float(f, 0, 1, "x",
                                       **{"alpha": 1, "beta": 2}))

    # Enum words egui does not have list the ones it does.
    record(lambda: pyegui.slider_float(f, 0, 1, "x", handle_shape="triangle"))
    record(lambda: pyegui.slider_float(f, 0, 1, "x", clamping="sometimes"))

def update(ctx):
    pyegui.central_panel(ctx, contents)

pyegui.run_native("unknown option", update)

problems = []

if outcomes[0][0] == "NOT RAISED":
    problems.append("a misspelled option was silently ignored")
elif "sufix" not in outcomes[0][1]:
    problems.append("the error does not name the option: " + outcomes[0][1])
elif "slider_float" not in outcomes[0][1]:
    problems.append("the error does not name the widget: " + outcomes[0][1])

if outcomes[1] != ("genuine", ""):
    problems.append("a genuine option was reported unknown: " + str(outcomes[1]))

if outcomes[2][0] == "NOT RAISED":
    problems.append("the int slider accepted a misspelled option")
elif "slider_int" not in outcomes[2][1]:
    problems.append("the int slider named the wrong widget: " + outcomes[2][1])

if outcomes[3][0] == "NOT RAISED":
    problems.append("two misspelled options were silently ignored")
elif "alpha" not in outcomes[3][1] or "beta" not in outcomes[3][1]:
    problems.append("not every unknown option was named: " + outcomes[3][1])

if outcomes[4][0] == "NOT RAISED":
    problems.append("a bad handle_shape word was accepted")
elif '"circle"' not in outcomes[4][1] or '"rect"' not in outcomes[4][1]:
    problems.append("the handle_shape error omits accepted values: " + outcomes[4][1])

if outcomes[5][0] == "NOT RAISED":
    problems.append("a bad clamping word was accepted")
elif '"never"' not in outcomes[5][1] or '"always"' not in outcomes[5][1]:
    problems.append("the clamping error omits accepted values: " + outcomes[5][1])

if problems:
    for p in problems:
        print("PROBLEM:", p)
    sys.exit(1)

print("unknown-option check behaved as expected for all 6 cases")
'''


def test_unknown_option_raises_and_names_the_widget():
    ok, detail = run_snippet(UNKNOWN_OPTION)
    assert ok, (
        "the unknown-option check did not behave as asserted:\n" + detail
    )


# TODO §6's list of Slider options is 22 names, not the 23 the task brief
# said: `update_while_editing` is DragValue's, and counting it against Slider
# is where the extra one came from.
#
# The inventory the implementation is held to. Kept as a set union rather than
# counted in the Rust source, so the expectation is stated once, readably, and
# a name added on one path but forgotten on another shows up here.
FLOAT_NAMED = {"suffix", "prefix", "step_by", "logarithmic", "clamping", "binary"}
# `binary` is deliberately absent from the int slider: egui's `binary()` is a
# display format, and `.integer()` already pins step 1 and 0 decimals. Accepting
# the name there would be a knob that does something unrelated to its name.
INT_NAMED = FLOAT_NAMED - {"binary"}
TAIL = {
    "drag_value_speed",
    "vertical",
    "show_value",
    "trailing_fill",
    "text_color",
    "fixed_decimals",
    "min_decimals",
    "max_decimals",
    "smallest_positive",
    "largest_finite",
    "octal",
    "hexadecimal",
    "handle_shape",
}
EXPECTED_OPTION_COUNT = 19

# TODO §6 lists 22 Slider options. Two are not implementable here and are
# recorded as such in TODO.md rather than stubbed: `custom_formatter` and
# `custom_parser` are Rust closures (`impl Fn(&str) -> Option<f64>`) with no
# Python equivalent in this binding. Accepting either name and ignoring it
# would be the exact silent-no-op the unknown-option check exists to prevent.
NOT_IMPLEMENTED = {"custom_formatter", "custom_parser"}

TODO_SECTION_6_LIST = {
    "logarithmic", "step_by", "binary", "hexadecimal", "octal",
    "prefix", "suffix", "custom_formatter", "custom_parser", "vertical",
    "clamping", "text_color", "handle_shape", "fixed_decimals",
    "min_decimals", "max_decimals", "show_value", "trailing_fill",
    "drag_value_speed", "update_while_editing", "largest_finite",
    "smallest_positive",
}


def test_inventory_matches_todo_section_6():
    """The 23 names TODO §6 lists, accounted for as implemented or not."""
    implemented = FLOAT_NAMED | INT_NAMED | TAIL

    # Not in TODO §6's list, and so not implemented on purpose.
    extra = implemented - TODO_SECTION_6_LIST
    assert not extra, f"options implemented that TODO §6 never listed: {extra}"

    missing = TODO_SECTION_6_LIST - implemented - NOT_IMPLEMENTED
    # `update_while_editing` is DragValue's option, not Slider's: it is absent
    # from egui 0.31.1's slider.rs entirely (verified in the pinned source).
    assert missing == {"update_while_editing"}, (
        "TODO §6 lists a Slider option that is neither implemented nor "
        f"recorded as absent from egui 0.31.1: {missing}"
    )

    assert len(TODO_SECTION_6_LIST) == 22, (
        "TODO §6's Slider list is no longer 22 names; re-derive this test: "
        f"{sorted(TODO_SECTION_6_LIST)}"
    )


def test_option_count():
    assert len(FLOAT_NAMED | INT_NAMED | TAIL) == EXPECTED_OPTION_COUNT, (
        f"expected {EXPECTED_OPTION_COUNT} option names, counted "
        f"{len(FLOAT_NAMED | INT_NAMED | TAIL)}"
    )
