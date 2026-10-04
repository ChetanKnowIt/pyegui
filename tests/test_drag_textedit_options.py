"""Builder options on the DragValue and TextEdit groups.

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

So the pattern throughout is the one `test_slider_options.py` established:

1. `record` calls the widget inside the update function and *catches* whatever
   comes out, appending it to a list. Nothing propagates, so the app never hangs
   and always closes normally.
2. `run_native` returns once the window closes.
3. Only then does the script assert on the collected outcomes and call
   `sys.exit(1)` if any expectation failed.

A widget that silently ignored `desired_widht` would append `"NOT RAISED"`,
which fails step 3 with a readable message.

## The pinned source, not the plan, is the inventory

Every option below was checked against egui 0.31.1's own source, extracted from
`egui-0.31.1.crate` at the pinned version:

- `src/widgets/drag_value.rs` -- `grep -n 'pub fn '` is the complete setter list
- `src/widgets/text_edit/builder.rs` -- same

The plan's prose list in TODO §6 disagrees with that source in several places.
`test_inventory_matches_todo_section_6` below records every disagreement, so a
future edit to either list fails loudly rather than drifting silently.
"""

from bounded_app import run_snippet


# --------------------------------------------------------------------------
# Step 1: DragValue
# --------------------------------------------------------------------------

# Every option DragValue accepts, at values egui accepts. A run that completes
# proves each name was read and forwarded rather than rejected as unknown.
#
# Two of these are spelled differently in Python than a reader might guess:
#
# - `binary`/`octal`/`hexadecimal` take egui's own multi-argument shape:
#   `(min_width, twos_complement)`, plus `upper` for hexadecimal. They are NOT
#   bools, despite looking like switches.
# - `suffix`/`prefix` are keyword parameters; everything else is in **options.
ALL_DRAG_OPTIONS = '''
import pyegui

f = pyegui.Float(5)
i = pyegui.Int(5)

def contents():
    pyegui.heading("drag options")
    pyegui.drag_float(f, 0, 50, 1.5, suffix="dB", prefix="$")
    pyegui.drag_float(f, 0, 50, 1.5,
                      update_while_editing=True,
                      clamp_existing_to_range=False,
                      fixed_decimals=2,
                      min_decimals=1,
                      max_decimals=3,
                      binary=(8, False),
                      octal=(8, False),
                      hexadecimal=(16, False, True))
    pyegui.drag_int(i, 0, 50, 1, suffix=" pcs", prefix="#")
    pyegui.drag_int(i, 0, 50, 1,
                    update_while_editing=False,
                    clamp_existing_to_range=True,
                    fixed_decimals=0,
                    min_decimals=0,
                    max_decimals=0,
                    binary=(8, True))
    # The no-tail path must still work with keyword parameters present.
    pyegui.drag_int_response(i, 0, 50, 1)
    pyegui.drag_float_response(f, 0, 50, 1.5)

def update(ctx):
    pyegui.central_panel(ctx, contents)

pyegui.run_native("all drag options", update)
'''


def test_every_drag_option_is_accepted():
    ok, detail = run_snippet(ALL_DRAG_OPTIONS)
    assert ok, (
        "a DragValue option the binding advertises was rejected or raised: "
        f"{detail}"
    )


# --------------------------------------------------------------------------
# Step 2: TextEdit
# --------------------------------------------------------------------------

