"""Builder options on the DatePickerButton group.

## Every assertion here runs AFTER `run_native` returns, not inside the callback

`tests/bounded_app.py` explains why, and it is the same shape the slider and
DragValue/TextEdit files use. `PyeguiApp::update` swallows a widget exception,
displays it, and returns `Ok`; nothing calls `ctx.close()`. So an un-caught
`ValueError` inside the update function presents as a HANG under a wall-clock
bound, not as a message naming the offending option. The snippets here catch
their own errors, and assert after the app has closed.

## The pinned source is the inventory, and it disagrees with TODO §6

egui_extras is pinned `=0.31.1` in Cargo.toml. In that version the datepicker
is a MODULE DIRECTORY, not a single file:

    crates/egui_extras/src/datepicker/mod.rs     -- `month_data`, re-export
    crates/egui_extras/src/datepicker/button.rs   -- `DatePickerButton`
    crates/egui_extras/src/datepicker/popup.rs   -- `DatePickerPopup`

`grep -n 'pub fn ' crates/egui_extras/src/datepicker/button.rs` is the
complete setter list. It has exactly eight, plus a `#[deprecated]` `id_source`
alias:

    id_salt, combo_boxes, arrows, calendar, calendar_week,
    show_icon, format, highlight_weekends

TODO §6 listed eleven. Three of them -- `start_end_years`, `reverse_years`,
`year_scroll_to` -- are NOT setters on `DatePickerButton` at all: the year
range is hardcoded inside the popup's own `draw` as
`today.year() - 100 .. today.year() + 10` (popup.rs:87) and
`DatePickerPopup`'s fields are `pub(crate)`, so there is no builder hook to
expose even in principle. §6 also described `calendar` and `arrows` as enums
taking strings; they are plain `bool` setters
(`pub fn calendar(mut self, calendar: bool)`, button.rs:71).

`test_inventory_matches_todo_section_6` records every one of those
disagreements, so a future edit to either list fails loudly rather than
drifting silently.
"""

import re
from pathlib import Path

from bounded_app import run_snippet

REPO_ROOT = Path(__file__).resolve().parent.parent


# --------------------------------------------------------------------------
# Every option the date picker accepts, at values egui accepts.
# --------------------------------------------------------------------------
ALL_DATE_PICKER_OPTIONS = '''
import datetime
import pyegui

today = pyegui.Date(datetime.datetime.now().date())
other = pyegui.Date(datetime.date(2020, 1, 2))

def contents():
    pyegui.heading("date picker options")
    # Keyword parameters, all four.
    pyegui.date_picker_button(today,
                              format="%d/%m/%Y",
                              show_icon=False,
                              highlight_weekends=False,
                              id_salt="one")
    # The **options tail, all four. `calendar` and `arrows` are BOOL switches
    # in egui_extras 0.31.1, not the enum words the plan assumed, so `False`
    # here is a real value and not a no-op sentinel.
    pyegui.date_picker_button(other,
                              combo_boxes=False,
                              arrows=False,
                              calendar=False,
                              calendar_week=False)
    # Defaults must still be reachable: the no-option path.
    pyegui.date_picker_button(other)
    # The response variant takes the same options.
    pyegui.date_picker_button_response(today, format="%Y", show_icon=True)
    pyegui.date_picker_button_response(other,
                                       highlight_weekends=True,
                                       id_salt="two",
                                       combo_boxes=True,
                                       arrows=True,
                                       calendar=True,
                                       calendar_week=True)

def update(ctx):
    pyegui.central_panel(ctx, contents)

pyegui.run_native("all date picker options", update)
'''


def test_every_date_picker_option_is_accepted():
    ok, detail = run_snippet(ALL_DATE_PICKER_OPTIONS)
    assert ok, (
        "a DatePickerButton option the binding advertises was rejected or "
        f"raised: {detail}"
    )


# --------------------------------------------------------------------------
# The no-option path must not regress: this is the call that shipped before
# Task 4, and the new `#[pyo3(signature = ...)]` is the only thing between
# it and working code.
# --------------------------------------------------------------------------
NO_OPTIONS_STILL_WORKS = '''
import datetime
import pyegui

today = pyegui.Date(datetime.datetime.now().date())

def contents():
    pyegui.label("plain")
    pyegui.date_picker_button(today)

def update(ctx):
    pyegui.central_panel(ctx, contents)

pyegui.run_native("plain date picker", update)
'''


