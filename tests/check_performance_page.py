"""Check docs/performance.rst against the committed benchmark snapshot.

Warn, never fail. Between-run variation on hosted runners is 1.30x-1.55x for
identical code, so a page number that differs from the last snapshot by a
rounding step is expected rather than defective. A hard gate here would go red
on ordinary benchmark runs, and a build that goes red for a reason nobody can
act on is a build people learn to ignore -- which would cost more than the
gate protects.

The README, CHANGELOG and TODO checks in `doc_claims.py` still fail the build.
This is a second, softer tier underneath them.

Two things this deliberately does not do:

* It does not touch the README's benchmark table. The README quotes 1.32-1.33x
  from a specific run; this page quotes the current snapshot. Both are honest,
  they are different runs on different machines, and `docs/performance.rst`
  says which run it is from so a reader can see that rather than guess.
* It does not invent a ratio for `python_side`. That scenario has no Rust
  twin, so there is no `comparison` block and there is nothing to compare
  against; the page states the frame time instead and this gate checks that.
"""

import json
import re
from pathlib import Path

# The widget count the page quotes. The snapshot holds 50, 500 and 2000, and a
# page that picked a different one per sentence would be unreadable; 2000 is
# the steady-state case and is where the per-widget numbers are tightest.
WIDGET_COUNT = "2000"

# scenario -> page substitution names, and the snapshot path within the
# scenario that carries the number. Ordered so the rendered page reads in the
# same order as the scenario table.
SUBSTITUTIONS = {
    "label": (
        "ratio_label",
        "ratio_label_range",
        "extra_us_label",
    ),
    "text_edit_plain": (
        "ratio_text_edit_plain",
        "ratio_text_edit_plain_range",
        "extra_us_text_edit_plain",
    ),
    "text_edit_hint": (
        "ratio_text_edit_hint",
        "ratio_text_edit_hint_range",
        "extra_us_text_edit_hint",
    ),
}

# Substitutions the page must define for the gate to have anything to check.
# `last_verified_run` and `last_verified_commit` carry no machine-checkable
# value -- the snapshot commit fires no workflow -- so they are checked for
# presence only, which is why this is a set of names rather than a mapping.
REQUIRED = {
    "import_ms",
    "frame_ms_python_side",
    "last_verified_run",
    "last_verified_commit",
}

def _substitutions(page_text):
    return dict(
        re.findall(r"^\.\. \|(\w+)\| replace:: (.+?)\s*$", page_text, re.M)
    )


def _fmt_ratio(value):
    return f"{value}x"


def _fmt_range(low, high):
    return f"{low}x-{high}x"


def _ms(value):
    return f"{value}ms"


def _fmt_us(value):
    return f"{value}us"


def _warn(warnings, message):
    """Every exit from here goes through this, so the contract stays obvious."""
    warnings.append(message)


def _as_dict(value):
    """Coerce a snapshot node to a dict, treating anything else as absent.

    The snapshot is written by the benchmark run, so its shape is a fact we
    read rather than one we can assume: a truncated or hand-edited file can put
    a list where a mapping belongs. This gate warns and never fails, so a
    surprising shape has to become a warning and never an AttributeError.
    """
    return value if isinstance(value, dict) else {}


