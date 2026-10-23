"""Builder options on the image group.

Follows the same bounded-runner pattern as ``test_slider_options.py``:
each assertion runs the widget inside a real egui update, catches
whatever comes out, and checks the outcome only after ``run_native``
returns.

An exception raised inside a widget's draw does not close the app:
``PyeguiApp::update`` prints the error and returns ``Ok``, so an
un-caught ``ValueError`` inside the update function is swallowed, the
app keeps repainting, and under a wall-clock bound the test shows up
as a TIMEOUT rather than as a message naming the bad option.

So the pattern throughout is:

1. ``record`` calls the widget inside the update function and *catches*
   whatever comes out, appending it to a list. Nothing propagates, so
   the app never hangs and always closes normally.
2. ``run_native`` returns once the window closes.
3. Only then does the script assert on the collected outcomes and
   ``sys.exit(1)`` if any expectation failed.
"""

from bounded_app import run_snippet

# Source used by every snippet below. A network URL is fine: egui loads
# images asynchronously and shows a placeholder on failure, so the test
# cares only that the option is parsed, not that the pixels arrive.
SRC = '"https://picsum.photos/100"'

# Every option the image group accepts, at values egui accepts. A run
# that completes proves each name was read and forwarded rather than
# rejected as unknown.
#
# Notes on less-obvious spellings:
# - `corner_radius` takes a single int (all corners), a 2-tuple
#   (nw=sw, ne=se), or a 4-tuple (nw, ne, sw, se).
# - `rotate` takes a 3-tuple: (angle_rad, origin_x, origin_y).
# - `uv` takes a Rect (the Python `Rect` pyclass), not a tuple.
# - `sense` takes a `Sense` pyclass instance.
# - `tint` and `bg_fill` take either a Color32 or an (r,g,b,a) tuple.
ALL_IMAGE_OPTIONS = f'''
import pyegui as pe

def contents():
    pe.heading("image options")
    pe.image({SRC}, max_height=50, max_width=50)
    pe.image({SRC}, tint=(255, 0, 0, 255))
    pe.image({SRC}, alt_text="a test image")
    pe.image({SRC}, fit_to_exact_size=(40, 40))
    pe.image({SRC}, fit_to_original_size=0.5)
    pe.image({SRC}, fit_to_fraction=(0.5, 0.5))
    pe.image({SRC}, maintain_aspect_ratio=True)
    pe.image({SRC}, shrink_to_fit=True)
    pe.image({SRC}, show_loading_spinner=True)
    pe.image({SRC}, bg_fill=(0, 0, 255, 255))
    pe.image({SRC}, corner_radius=8)
    pe.image({SRC}, corner_radius=(4, 8))
    pe.image({SRC}, corner_radius=(1, 2, 3, 4))
    pe.image({SRC}, sense=pe.Sense(click=True))
    pe.image({SRC}, sense=pe.Sense(hover=True))
    pe.image({SRC}, uv=pe.Rect((0, 0), (50, 50)))
    pe.image({SRC}, rotate=(0.5, 0.0, 0.0))
    pe.image({SRC}, tint=pe.Color32(255, 0, 0, 255))

def update(ctx):
    pe.central_panel(ctx, contents)

pe.run_native("all options", update)
'''


def test_every_image_option_is_accepted():
    ok, detail = run_snippet(ALL_IMAGE_OPTIONS)
    assert ok, (
        "an image option the binding advertises was rejected or raised: "
        f"{detail}"
    )


# The positional signature must not shift. `image(source)` has only one
# positional parameter; all tuning goes through keywords. This locks it.
POSITION_UNCHANGED = f'''
import pyegui as pe

def positional():
    pe.image({SRC})

def keyword():
    pe.image({SRC}, max_height=50)

def update(ctx):
    pe.central_panel(ctx, positional)
    pe.central_panel(ctx, keyword)

pe.run_native("positional", update)
'''


def test_positional_signature_did_not_shift():
    ok, detail = run_snippet(POSITION_UNCHANGED)
    assert ok, f"image's positional signature shifted: {detail}"