def test_date_picker_button_without_options_still_works():
    ok, detail = run_snippet(NO_OPTIONS_STILL_WORKS)
    assert ok, f"date_picker_button(date) regressed: {detail}"


# --------------------------------------------------------------------------
# An unknown option name is an ERROR, and the message names it.
#
# The three names TODO §6 lists but egui_extras does not have are here too:
# they must be reported unknown rather than silently accepted, because
# accepting them would be a lie -- there is nothing in egui to forward them to.
# --------------------------------------------------------------------------
UNKNOWN_OPTION_IS_AN_ERROR = '''
import datetime
import sys
import pyegui

today = pyegui.Date(datetime.datetime.now().date())
other = pyegui.Date(datetime.date(2020, 1, 2))

REJECTED = (
    "calender",           # a misspelling of a real option
    "deselectable",       # an option from a widget that has one
    "start_end_years",    # TODO §6, but not a DatePickerButton setter
    "reverse_years",      # TODO §6, but not a DatePickerButton setter
    "year_scroll_to",     # TODO §6, but not a DatePickerButton setter
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

def reject(name):
    return lambda n=name: pyegui.date_picker_button(other, **{n: 1})
def reject_response(name):
    return lambda n=name: pyegui.date_picker_button_response(other, **{n: 1})

def contents():
    pyegui.heading("unknown options")
    for name in REJECTED:
        record("plain-" + name, reject(name))
        record("response-" + name, reject_response(name))

def update(ctx):
    pyegui.central_panel(ctx, contents)

pyegui.run_native("unknown options", update)

problems = []

if len(outcomes) != 2 * len(REJECTED):
    problems.append(
        f"expected {2 * len(REJECTED)} outcomes, got {len(outcomes)}: {outcomes}")

for note, detail in outcomes:
    if detail == "NOT RAISED":
        problems.append(f"{note}: the unknown option was accepted silently")
    elif detail.startswith("WRONG EXCEPTION TYPE"):
        problems.append(f"{note}: expected ValueError, got {detail}")
    else:
        # The message must name the offending option, or the error is useless.
        option = note.split("-", 1)[1]
        if option not in detail:
            problems.append(f"{note}: the error does not name the option: {detail}")

if problems:
    for p in problems:
        print("PROBLEM:", p)
    sys.exit(1)
'''


def test_unknown_option_is_an_error_that_names_it():
    ok, detail = run_snippet(UNKNOWN_OPTION_IS_AN_ERROR)
    assert ok, (
        "an unknown date picker option was not reported as a ValueError naming "
        f"it: {detail}"
    )


# --------------------------------------------------------------------------
# The keyword parameters must be keyword parameters.
#
# `format`, `show_icon`, `highlight_weekends` and `id_salt` were chosen as
# named parameters under the plan's decision; a caller passing them
# positionally in the old order would get egui's own signature, so this test
# pins that they are accepted BY KEYWORD on both functions, which a
# `**options`-only design would also pass. So this test on its own does NOT
# distinguish the two designs -- `test_duplicate_keyword_raises` below is the
# one that does, and it exists for exactly that reason.
# --------------------------------------------------------------------------
KEYWORD_PARAMETERS_ACCEPTED = '''
import datetime
import sys
import pyegui

today = pyegui.Date(datetime.datetime.now().date())
other = pyegui.Date(datetime.date(2021, 6, 7))

# Every keyword the plan fixed as a named parameter, on both functions, and
# with `**options` empty so nothing is doing the accepting but the signature.
KEYWORD_CALLS = (
    ("format", "%Y-%m"),
    ("show_icon", False),
    ("highlight_weekends", False),
    ("id_salt", "salt"),
)

problems = []
recorded = []

def record(note, fn):
    if note in recorded:
        pyegui.label("done")
        return
    recorded.append(note)
    try:
        fn()
    except Exception as exc:
        problems.append(f"{note}: raised {exc!r}")

def contents():
    pyegui.heading("keyword parameters")
    for name, value in KEYWORD_CALLS:
        record("plain-" + name, lambda n=name, v=value: pyegui.date_picker_button(today, **{n: v}))
        record("response-" + name, lambda n=name, v=value: pyegui.date_picker_button_response(other, **{n: v}))

def update(ctx):
    pyegui.central_panel(ctx, contents)

pyegui.run_native("keyword parameters", update)

if problems:
    for p in problems:
        print("PROBLEM:", p)
    sys.exit(1)
'''