# `hint_text` first and separately: it is the one option this API shipped with
# (it landed with the tracker in Task 1), and this is the preservation test.
# `desired_rows` on a singleline field is egui's own behaviour -- the setter
# exists on the shared builder -- so it is exercised on both.
ALL_TEXT_EDIT_OPTIONS = '''
import pyegui

single = pyegui.Str("")
multi = pyegui.Str("")

def contents():
    pyegui.heading("text edit options")
    pyegui.text_edit_singleline(single, hint_text="name")
    pyegui.text_edit_singleline(single,
                                 password=True,
                                 desired_width=200.0,
                                 desired_rows=1,
                                 char_limit=32,
                                 interactive=True,
                                 clip_text=False,
                                 lock_focus=False,
                                 frame=False,
                                 cursor_at_end=True,
                                 background_color=(30, 30, 30),
                                 margin=4,
                                 horizontal_align="center",
                                 vertical_align="max")
    pyegui.text_edit_multiline(multi, hint_text="body")
    pyegui.text_edit_multiline(multi,
                               password=False,
                               desired_width=300.0,
                               desired_rows=4,
                               char_limit=0,
                               interactive=False,
                               clip_text=True,
                               lock_focus=True,
                               frame=True,
                               cursor_at_end=False,
                               background_color=(10, 10, 10, 255),
                               margin=(8, 4),
                               horizontal_align="min",
                               vertical_align="min")
    # Every accepted Align word, on both widgets.
    for word in ("min", "center", "max"):
        pyegui.text_edit_singleline(single, horizontal_align=word)
        pyegui.text_edit_singleline(single, vertical_align=word)
        pyegui.text_edit_multiline(multi, horizontal_align=word)
        pyegui.text_edit_multiline(multi, vertical_align=word)
    # The response variants take the same options.
    pyegui.text_edit_singleline_response(single, hint_text="r")
    pyegui.text_edit_multiline_response(multi, desired_rows=3)

def update(ctx):
    pyegui.central_panel(ctx, contents)

pyegui.run_native("all text edit options", update)
'''


def test_every_text_edit_option_is_accepted():
    ok, detail = run_snippet(ALL_TEXT_EDIT_OPTIONS)
    assert ok, (
        "a TextEdit option the binding advertises was rejected or raised: "
        f"{detail}"
    )


# Step 6 of the brief, as a behavioural test rather than a bare call: an empty
# text field with a hint draws the hint, so the field must still render and the
# app must still draw a frame. A field that silently lost its `hint_text` would
# still draw, so this alone cannot prove it -- which is why the stronger
# preservation evidence is that `hint_text` is in the recorded option set below
# and therefore cannot be reported unknown by the check.
HINT_TEXT_STILL_WORKS = '''
import pyegui

single = pyegui.Str("")
multi = pyegui.Str("")

def contents():
    pyegui.heading("hints")
    pyegui.text_edit_singleline(single, hint_text="name")
    pyegui.text_edit_multiline(multi, hint_text="body")

def update(ctx):
    pyegui.central_panel(ctx, contents)

pyegui.run_native("hints", update)
'''


def test_hint_text_still_works_on_both():
    ok, detail = run_snippet(HINT_TEXT_STILL_WORKS)
    assert ok, (
        "hint_text stopped working on text_edit_singleline/text_edit_multiline: "
        f"{detail}"
    )


# --------------------------------------------------------------------------
# Step 5: the enum-string rule, tested both ways
# --------------------------------------------------------------------------

# Step 4 of the brief asked for a macro; the repo already had `enum_word` from
# Task 2 (built for `Slider`'s `handle_shape`) and the instructions say to reuse
# it rather than write a second mechanism. `opt_align` is the dict-reading
# wrapper it lacked. These assertions are what the reuse has to satisfy.
ALIGN_WORDS_BOTH_WAYS = '''
import sys
import pyegui

single = pyegui.Str("")

# Every accepted word, on both widgets, must succeed.
ACCEPTED = ("min", "center", "max")

# These must all be rejected. Chosen because each is a plausible mistake rather
# than a random string:
#   "left"       -- an egui const (Align::LEFT) that aliases Min, but not a
#                   variant name, so it is not in the accepted list
#   "Min"        -- the variant name in Rust's casing
#   "min "       -- a trailing space
#   ""           -- empty
REJECTED = ("left", "Min", "min ", "", "middle")

accepted_outcomes = []
rejected = []
recorded = []

def record(fn, note):
    # `contents` runs once per frame; record each case exactly once, then just
    # redraw, or the diagnostic fills with thousands of duplicates.
    if recorded.count(note):
        pyegui.label("done")
        return
    recorded.append(note)
    try:
        fn()
    except ValueError as exc:
        if note.startswith("reject"):
            rejected.append((note, str(exc)))
        else:
            accepted_outcomes.append((note, "REJECTED: " + str(exc)))
    else:
        if note.startswith("reject"):
            accepted_outcomes.append((note, "NOT REJECTED"))
        else:
            accepted_outcomes.append((note, ""))

def contents():
    pyegui.heading("align words")
    for word in ACCEPTED:
        record(lambda w=word: pyegui.text_edit_singleline(single, horizontal_align=w),
               "accept-h-" + word)
        record(lambda w=word: pyegui.text_edit_multiline(single, vertical_align=w),
               "accept-v-" + word)
    for word in REJECTED:
        record(lambda w=word: pyegui.text_edit_singleline(single, horizontal_align=w),
               "reject-h-" + repr(w))
        record(lambda w=word: pyegui.text_edit_multiline(single, vertical_align=w),
               "reject-v-" + repr(w))

def update(ctx):
    pyegui.central_panel(ctx, contents)

pyegui.run_native("align words", update)

problems = []

for note, detail in accepted_outcomes:
    if detail:
        problems.append(f"{note}: {detail}")

# Every rejection must name the option AND list the accepted words, so a caller
# can self-correct without reading the docs.
for note, detail in rejected:
    if "horizontal_align" not in detail and "vertical_align" not in detail:
        problems.append(f"{note}: the error does not name the option: {detail}")
    elif '"min"' not in detail or '"max"' not in detail or '"center"' not in detail:
        problems.append(f"{note}: the error omits accepted values: {detail}")

if problems:
    for p in problems:
        print("PROBLEM:", p)
    sys.exit(1)

print(f"all {len(accepted_outcomes)} align cases behaved as expected")
'''