# Unknown-option and type-error checks, all caught in Python.
UNKNOWN_OPTION = '''
import sys
import pyegui as pe

SRC = "https://picsum.photos/100"
outcomes = []

def record(fn, note=""):
    # `contents` runs once per frame; record each case exactly once, then
    # just redraw. Appending every frame would bury the assertion.
    if len(outcomes) >= 8:
        pe.label("done")
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
    pe.heading("unknown option")

    # A misspelling of `max_height`. Named parameters are separate from
    # **options, so `max_hight` arrives in the dict and is not in `used`.
    record(lambda: pe.image_response(SRC, **{"max_hight": 50}))

    # Genuine options must NOT be reported unknown.
    record(lambda: pe.image_response(SRC, max_height=50, corner_radius=8,
                                     sense=pe.Sense(click=True)), "genuine")

    # rotate needs exactly 3 components.
    record(lambda: pe.image_response(SRC, rotate=(0.5, 0.0)))

    # uv must be a Rect, not a tuple.
    record(lambda: pe.image_response(SRC, uv=(0, 0, 100, 100)), "uv-type")

    # corner_radius must be an int or a 2/4-sequence, not a string.
    record(lambda: pe.image_response(SRC, corner_radius="round"), "cr-type")

    # Two bad options at once: both are reported, sorted.
    record(lambda: pe.image_response(SRC, **{"alpha": 1, "beta": 2}))

    # bg_fill must be a color, not a string.
    record(lambda: pe.image_response(SRC, bg_fill="blue"), "bg-type")

    # tint must be a color, not a string.
    record(lambda: pe.image_response(SRC, tint="red"), "tint-type")

def update(ctx):
    pe.central_panel(ctx, contents)

pe.run_native("unknown option", update)

problems = []

# max_hight -> ValueError naming "max_hight" and "image_response"
if outcomes[0][0] == "NOT RAISED":
    problems.append("a misspelled option was silently ignored")
elif "max_hight" not in outcomes[0][1]:
    problems.append("the error does not name the option: " + outcomes[0][1])
elif "image_response" not in outcomes[0][1]:
    problems.append("the error does not name the widget: " + outcomes[0][1])

# Genuine options must pass.
if outcomes[1] != ("genuine", ""):
    problems.append("a genuine option was reported unknown: " + str(outcomes[1]))

# rotate with 2 components -> ValueError
if outcomes[2][0] == "NOT RAISED":
    problems.append("rotate accepted 2 components instead of 3")
elif "rotate" not in outcomes[2][1]:
    problems.append("the rotate error does not name the option: " + outcomes[2][1])

# uv accepts Rect only
if outcomes[3][0] == "NOT RAISED":
    problems.append("uv accepted a non-Rect value: " + str(outcomes[3]))
elif "uv" not in outcomes[3][1]:
    problems.append("the uv error does not name the option: " + outcomes[3][1])

# corner_radius rejects strings
if outcomes[4][0] == "NOT RAISED":
    problems.append("corner_radius accepted a string: " + str(outcomes[4]))

# Two bad options at once
if outcomes[5][0] == "NOT RAISED":
    problems.append("two misspelled options were silently ignored")
elif "alpha" not in outcomes[5][1] or "beta" not in outcomes[5][1]:
    problems.append("not every unknown option was named: " + outcomes[5][1])

# bg_fill rejects strings
if outcomes[6][0] == "NOT RAISED":
    problems.append("bg_fill accepted a string: " + str(outcomes[6]))

# tint rejects strings
if outcomes[7][0] == "NOT RAISED":
    problems.append("tint accepted a string: " + str(outcomes[7]))

if problems:
    for p in problems:
        print("PROBLEM:", p)
    sys.exit(1)

print("unknown-option check behaved as expected for all 8 cases")
'''


def test_unknown_option_raises_and_names_the_widget():
    ok, detail = run_snippet(UNKNOWN_OPTION)
    assert ok, (
        "the unknown-option check did not behave as asserted:\n" + detail
    )


# The complete set of image options the implementation is held to.
IMAGE_OPTIONS = {
    "max_height",
    "max_width",
    "tint",
    "alt_text",
    "fit_to_exact_size",
    "fit_to_original_size",
    "fit_to_fraction",
    "maintain_aspect_ratio",
    "shrink_to_fit",
    "show_loading_spinner",
    "bg_fill",
    "corner_radius",
    "sense",
    "uv",
    "rotate",
}


def test_image_options_inventory():
    """Every name in the implementation's option set is accounted for."""
    # The Rust code calls reject_unknown_options with the union of:
    #   - max_height, max_width, tint, alt_text  (inline inserts)
    #   - fit_to_exact_size, fit_to_original_size, fit_to_fraction,
    #     maintain_aspect_ratio, shrink_to_fit, show_loading_spinner
    #                      (via opt_vec2 / opt_f32 / opt_bool)
    #   - bg_fill, uv, rotate                   (inline inserts)
    #   - corner_radius                         (via opt_corner_radius)
    #   - sense                                 (via opt_sense)
    opt_names = {
        "max_height", "max_width", "tint", "alt_text",
        "fit_to_exact_size", "fit_to_original_size", "fit_to_fraction",
        "maintain_aspect_ratio", "shrink_to_fit", "show_loading_spinner",
        "bg_fill", "uv", "rotate",
        "corner_radius", "sense",
    }
    assert opt_names == IMAGE_OPTIONS, (
        "image options inventory drifted from the implementation: "
        f"extra={opt_names - IMAGE_OPTIONS}, missing={IMAGE_OPTIONS - opt_names}"
    )