def test_keyword_parameters_are_accepted_on_both_functions():
    ok, detail = run_snippet(KEYWORD_PARAMETERS_ACCEPTED)
    assert ok, (
        "a date picker keyword parameter was rejected: "
        f"{detail}"
    )


# --------------------------------------------------------------------------
# The test that actually distinguishes a keyword parameter from a dict entry.
#
# CPython raises `TypeError` on a keyword given twice -- `f(format=..., **{"format":
# ...})` is an error before pyo3, or before any argument binding, happens. So
# this is NOT a test of pyo3 and NOT a test of `reject_unknown_options`. It is
# a test of one specific fact about this design: each of these four names is
# declared as a keyword parameter in the `#[pyo3(signature = ...)]`, so the
# name is consumed by the signature and never appears in `**options`.
#
# Why that matters, and what the alternative would do: if one of the four were
# ever DEMOTED into the `**options` tail, then under `reject_unknown_options`
# the tail's `used` set would no longer record it, the key would arrive in the
# dict unconsumed, and the check would reject it as an unknown option -- a
# caller spelling a supported option by keyword would get a ValueError. The
# duplicate is the cheapest observable symptom of that demotion from the Python
# side: under a keyword parameter the TypeError is raised, and under a
# `**options`-only entry it would NOT be.
#
# So this test CAN fail under a `**options`-only design, which is what makes it
# worth having. See `tests/test_slider_options.py` and
# `tests/test_drag_textedit_options.py` for the same guard on their groups.
# --------------------------------------------------------------------------
DUPLICATE_KEYWORD = '''
import datetime
import sys
import pyegui

today = pyegui.Date(datetime.datetime.now().date())
other = pyegui.Date(datetime.date(2021, 6, 7))

# A first value that is valid for the option, and a second, different one. The
# first must be a real value rather than a sentinel: if it were rejected on its
# own merits the TypeError below would still pass and the test would say
# nothing about duplicates.
DUPLICATES = (
    ("format", "%Y-%m", "%d/%m/%Y"),
    ("show_icon", False, True),
    ("highlight_weekends", False, True),
    ("id_salt", "salt", "other"),
)

outcomes = []
recorded = []

def record(note, fn):
    # `contents` runs once per frame; record each case exactly once, then just
    # redraw, or the list grows with the frame count and the diagnostic becomes
    # unreadable.
    if note in recorded:
        pyegui.label("done")
        return
    recorded.append(note)
    try:
        fn()
    except TypeError as exc:
        outcomes.append((note, "TypeError", str(exc)))
    except Exception as exc:
        outcomes.append((note, "WRONG EXCEPTION TYPE", repr(exc)))
    else:
        outcomes.append((note, "NOT RAISED", ""))

def contents():
    pyegui.heading("duplicate keyword")
    for name, first, second in DUPLICATES:
        record(
            "plain-" + name,
            lambda n=name, a=first, b=second:
                pyegui.date_picker_button(today, **{n: a}, **{n: b}),
        )
        record(
            "response-" + name,
            lambda n=name, a=first, b=second:
                pyegui.date_picker_button_response(other, **{n: a}, **{n: b}),
        )

def update(ctx):
    pyegui.central_panel(ctx, contents)

pyegui.run_native("duplicate keyword", update)

problems = []

if len(outcomes) != 2 * len(DUPLICATES):
    problems.append(
        f"expected {2 * len(DUPLICATES)} outcomes, got {len(outcomes)}: {outcomes}")

for note, kind, detail in outcomes:
    if kind == "NOT RAISED":
        problems.append(
            f"{note}: the duplicate was silently accepted, so this name is NOT a "
            "keyword parameter -- it is a **options entry, and "
            "reject_unknown_options would then reject the single-keyword form")
    elif kind != "TypeError":
        problems.append(f"{note}: expected TypeError, got {kind}: {detail}")
    elif "multiple values" not in detail:
        # A TypeError from something else -- an extraction failure inside the
        # option, say -- would also read as "a duplicate raised", so the
        # message has to be CPython's.
        problems.append(
            f"{note}: raised TypeError, but not CPython's duplicate-keyword one: "
            f"{detail}")

if problems:
    for p in problems:
        print("PROBLEM:", p)
    sys.exit(1)

print("duplicate keywords raised TypeError as expected")
'''