def check_performance_page(snapshot_path, page_path, warnings):
    """Append warnings about page/snapshot disagreement. Never raises."""
    snapshot_path = Path(snapshot_path)
    page_path = Path(page_path)

    if not page_path.exists():
        # No page, no claims. Warning here would be noise on every branch that
        # predates the page.
        return

    if not page_path.is_file():
        # Exists but is not a file -- a directory, most likely. read_text on
        # one raises IsADirectoryError, which would turn a documentation-path
        # mistake into a red `examples` job.
        _warn(
            warnings,
            f"docs/performance.rst: {page_path} is not a file, so its numbers "
            "cannot be checked.",
        )
        return

    try:
        page_text = page_path.read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError) as exc:
        # UnicodeDecodeError is a ValueError, not an OSError, so catching only
        # OSError would let a page saved in the platform's default encoding --
        # or a file that is not text at all -- escape and turn `examples` red.
        _warn(
            warnings,
            f"docs/performance.rst: {page_path} could not be read or decoded "
            f"as UTF-8 ({exc}), so its numbers cannot be checked.",
        )
        return
    declared = _substitutions(page_text)

    if not snapshot_path.exists():
        _warn(
            warnings,
            f"docs/performance.rst: no benchmark snapshot at {snapshot_path}, "
            "so its numbers cannot be checked. Dispatch the `benchmark` "
            "workflow to produce one.",
        )
        return

    try:
        snapshot = json.loads(snapshot_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        _warn(warnings, f"docs/performance.rst: snapshot is not valid JSON ({exc})")
        return
    except (OSError, UnicodeDecodeError) as exc:
        # Same hole as the page read: a snapshot written in another encoding
        # decodes to a UnicodeDecodeError (a ValueError), which OSError alone
        # would not catch.
        _warn(
            warnings,
            f"docs/performance.rst: the snapshot at {snapshot_path} could not "
            f"be read or decoded as UTF-8 ({exc}), so the page cannot be "
            "checked against it.",
        )
        return

    if not isinstance(snapshot, dict):
        _warn(
            warnings,
            "docs/performance.rst: the snapshot is a "
            f"{type(snapshot).__name__}, not a mapping, so its shape is "
            "malformed and the page cannot be checked against it. Re-run "
            "`benchmark`.",
        )
        return

    by_scenario = snapshot.get("by_scenario")
    if not isinstance(by_scenario, dict):
        _warn(
            warnings,
            "docs/performance.rst: the snapshot predates by_scenario, so the "
            "page cannot be checked against it. Re-run `benchmark`."
            + ("" if by_scenario is None
               else " by_scenario is a "
                    f"{type(by_scenario).__name__}, not a mapping."),
        )
        return

    import_ms = _as_dict(snapshot.get("import")).get("import_pyegui_ms")
    if import_ms is not None:
        _compare(warnings, declared, "import_ms", _ms(import_ms), "import time")
    else:
        _warn(
            warnings,
            "docs/performance.rst: the snapshot has no import.import_pyegui_ms, "
            "so |import_ms| cannot be checked. Re-run `benchmark`.",
        )

    for scenario, names in SUBSTITUTIONS.items():
        if scenario not in by_scenario:
            _warn(
                warnings,
                f"docs/performance.rst: the snapshot has no '{scenario}' "
                "scenario. Re-run `benchmark`.",
            )
            continue
        for name in names:
            if name not in declared:
                _warn(
                    warnings,
                    f"docs/performance.rst: |{name}| is never defined, so the "
                    f"page has no number for '{scenario}'.",
                )

    # python_side. No Rust twin, so no ratio -- only the frame time.
    python_side = by_scenario.get("python_side")
    if not isinstance(python_side, dict):
        _warn(
            warnings,
            "docs/performance.rst: the snapshot has no 'python_side' scenario. "
            "Re-run `benchmark`.",
        )
    else:
        entry = _as_dict(python_side.get(WIDGET_COUNT))
        python_only = _as_dict(entry.get("python_only"))
        where = f"python_side at {WIDGET_COUNT} widgets/frame"
        expected = {}
        for key, name, fmt in (
            ("pyegui_frame_ms_median", "frame_ms_python_side", str),
            ("pyegui_per_widget_us_median", "per_widget_us_python_side", _fmt_us),
        ):
            # Presence is checked as well as value. `_compare` deliberately
            # stays silent when a name is undeclared (the "never defined"
            # warning is the caller's job), so without this an undeclared
            # python_side figure would produce no warning at all -- the page
            # could drop the number and nothing would notice.
            if name not in declared:
                _warn(
                    warnings,
                    f"docs/performance.rst: |{name}| is never defined, so the "
                    f"page has no number for python_side.",
                )
            value = python_only.get(key)
            if value is None:
                _warn(
                    warnings,
                    f"docs/performance.rst: python_side at {WIDGET_COUNT} "
                    f"widgets has no python_only.{key}, so |{name}| cannot be "
                    "checked. Re-run `benchmark`.",
                )
                continue
            expected[name] = fmt(value)
        for name, value in expected.items():
            _compare(warnings, declared, name, value, where)

    for name in sorted(REQUIRED):
        if name not in declared:
            _warn(
                warnings,
                f"docs/performance.rst: |{name}| is never defined, so the page "
                "has no number for it.",
            )

    # The ratios, at the widget count the page quotes.
    for scenario, names in SUBSTITUTIONS.items():
        if scenario not in by_scenario:
            # Already reported as an absent scenario by the first loop; a
            # second warning saying the same file has "no comparison block"
            # would read as a distinct defect and is not one.
            continue
        entry = _as_dict(by_scenario.get(scenario)).get(WIDGET_COUNT)
        comparison = _as_dict(_as_dict(entry).get("comparison"))
        if not comparison:
            _warn(
                warnings,
                f"docs/performance.rst: scenario '{scenario}' at {WIDGET_COUNT} "
                "widgets has no comparison block, so its ratio cannot be "
                "checked. Re-run `benchmark`.",
            )
            continue
        # The per-widget overhead the page quotes is the third substitution
        # for each scenario. It lives in the same `comparison` block, and it is
        # the number that says what the binding charges rather than what the
        # ratio came out at -- so it is checked like the other two.
        for missing in ("ratio_median", "ratio_min", "ratio_max", "extra_us_median"):
            if missing not in comparison:
                _warn(
                    warnings,
                    f"docs/performance.rst: scenario '{scenario}' comparison at "
                    f"{WIDGET_COUNT} widgets has no {missing}.",
                )
        if not all(
            k in comparison
            for k in ("ratio_median", "ratio_min", "ratio_max", "extra_us_median")
        ):
            continue
        expected = {
            names[0]: _fmt_ratio(comparison["ratio_median"]),
            names[1]: _fmt_range(comparison["ratio_min"], comparison["ratio_max"]),
            names[2]: _fmt_us(comparison["extra_us_median"]),
        }
        for name, value in expected.items():
            _compare(
                warnings, declared, name, value,
                f"scenario '{scenario}', {WIDGET_COUNT} widgets/frame",
            )


def _compare(warnings, declared, name, expected, where):
    actual = declared.get(name)
    if actual is None:
        # Already reported as "never defined" by the caller; do not double up.
        return
    if actual != expected:
        _warn(
            warnings,
            f"docs/performance.rst: |{name}| says {actual}, the snapshot says "
            f"{expected} ({where}).",
        )