def test_align_words_accepted_and_rejected():
    ok, detail = run_snippet(ALIGN_WORDS_BOTH_WAYS)
    assert ok, "the align word rule did not behave as asserted:\\n" + detail


# --------------------------------------------------------------------------
# The unknown-option check on both new groups
# --------------------------------------------------------------------------

UNKNOWN_OPTION = '''
import sys
import pyegui

f = pyegui.Float(5)
i = pyegui.Int(5)
single = pyegui.Str("")
multi = pyegui.Str("")
outcomes = []

def record(fn, note=""):
    # Same once-only guard as above: `contents` runs every frame.
    if len(outcomes) >= 8:
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

    # A misspelling of a *tail* option on the float drag value.
    record(lambda: pyegui.drag_float(f, 0, 1, 1.0, **{"clamp_existing_to_raneg": True}))

    # Genuine options must NOT be reported unknown. This is the false-rejection
    # guard, and it is the one that would catch a tracker regression.
    record(lambda: pyegui.drag_float(f, 0, 1, 1.0,
                                     update_while_editing=True,
                                     fixed_decimals=2,
                                     hexadecimal=(16, False, True),
                                     suffix="m",
                                     prefix="$"), "genuine-drag")

    # The int drag names itself, not drag_float.
    record(lambda: pyegui.drag_int(i, 0, 1, 1, **{"sufix": "ms"}))

    # A misspelling on text_edit_singleline.
    record(lambda: pyegui.text_edit_singleline(single, **{"hint_tex": "name"}))

    # `hint_text` -- the option that shipped before the check was live -- must
    # not be reported unknown now that the check is.
    record(lambda: pyegui.text_edit_multiline(multi, hint_text="body",
                                              desired_rows=3,
                                              horizontal_align="max"), "genuine-text")

    # Two bad options at once: both are reported.
    record(lambda: pyegui.text_edit_singleline(single, **{"alpha": 1, "beta": 2}))

    # A bad enum word on the multiline side names vertical_align.
    record(lambda: pyegui.text_edit_multiline(multi, vertical_align="middle"))

def update(ctx):
    pyegui.central_panel(ctx, contents)

pyegui.run_native("unknown option", update)

problems = []

if outcomes[0][0] == "NOT RAISED":
    problems.append("a misspelled DragValue option was silently ignored")
elif "clamp_existing_to_raneg" not in outcomes[0][1]:
    problems.append("the error does not name the option: " + outcomes[0][1])
elif "drag_float" not in outcomes[0][1]:
    problems.append("the error does not name the widget: " + outcomes[0][1])

if outcomes[1] != ("genuine-drag", ""):
    problems.append("a genuine DragValue option was reported unknown: " + str(outcomes[1]))

if outcomes[2][0] == "NOT RAISED":
    problems.append("the int drag value accepted a misspelled option")
elif "drag_int" not in outcomes[2][1]:
    problems.append("the int drag value named the wrong widget: " + outcomes[2][1])

if outcomes[3][0] == "NOT RAISED":
    problems.append("text_edit_singleline accepted a misspelled option")
elif "text_edit_singleline" not in outcomes[3][1]:
    problems.append("the error named the wrong widget: " + outcomes[3][1])

if outcomes[4] != ("genuine-text", ""):
    problems.append("hint_text or another genuine TextEdit option was "
                    "reported unknown: " + str(outcomes[4]))

if outcomes[5][0] == "NOT RAISED":
    problems.append("two misspelled TextEdit options were silently ignored")
elif "alpha" not in outcomes[5][1] or "beta" not in outcomes[5][1]:
    problems.append("not every unknown option was named: " + outcomes[5][1])

if outcomes[6][0] == "NOT RAISED":
    problems.append("a bad vertical_align word was accepted")
elif '"min"' not in outcomes[6][1] or '"max"' not in outcomes[6][1]:
    problems.append("the vertical_align error omits accepted values: " + outcomes[6][1])

if problems:
    for p in problems:
        print("PROBLEM:", p)
    sys.exit(1)

print("unknown-option check behaved as expected for all 7 cases")
'''