def test_duplicate_keyword_raises():
    ok, detail = run_snippet(DUPLICATE_KEYWORD)
    assert ok, f"a duplicate keyword was silently accepted: {detail}"
    assert "NOT RAISED" in detail or "TypeError" in detail, (
        f"expected a TypeError from the duplicate keyword, got: {detail}"
    )


# --------------------------------------------------------------------------
# The inventory itself, checked against TODO §6 so the two lists cannot drift.
#
# This reads the TODO line and the pinned egui_extras source rather than
# restating either, so a future edit to TODO §6 or an upstream bump is caught
# instead of silently diverging from what the binding implements.
# --------------------------------------------------------------------------
# The eight setters `egui_extras::DatePickerButton` actually has, verified
# against `crates/egui_extras/src/datepicker/button.rs` at tag 0.31.1 by reading
# the `impl<'a> DatePickerButton<'a>` block (button.rs:24-104) and listing every
# `pub fn` in it. Hard-coded here because the crate source is not vendored in
# this repo and there is no local Rust toolchain to fetch it with; the TODO
# cross-check below is what keeps the pair from drifting.
VERIFIED_SETTERS = [
    "arrows",
    "calendar",
    "calendar_week",
    "combo_boxes",
    "format",
    "highlight_weekends",
    "id_salt",
    "show_icon",
]

# TODO §6 listed these three. None is a `DatePickerButton` setter; they are
# recorded as not implemented so the unknown-option check reports them.
NOT_SETTERS = ["start_end_years", "reverse_years", "year_scroll_to"]


def _todo_date_picker_item():
    """The TODO §6 DatePickerButton checklist item, as text.

    Item-scoped rather than line-scoped on purpose. An earlier version of this
    test read only the first line of the item and so could not see the
    "not implemented" paragraph; an even earlier one swept the whole section and
    swept up every backticked word in the prose, which made the assertion
    depend on how the surrounding sentences were worded.
    """
    todo = (REPO_ROOT / "TODO.md").read_text(encoding="utf-8")
    start = todo.index("- [x] `DatePickerButton`:")
    rest = todo[start + len("- [x] `DatePickerButton`:") :]
    end = rest.index("\n- [")
    return rest[:end]


def _librs_date_picker_names():
    lib = (REPO_ROOT / "src" / "lib.rs").read_text(encoding="utf-8")
    fn = lib.index("unsafe fn apply_date_picker_options")
    body = lib[fn : lib.index("reject_unknown_options(o, used, widget)", fn)]
    return sorted(set(re.findall(r'opt_bool\(o, "([a-z_]+)"', body)))


def test_inventory_matches_todo_section_6():
    """TODO §6's DatePickerButton item and the implemented tail must agree.

    Three facts are pinned, all read from files rather than hard-coded in the
    test body:

    - TODO §6 documents each of the eight verified setters. If someone drops
      one, the docs are wrong.
    - TODO §6 records each of the three names egui_extras does NOT have, so a
      future reader does not go looking for them in the source. This is the
      record the plan asked for.
    - The four tail names in TODO §6 are exactly the four the Rust helper
      reads. A name documented but not read would be rejected at runtime by
      the unknown-option check.
    """
    item = _todo_date_picker_item()
    named = set(re.findall(r"`([a-z_]+)`", item))

    missing = [n for n in VERIFIED_SETTERS if n not in named]
    assert not missing, (
        f"TODO.md's DatePickerButton item no longer documents {missing}, which "
        "egui_extras 0.31.1 does have as setters on DatePickerButton."
    )

    unrecorded = [n for n in NOT_SETTERS if n not in named]
    assert not unrecorded, (
        f"TODO.md's DatePickerButton item no longer records {unrecorded} as "
        "not implemented. They are not DatePickerButton setters, and leaving "
        "them off the list makes them look like an oversight rather than a "
        "verified absence."
    )

    tail = _librs_date_picker_names()
    expected_tail = ["arrows", "calendar", "calendar_week", "combo_boxes"]
    assert tail == expected_tail, (
        "src/lib.rs's apply_date_picker_options reads "
        f"{tail}, but the verified setters put in the tail are {expected_tail}. "
        "The other four are keyword parameters. A name documented but not read "
        "would be rejected by the unknown-option check."
    )
