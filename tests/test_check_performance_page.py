"""Tests for the performance-page gate.

The gate warns and never fails, so every test here is asserting on the
`warnings` list rather than on an exception. The "does not raise" half matters
as much as the message: `tests/doc_claims.py` calls this from the `examples`
workflow, and an exception there would turn the whole job red over a
documentation drift nobody can act on.
"""

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from check_performance_page import WIDGET_COUNT, check_performance_page  # noqa: E402


def _comparison(ratio_median, ratio_min, ratio_max):
    return {
        "comparison": {
            "ratio_median": ratio_median,
            "ratio_min": ratio_min,
            "ratio_max": ratio_max,
        }
    }


# Values taken from the committed snapshot at 2000 widgets/frame, so the
# fixture is a shape the real file actually has rather than a convenient one.
SNAPSHOT = {
    "import": {"import_pyegui_ms": 7.277},
    "by_scenario": {
        "label": {
            "50": _comparison(1.46, 1.43, 1.51),
            "2000": _comparison(1.37, 1.36, 1.37),
        },
        "text_edit_plain": {
            "2000": _comparison(1.31, 1.28, 1.34),
        },
        "text_edit_hint": {
            "2000": _comparison(1.5, 1.45, 1.51),
        },
        "python_side": {
            "2000": {"python_only": {"pyegui_frame_ms_median": 0.636}},
        },
    },
}

MATCHING_PAGE = (
    ".. |import_ms| replace:: 7.277ms\n"
    ".. |ratio_label| replace:: 1.37x\n"
    ".. |ratio_label_range| replace:: 1.36x-1.37x\n"
    ".. |ratio_text_edit_plain| replace:: 1.31x\n"
    ".. |ratio_text_edit_plain_range| replace:: 1.28x-1.34x\n"
    ".. |ratio_text_edit_hint| replace:: 1.5x\n"
    ".. |ratio_text_edit_hint_range| replace:: 1.45x-1.51x\n"
    ".. |frame_ms_python_side| replace:: 0.636\n"
    # Presence-checked only: no machine-checkable value exists for these.
    ".. |last_verified_run| replace:: 37196277892\n"
    ".. |last_verified_commit| replace:: e766e6b\n"
)


def write(tmp_path, snapshot, page):
    sp = tmp_path / "combined.json"
    sp.write_text(json.dumps(snapshot), encoding="utf-8")
    pp = tmp_path / "performance.rst"
    pp.write_text(page, encoding="utf-8")
    return sp, pp


def test_missing_snapshot_warns_and_does_not_raise(tmp_path):
    page = tmp_path / "performance.rst"
    page.write_text(".. |ratio_label| replace:: 1.33x\n", encoding="utf-8")
    warnings = []
    check_performance_page(tmp_path / "nope.json", page, warnings)
    assert warnings, "a missing snapshot must warn, not pass silently"
    assert any("no benchmark snapshot" in w for w in warnings), warnings


def test_missing_page_is_silent(tmp_path):
    """No page means no claims to check; warning about it would be noise."""
    sp, _ = write(tmp_path, SNAPSHOT, "")
    warnings = []
    check_performance_page(sp, tmp_path / "absent.rst", warnings)
    assert not warnings, warnings


def test_matching_page_warns_about_nothing(tmp_path):
    sp, pp = write(tmp_path, SNAPSHOT, MATCHING_PAGE)
    warnings = []
    check_performance_page(sp, pp, warnings)
    assert not warnings, warnings


def test_drifted_page_warns_with_both_values(tmp_path):
    page = MATCHING_PAGE.replace("|ratio_label| replace:: 1.37x",
                                  "|ratio_label| replace:: 9.99x")
    sp, pp = write(tmp_path, SNAPSHOT, page)
    warnings = []
    check_performance_page(sp, pp, warnings)
    assert any("9.99x" in w and "1.37" in w for w in warnings), warnings


def test_snapshot_without_by_scenario_warns_and_does_not_raise(tmp_path):
    sp, pp = write(tmp_path, {"import": {}}, "nothing here\n")
    warnings = []
    check_performance_page(sp, pp, warnings)
    assert any("by_scenario" in w for w in warnings), warnings


def test_unparseable_snapshot_warns_and_does_not_raise(tmp_path):
    sp = tmp_path / "combined.json"
    sp.write_text("{not json", encoding="utf-8")
    pp = tmp_path / "performance.rst"
    pp.write_text(MATCHING_PAGE, encoding="utf-8")
    warnings = []
    check_performance_page(sp, pp, warnings)
    assert any("valid JSON" in w for w in warnings), warnings


def test_missing_substitution_warns(tmp_path):
    """A page that drops a token has no number where one belongs."""
    page = "\n".join(
        line for line in MATCHING_PAGE.splitlines()
        if "ratio_text_edit_hint|" not in line
    ) + "\n"
    sp, pp = write(tmp_path, SNAPSHOT, page)
    warnings = []
    check_performance_page(sp, pp, warnings)
    assert any("ratio_text_edit_hint" in w and "never defined" in w
               for w in warnings), warnings


def test_missing_scenario_warns(tmp_path):
    snapshot = json.loads(json.dumps(SNAPSHOT))
    del snapshot["by_scenario"]["text_edit_hint"]
    sp, pp = write(tmp_path, snapshot, MATCHING_PAGE)
    warnings = []
    check_performance_page(sp, pp, warnings)
    assert any("text_edit_hint" in w and "no 'text_edit_hint'" in w
               for w in warnings), warnings


def test_import_and_python_side_numbers_are_checked(tmp_path):
    page = MATCHING_PAGE.replace("|import_ms| replace:: 7.277ms",
                                 "|import_ms| replace:: 99.0ms")
    page = page.replace("|frame_ms_python_side| replace:: 0.636",
                        "|frame_ms_python_side| replace:: 0.500")
    sp, pp = write(tmp_path, SNAPSHOT, page)
    warnings = []
    check_performance_page(sp, pp, warnings)
    assert any("99.0ms" in w and "7.277" in w for w in warnings), warnings
    assert any("0.500" in w and "0.636" in w for w in warnings), warnings


def test_no_comparison_key_is_not_invented_into_a_ratio(tmp_path):
    """python_side has no Rust twin, so there is no ratio to check.

    Asserted because the tempting mistake is to derive one from the frame
    time and print a number no measurement supports.
    """
    sp, pp = write(tmp_path, SNAPSHOT, MATCHING_PAGE)
    warnings = []
    check_performance_page(sp, pp, warnings)
    assert not any("python_side" in w and "ratio" in w for w in warnings), warnings


def test_the_real_page_and_the_real_snapshot_agree():
    """The committed pair, checked in CI as well as here."""
    root = Path(__file__).resolve().parent.parent
    warnings = []
    check_performance_page(
        root / "bench" / "results" / "combined.json",
        root / "docs" / "performance.rst",
        warnings,
    )
    assert not warnings, warnings


def test_widget_count_is_the_one_the_page_states():
    assert WIDGET_COUNT == "2000"