def test_unknown_option_raises_and_names_the_widget():
    ok, detail = run_snippet(UNKNOWN_OPTION)
    assert ok, (
        "the unknown-option check did not behave as asserted on the drag or "
        "text-edit groups:\n" + detail
    )


# A duplicate keyword is CPython's TypeError, before pyo3 is entered. It is the
# guarantee that lets the keyword parameters skip the `used` record, so it is
# worth pinning for both groups.
DUPLICATE_KEYWORD = '''
import sys
import pyegui

f = pyegui.Float(5)
single = pyegui.Str("")
outcomes = []
recorded = []

def record(fn, note):
    # `contents` runs once per frame; record each case exactly once, then just
    # redraw, or the list grows with the frame count.
    if recorded.count(note):
        pyegui.label("draw something")
        return
    recorded.append(note)
    try:
        fn()
    except TypeError:
        outcomes.append((note, "TypeError"))
    else:
        outcomes.append((note, "NOT RAISED"))

def contents():
    pyegui.heading("duplicate keyword")
    record(lambda: pyegui.drag_float(f, 0, 1, 1.0, suffix="ms", **{"suffix": "s"}),
           "drag_float")
    record(lambda: pyegui.drag_int(pyegui.Int(1), 0, 1, 1,
                                   prefix="$", **{"prefix": "£"}),
           "drag_int")

def update(ctx):
    pyegui.central_panel(ctx, contents)

pyegui.run_native("duplicate", update)

problems = [f"{note}: a duplicate keyword was silently accepted"
            for note, outcome in outcomes if outcome != "TypeError"]

if problems:
    for p in problems:
        print("PROBLEM:", p)
    sys.exit(1)

print("duplicate keywords raised TypeError as expected")
'''


def test_duplicate_keyword_raises_on_both_groups():
    ok, detail = run_snippet(DUPLICATE_KEYWORD)
    assert ok, f"a duplicate keyword was silently accepted: {detail}"


# --------------------------------------------------------------------------
# The inventory, held against the pinned source
# --------------------------------------------------------------------------

# TODO §6's DragValue line, verbatim.
TODO_SECTION_6_DRAG = {
    "prefix", "suffix", "custom_formatter", "custom_parser",
    "binary", "hexadecimal", "octal", "fixed_decimals", "min_decimals",
    "max_decimals", "update_while_editing", "clamp_existing_to_range",
}

# TODO §6's TextEdit line, verbatim.
TODO_SECTION_6_TEXT_EDIT = {
    "password", "desired_width", "desired_rows", "char_limit",
    "lock_focus", "font", "interactive", "cursor_at_end",
    "background_color", "margin", "horizontal_align", "vertical_align",
    "clip_text", "frame", "return_key", "load_state", "store_state",
}

DRAG_NAMED = {"suffix", "prefix"}
DRAG_TAIL = {
    "binary",
    "octal",
    "hexadecimal",
    "fixed_decimals",
    "min_decimals",
    "max_decimals",
    "update_while_editing",
    "clamp_existing_to_range",
}

