"""Measure pyegui's own overhead, so it can be compared with plain egui.

There is a specific number worth knowing about a Python binding, and it is not
how fast the GUI is: the same egui code runs either way, so rendering speed is
egui's and says nothing about the binding. What the binding adds is Python-side
work on top of egui:

- how long `import pyegui` takes
- how long a widget call costs from Python, per call
- how long the first frame takes versus a steady-state frame

Each is reported separately because they answer different questions. The
per-call figure is the one that decides whether an app stays responsive with a
few thousand widgets; the first-frame figure is what a user perceives as
launch time.

Method notes, because these numbers are easy to get wrong:

- The widget loop runs inside a real frame. pyegui raises
  "UI functions should be called only within update_fun" otherwise, which is
  correct behaviour, so there is no way to measure dispatch from outside a
  frame -- and no reason to try, since a fake harness would not be measuring
  the path an app takes.
- The first frame is excluded from the steady-state loop: it includes shader
  compilation, texture upload and font rasterisation, none of which recur.
- `min` is reported as well as the mean. min is the cost of the call itself;
  the mean folds in whatever else the machine was doing.
- This measures pyegui only. It does not compare against a Rust egui app,
  because the egui side is identical in both and a number attributed to "egui"
  here would be measuring eframe's window creation, not a language.

Run under Xvfb, since eframe opens a real window:

    xvfb-run -a python3 bench/bench.py
"""

import json
import os
import statistics
import sys
import time

import pyegui
from pyegui import *  # noqa: F401,F403  -- names come from the star import

# Overhead scales with widget count, so a single figure at one count describes
# no other. `WIDGETS` lets the workflow re-run this at a second count without a
# code change; `bench/src/main.rs` reads the same variable, so both sides always
# measure the same thing.
WIDGETS_PER_FRAME = int(os.environ.get("WIDGETS", "500"))
STEADY_FRAMES = 60

# Fill a frame with work, and report how long the frame body took rather than
# the whole frame: the difference is eframe's compositing, which is egui's
# cost and identical in a Rust app.


def make_update(first_frame):
    def contents():
        # The widgets are the thing being measured. They need a Ui, and this
        # function composes its own frame, so the labels go inside a panel
        # rather than at the top level.
        t0 = time.perf_counter()

        for i in range(WIDGETS_PER_FRAME):
            pyegui.label(f"row {i}")

        elapsed = (time.perf_counter() - t0) * 1000

        # Record the first frame by its index in `steady`, not by a separate
        # counter: the frame number is advanced in update(), which runs before
        # the panel, so testing `n == 0` inside contents never matched.
        if not steady:
            first_frame["ms"] = elapsed
        steady.append(elapsed)

    def update(ctx):
        # Drawn first, so the closing frame is measured too: closing before the
        # panel would drop the last frame from `steady` and leave it one short
        # of STEADY_FRAMES. The Rust baseline measures inside its closure and
        # closes afterwards, and now both record the same number of frames.
        pyegui.central_panel(ctx, contents)

        # Count first, then decide -- so exactly STEADY_FRAMES frames are drawn
        # and measured. The Rust baseline increments before its close check,
        # which is why these have to match.
        first_frame["n"] += 1
        if first_frame["n"] >= STEADY_FRAMES:
            ctx.close()
        else:
            # egui idles when it sees no change, and an idle frame never
            # advances the loop -- so 500 identical labels is not a visible
            # change after the first frame. Without this the benchmark hangs
            # after one frame. The Rust baseline does the same for the same
            # reason.
            ctx.request_repaint()

    return update


steady = []
first_frame = {"n": 0, "ms": None}


def main():
    t0 = time.perf_counter()
    pyegui.run_native("bench", make_update(first_frame))
    total = (time.perf_counter() - t0) * 1000

    # Per-widget cost: the frame body is WIDGETS_PER_FRAME label calls, so
    # divide out the loop and the timing calls themselves.
    per_widget_min = min(steady) / WIDGETS_PER_FRAME * 1000  # microseconds
    per_widget_mean = statistics.mean(steady) / WIDGETS_PER_FRAME * 1000

    results = {
        "widgets_per_frame": WIDGETS_PER_FRAME,
        "frames_measured": len(steady),
        "first_frame_ms": round(first_frame["ms"], 4),
        "steady_frame_ms": {
            "min": round(min(steady), 4),
            "mean": round(statistics.mean(steady), 4),
            "stdev": round(statistics.stdev(steady), 4) if len(steady) > 1 else 0.0,
        },
        "per_widget_us": {
            "min": round(per_widget_min, 3),
            "mean": round(per_widget_mean, 3),
        },
        "run_native_total_ms": round(total, 2),
        "python": sys.version.split()[0],
    }

    print(json.dumps(results, indent=2))


if __name__ == "__main__":
    # import cost measured in a fresh interpreter, since by the time this runs
    # the module is long since imported.
    if "--import-cost" in sys.argv:
        t = time.perf_counter()
        import pyegui as _  # noqa: F401
        print(json.dumps({"import_pyegui_ms": round((time.perf_counter() - t) * 1000, 3)}))
    else:
        main()