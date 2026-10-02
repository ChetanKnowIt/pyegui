"""Run one of the repo's examples headlessly and prove it actually drew.

An example that imports cleanly and opens a blank window is broken, and the
export gate in `check.yml` cannot see that. This runs each app for a few
frames, screenshots the display, and fails if the result is a blank frame.

Usage:

    python tests/smoke.py --list          # print every app path, one per line
    python tests/smoke.py examples/pages.py

Why an app can be run at all: pyegui closes its window from Python, so an
example is a normal script that returns. `run_example` below patches
`run_native` so that after N frames it captures the display and requests a
close, which makes every example deterministic under Xvfb without editing a
single one of them.
"""

import os
import runpy
import shutil
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent

# Every app in the repo. Discovered rather than hardcoded where possible, so a
# newly added example is covered without touching this file.
SEARCH_DIRS = ["examples", "guides"]
EXTRA_FILES = ["debug.py"]

# How long to let each app draw before capturing. Long enough for the window to
# map and a few frames to composite, short enough to keep the job quick.
WARMUP_SECONDS = 2.5

# How many distinct colours a rendered window must have before it counts as
# having drawn something.
#
# This started out as a PNG byte-size threshold, which was wrong. Run
# 37035941793 failed six examples at 2983-3614 bytes, and every one of them was
# a real app rendering a single heading -- a sparse window legitimately
# compresses to about that, so the test rejected working code. File size says
# how much there is to compress, not whether anything was drawn.
#
# Counting distinct colours asks the question directly. A blank window is one
# flat colour; anything rendered adds more. ImageMagick is already installed
# in the job, so this costs nothing extra.
MIN_DISTINCT_COLOURS = 4


def find_apps():
    """Every runnable app in the repo, sorted for stable output."""
    found = []

    for directory in SEARCH_DIRS:
        base = REPO_ROOT / directory
        if not base.is_dir():
            continue
        for path in sorted(base.glob("*.py")):
            # gallery.py takes arguments and closes its own windows; the
            # screenshot job drives it, so it is not a standalone smoke app.
            if path.name == "gallery.py":
                continue
            if "run_native" not in path.read_text(encoding="utf-8"):
                continue
            found.append(path)

    for name in EXTRA_FILES:
        path = REPO_ROOT / name
        if path.is_file() and "run_native" in path.read_text(encoding="utf-8"):
            found.append(path)

    return found


def capture(path):
    """Screenshot the X root into `path`. True on success."""
    if shutil.which("import") is None:
        print("no 'import' binary; cannot capture", file=sys.stderr)
        return False

    display = os.environ.get("DISPLAY", ":0")
    result = subprocess.run(
        ["import", "-display", display, "-window", "root", "+repage", str(path)],
        capture_output=True,
    )
    if result.returncode != 0:
        print("capture failed:", result.stderr.decode(errors="replace").strip())
        return False
    return True


def count_colours(path):
    """Distinct colours in the PNG at `path`, or None if it cannot be read.

    egui's default dark theme paints a window in a handful of greys, so even a
    sparse example -- a single heading -- lands comfortably above four. A window
    that never drew anything is one flat colour regardless of how many bytes
    it compressed to, which is the thing worth testing.
    """
    identify = shutil.which("identify")
    if identify is None:
        return None

    # -format with %k reports the number of unique colours in the image.
    result = subprocess.run(
        [identify, "-format", "%k", str(path)],
        capture_output=True,
    )
    if result.returncode != 0:
        print("identify failed:", result.stderr.decode(errors="replace").strip())
        return None

    try:
        return int(result.stdout.decode().strip())
    except ValueError:
        return None


def run_example(app_path):
    """Run `app_path` as a script, capture it, and return (ok, reason).

    The example's own `run_native` is replaced so the app closes after a fixed
    number of frames. Everything else -- its imports, its state holders, its
    frame composition -- runs exactly as written.
    """
    import time

    import pyegui

    target = Path(app_path)
    if not target.is_absolute():
        target = REPO_ROOT / target

    frames = {"n": 0}
    captured = {"path": None, "ok": False}
    started = time.monotonic()
    original_run_native = pyegui.run_native

    def bounded_run_native(app_name, update_func, **kwargs):
        """Stand-in for run_native that captures and closes after N frames."""

        def update(ctx):
            update_func(ctx)
            frames["n"] += 1

            # Repaint explicitly: egui idles when it detects no change, and an
            # idle frame would never reach the capture.
            ctx.request_repaint()

            if time.monotonic() - started >= WARMUP_SECONDS:
                time.sleep(0.75)
                out = REPO_ROOT / "smoke-screenshot.png"
                captured["ok"] = capture(out)
                captured["path"] = out
                ctx.close()

        return original_run_native(app_name, update, **kwargs)

    pyegui.run_native = bounded_run_native

    try:
        # Mirror what `python examples/foo.py` does, including its
        # `__main__` guard, without the harness itself being importable.
        sys.argv = [str(target)]
        cwd = os.getcwd()
        os.chdir(REPO_ROOT)
        try:
            runpy.run_path(str(target), run_name="__main__")
        finally:
            os.chdir(cwd)
    except SystemExit as exc:
        # An example may call sys.exit; that is a clean end, not a failure.
        if exc.code not in (0, None):
            return False, f"exited with {exc.code}"
    finally:
        pyegui.run_native = original_run_native

    if not captured["ok"]:
        return False, "the app never drew a frame, so nothing was captured"

    shot = captured["path"]
    size = shot.stat().st_size
    colours = count_colours(shot)
    shot.unlink()

    if colours is None:
        return False, (
            f"could not read {shot.name}: ImageMagick's `identify` is not "
            "available, so the capture cannot be checked for content"
        )

    if colours < MIN_DISTINCT_COLOURS:
        return False, (
            f"captured {size} bytes but only {colours} distinct colour(s), so the "
            f"window is blank (needs at least {MIN_DISTINCT_COLOURS})"
        )

    return True, f"captured {size} bytes, {colours} distinct colours"


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        return 1

    target = sys.argv[1]
    if target == "--list":
        for path in find_apps():
            print(path.relative_to(REPO_ROOT))
        return 0

    # run_example replaces sys.argv so the example under test sees its own
    # name, exactly as `python examples/foo.py` would. Hold the path here
    # first -- reading sys.argv[1] after the run is an IndexError, which is
    # how every app "failed" in run 37034474791.
    ok, reason = run_example(target)
    if ok:
        print(f"ok {target}: {reason}")
        return 0

    print(f"FAIL {target}: {reason}", file=sys.stderr)
    return 1


if __name__ == "__main__":
    sys.exit(main())