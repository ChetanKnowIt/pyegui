"""Tests for the performance-page gate.

The gate warns and never fails, so every test here is asserting on the
`warnings` list rather than on an exception. The "does not raise" half matters
as much as the message: `tests/doc_claims.py` calls this from the `examples`
workflow, and an exception there would turn the whole job red over a
documentation drift nobody can act on.
"""

import json
import re
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
    """The gate checks one widget-count column; the page must quote that one.

    Asserting the constant against itself would pass forever and catch nothing.
    The gate compares only the WIDGET_COUNT column, so a page rewritten to
    quote a different column would carry numbers for a run the gate never
    checks -- and this gate is the only thing that would notice.

    Matched on "<count> widgets" rather than a bare number: the page quotes
    several figures (us-per-widget costs, a ratio, a run id), and only the
    frame's widget count is the column the gate reads.
    """
    page = (Path(__file__).resolve().parent.parent / "docs" / "performance.rst").read_text(
        encoding="utf-8"
    )
    assert re.search(rf"\b{WIDGET_COUNT} widgets?\b", page), (
        f"the page no longer states {WIDGET_COUNT} widgets, the only column "
        f"this gate checks (WIDGET_COUNT = {WIDGET_COUNT})"
    )
    # No other column may be advertised as the one being quoted. 50 and 500
    # are in the snapshot too, and the page mentioning them as prose would be
    # fine; quoting a *different* count as the run's subject is not.
    others = {c for c in ("50", "500", "2000") if c != WIDGET_COUNT}
    for count in others:
        assert not re.search(rf"\b{count} widgets\b", page), (
            f"the page also quotes {count} widgets, which is not the column "
            f"the gate checks"
        )


def _malformed_shapes():
    """Every shape a truncated or hand-edited snapshot can arrive in.

    Each one is a snapshot that parses as JSON but is not the shape the gate
    reads. A crash here would turn `examples` red for a documentation number
    nobody can act on, which is exactly what this tier exists to prevent.
    """
    return {
        "list": [],
        "string": "hello",
        "null": None,
        "number": 123,
        "by_scenario list": {"by_scenario": []},
        "by_scenario string": {"by_scenario": {"label": "x"}},
        "import is a number": {"import": 5},
        "import is a list": {"import": []},
        "scenario is a string": {"by_scenario": {"label": "x", "python_side": {}}},
        "widget entry is a string": {"by_scenario": {"python_side": {"2000": "x"}}},
        "comparison is a string": {
            "by_scenario": {"label": {"2000": {"comparison": "1.37x"}}}
        },
    }


def test_no_malformed_snapshot_shape_raises(tmp_path):
    page = tmp_path / "performance.rst"
    page.write_text(MATCHING_PAGE, encoding="utf-8")
    for label, snapshot in _malformed_shapes().items():
        sp = tmp_path / "combined.json"
        sp.write_text(json.dumps(snapshot), encoding="utf-8")
        warnings = []
        check_performance_page(sp, page, warnings)
        assert warnings, f"{label} produced neither a warning nor a raise"


def test_empty_by_scenario_warns(tmp_path):
    """No scenarios at all is a different defect from a malformed shape."""
    sp, pp = write(tmp_path, {"by_scenario": {}}, MATCHING_PAGE)
    warnings = []
    check_performance_page(sp, pp, warnings)
    assert warnings, warnings


def test_page_path_that_is_a_directory_warns_and_does_not_raise(tmp_path):
    page_dir = tmp_path / "performance.rst.d"
    page_dir.mkdir()
    sp, _ = write(tmp_path, SNAPSHOT, MATCHING_PAGE)
    warnings = []
    check_performance_page(sp, page_dir, warnings)
    assert any("not a file" in w for w in warnings), warnings


def test_missing_snapshot_file_warns_and_does_not_raise(tmp_path):
    page = tmp_path / "performance.rst"
    page.write_text(MATCHING_PAGE, encoding="utf-8")
    warnings = []
    check_performance_page(tmp_path / "absent.json", page, warnings)
    assert any("no benchmark snapshot" in w for w in warnings), warnings


# Bytes that are not valid UTF-8, built at runtime rather than written as
# escapes. A page or snapshot saved in the platform's default encoding, or one
# that is not text at all, arrives this way; catching only OSError would let
# the UnicodeDecodeError escape and turn `examples` red.
NOT_UTF8 = bytes([0xff, 0xfe])


def test_non_utf8_snapshot_warns_and_does_not_raise(tmp_path):
    sp = tmp_path / "combined.json"
    sp.write_bytes(b'{"by_scenario": {}}' + NOT_UTF8)
    page = tmp_path / "performance.rst"
    page.write_text(MATCHING_PAGE, encoding="utf-8")
    warnings = []
    check_performance_page(sp, page, warnings)
    assert any(
        "could not be read or decoded as UTF-8" in w for w in warnings
    ), warnings


def test_non_utf8_page_warns_and_does_not_raise(tmp_path):
    sp = tmp_path / "combined.json"
    sp.write_text(json.dumps(SNAPSHOT), encoding="utf-8")
    pp = tmp_path / "performance.rst"
    pp.write_bytes(MATCHING_PAGE.encode("utf-8") + NOT_UTF8)
    warnings = []
    check_performance_page(sp, pp, warnings)
    assert any(
        "could not be read or decoded as UTF-8" in w for w in warnings
    ), warnings


def test_both_non_utf8_warns_once_and_does_not_raise(tmp_path):
    """The page is read first, so it is the only one reported."""
    sp = tmp_path / "combined.json"
    sp.write_bytes(b'{"by_scenario": {}}' + NOT_UTF8)
    pp = tmp_path / "performance.rst"
    pp.write_bytes(MATCHING_PAGE.encode("utf-8") + NOT_UTF8)
    warnings = []
    check_performance_page(sp, pp, warnings)
    assert any("decoded as UTF-8" in w for w in warnings), warnings


def test_page_that_is_not_text_at_all_warns_and_does_not_raise(tmp_path):
    """Pure binary content is the same failure mode as a wrong encoding."""
    sp = tmp_path / "combined.json"
    sp.write_text(json.dumps(SNAPSHOT), encoding="utf-8")
    pp = tmp_path / "performance.rst"
    pp.write_bytes(bytes([0x00, 0x01, 0x02, 0xfe, 0xff]))
    warnings = []
    check_performance_page(sp, pp, warnings)
    assert any("decoded as UTF-8" in w for w in warnings), warnings


def test_invalid_json_warns_and_does_not_raise(tmp_path):
    sp = tmp_path / "combined.json"
    sp.write_text("{not json", encoding="utf-8")
    page = tmp_path / "performance.rst"
    page.write_text(MATCHING_PAGE, encoding="utf-8")
    warnings = []
    check_performance_page(sp, page, warnings)
    assert any("valid JSON" in w for w in warnings), warnings


def test_absent_scenario_warns_once_not_twice(tmp_path):
    """The absent-scenario warning must not be joined by a second one.

    Two warnings for one missing scenario reads as two defects and sends a
    reader looking for a problem that is not there.
    """
    snapshot = json.loads(json.dumps(SNAPSHOT))
    del snapshot["by_scenario"]["text_edit_hint"]
    sp, pp = write(tmp_path, snapshot, MATCHING_PAGE)
    warnings = []
    check_performance_page(sp, pp, warnings)
    mentioning = [w for w in warnings if "text_edit_hint" in w]
    assert len(mentioning) == 1, mentioning