# `hint_text` is not in TODO §6's list because it already worked. It is in
# `used`, so the unknown-option check must accept it, so the implementation is
# held to it here.
TEXT_EDIT_OPTIONS = {
    "hint_text",
    "password",
    "desired_width",
    "desired_rows",
    "char_limit",
    "interactive",
    "clip_text",
    "lock_focus",
    "frame",
    "cursor_at_end",
    "background_color",
    "margin",
    "horizontal_align",
    "vertical_align",
}

# TODO §6 names these; egui 0.31.1 does not make them implementable here.
#
# - `custom_formatter` / `custom_parser` are Rust closures
#   (`impl 'a + Fn(f64, RangeInclusive<usize>) -> String` and
#   `impl 'a + Fn(&str) -> Option<f64>`). Handing a Python callable to a Rust
#   closure needs a trampoline plus a lifetime strategy for the Py reference;
#   the `Slider` and `DragValue` builders are built and consumed within one
#   call, so the closure cannot outlive a borrow of the Python object safely.
#   Not stubbed, and deliberately not accepted as an option name, so a caller
#   who tries gets the unknown-option ValueError rather than a silent no-op.
# - `font` takes a `FontSelection`, which needs a `TextStyle` class that does not
#   exist (TODO §4). Deferred for the same reason rather than half-done.
# - `return_key` takes `impl Into<Option<KeyboardShortcut>>`; a `KeyboardShortcut`
#   is `{ modifiers, logical_key }` with a `Key` enum, and neither has a Python
#   type. `load_state` / `store_state` are static methods on `TextEdit` taking a
#   `Context` and an `Id`, reaching outside this binding's widget call shape.
DRAG_NOT_IMPLEMENTED = {"custom_formatter", "custom_parser"}
TEXT_EDIT_NOT_IMPLEMENTED = {
    "font", "return_key", "load_state", "store_state",
}

# Names TODO §6 lists that do not exist as DragValue setters in egui 0.31.1, or
# that exist under a different name. Checked against `drag_value.rs`:
#
# - `clamping` is NOT a DragValue option. It is `Slider`'s, and takes a
#   `SliderClamping`. This is the third Slider/DragValue mix-up in TODO §6 after
#   `update_while_editing` and `binary`.
DRAG_NAME_MISTAKES = set()


def test_inventory_matches_todo_section_6():
    """Every name in TODO §6's two lines is implemented, or recorded as not."""
    drag_implemented = DRAG_NAMED | DRAG_TAIL
    drag_unaccounted = (
        TODO_SECTION_6_DRAG - drag_implemented - DRAG_NOT_IMPLEMENTED - DRAG_NAME_MISTAKES
    )
    assert not drag_unaccounted, (
        "TODO §6 lists a DragValue option that is neither implemented nor "
        f"recorded as not implementable: {sorted(drag_unaccounted)}"
    )
    drag_extra = drag_implemented - TODO_SECTION_6_DRAG
    assert not drag_extra, (
        f"DragValue options implemented that TODO §6 never listed: {sorted(drag_extra)}"
    )

    text_unaccounted = (
        TODO_SECTION_6_TEXT_EDIT - TEXT_EDIT_OPTIONS - TEXT_EDIT_NOT_IMPLEMENTED
    )
    assert not text_unaccounted, (
        "TODO §6 lists a TextEdit option that is neither implemented nor "
        f"recorded as not implementable: {sorted(text_unaccounted)}"
    )
    text_extra = TEXT_EDIT_OPTIONS - TODO_SECTION_6_TEXT_EDIT
    # `hint_text` predates this work; it is expected here and only here.
    assert text_extra == {"hint_text"}, (
        "TextEdit options implemented that TODO §6 never listed and that are "
        f"not the pre-existing hint_text: {sorted(text_extra)}"
    )


def test_inventory_counts():
    assert len(DRAG_NAMED | DRAG_TAIL) == 10, (
        f"expected 10 DragValue option names, counted {len(DRAG_NAMED | DRAG_TAIL)}"
    )
    assert len(TEXT_EDIT_OPTIONS) == 14, (
        f"expected 14 TextEdit option names, counted {len(TEXT_EDIT_OPTIONS)}"
    )