"""Tests for the performance-page gate.

The gate warns and never fails, so every test here is asserting on the
`warnings` list rather than on an exception. The "does not raise" half matters
as much as the message: `tests/doc_claims.py` calls this from the `examples`
workflow, and an exception there would turn the whole job red over a
documentation drift nobody can act on.
"""

import ast
import json
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from check_performance_page import WIDGET_COUNT, check_performance_page  # noqa: E402


def _comparison(ratio_median, ratio_min, ratio_max, extra_us_median):
    return {
        "comparison": {
            "ratio_median": ratio_median,
            "ratio_min": ratio_min,
            "ratio_max": ratio_max,
            "extra_us_median": extra_us_median,
        }
    }


# Values taken from the committed snapshot at 2000 widgets/frame, so the
# fixture is a shape the real file actually has rather than a convenient one.
SNAPSHOT = {
    "import": {"import_pyegui_ms": 9.652},
    "by_scenario": {
        "label": {
            "50": _comparison(1.44, 1.39, 1.47, 0.23),
            "2000": _comparison(1.33, 1.14, 1.36, 0.131),
        },
        "text_edit_plain": {
            "2000": _comparison(1.26, 1.19, 1.27, 0.182),
        },
        "text_edit_hint": {
            "2000": _comparison(3.26, 3.11, 3.32, 1.65),
        },
        "slider_many_options": {
            "2000": _comparison(2.66, 2.61, 2.96, 4.181),
        },
        "python_side": {
            "2000": {
                "python_only": {
                    "pyegui_frame_ms_median": 0.5263,
                    "pyegui_per_widget_us_median": 0.263,
                }
            },
        },
    },
}

MATCHING_PAGE = (
    ".. |import_ms| replace:: 9.652ms\n"
    ".. |ratio_label| replace:: 1.33x\n"
    ".. |ratio_label_range| replace:: 1.14x-1.36x\n"
    ".. |extra_us_label| replace:: 0.131us\n"
    ".. |ratio_text_edit_plain| replace:: 1.26x\n"
    ".. |ratio_text_edit_plain_range| replace:: 1.19x-1.27x\n"
    ".. |extra_us_text_edit_plain| replace:: 0.182us\n"
    ".. |ratio_text_edit_hint| replace:: 3.26x\n"
    ".. |ratio_text_edit_hint_range| replace:: 3.11x-3.32x\n"
    ".. |extra_us_text_edit_hint| replace:: 1.65us\n"
    ".. |ratio_slider_many_options| replace:: 2.66x\n"
    ".. |ratio_slider_many_options_range| replace:: 2.61x-2.96x\n"
    ".. |extra_us_slider_many_options| replace:: 4.181us\n"
    ".. |frame_ms_python_side| replace:: 0.5263\n"
    ".. |per_widget_us_python_side| replace:: 0.263us\n"
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
    page = MATCHING_PAGE.replace("|ratio_label| replace:: 1.33x",
                                  "|ratio_label| replace:: 9.99x")
    sp, pp = write(tmp_path, SNAPSHOT, page)
    warnings = []
    check_performance_page(sp, pp, warnings)
    assert any("9.99x" in w and "1.33" in w for w in warnings), warnings


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
    page = MATCHING_PAGE.replace("|import_ms| replace:: 9.652ms",
                                 "|import_ms| replace:: 99.0ms")
    page = page.replace("|frame_ms_python_side| replace:: 0.5263",
                        "|frame_ms_python_side| replace:: 0.500")
    sp, pp = write(tmp_path, SNAPSHOT, page)
    warnings = []
    check_performance_page(sp, pp, warnings)
    assert any("99.0ms" in w and "9.652" in w for w in warnings), warnings
    assert any("0.500" in w and "0.5263" in w for w in warnings), warnings


NEW_SUBSTITUTIONS = (
    "extra_us_label",
    "extra_us_text_edit_plain",
    "extra_us_text_edit_hint",
    "extra_us_slider_many_options",
    "per_widget_us_python_side",
)


def test_each_per_widget_number_is_individually_checked(tmp_path):
    """Every new substitution must actually be compared, not merely defined.

    Asserted one at a time because a check that silently stopped covering one
    of them would still pass a single "the gate warns" test, and the page would
    go on quoting a number nothing checked.
    """
    snapshot = json.loads(json.dumps(SNAPSHOT))
    bumps = {
        "extra_us_label": ("label", "extra_us_median"),
        "extra_us_text_edit_plain": ("text_edit_plain", "extra_us_median"),
        "extra_us_text_edit_hint": ("text_edit_hint", "extra_us_median"),
        "extra_us_slider_many_options": ("slider_many_options", "extra_us_median"),
    }
    for name in NEW_SUBSTITUTIONS:
        page = "\n".join(
            line for line in MATCHING_PAGE.splitlines()
            if not line.startswith(f".. |{name}|")
        ) + "\n"
        sp, pp = write(tmp_path, snapshot, page)
        warnings = []
        check_performance_page(sp, pp, warnings)
        assert any(name in w and "never defined" in w for w in warnings), (
            f"|{name}| is not covered by the gate: dropping it warned nothing"
        )

    # And the positive direction: a wrong value must warn with both numbers.
    for name, (scenario, key) in bumps.items():
        snapshot2 = json.loads(json.dumps(SNAPSHOT))
        snapshot2["by_scenario"][scenario]["2000"]["comparison"][key] = 9.99
        sp, pp = write(tmp_path, snapshot2, MATCHING_PAGE)
        warnings = []
        check_performance_page(sp, pp, warnings)
        assert any(name in w and "9.99" in w for w in warnings), (
            f"|{name}| drifted from the snapshot but the gate said nothing"
        )

    # python_side's per-widget figure, the same two directions.
    snapshot3 = json.loads(json.dumps(SNAPSHOT))
    snapshot3["by_scenario"]["python_side"]["2000"]["python_only"][
        "pyegui_per_widget_us_median"] = 9.99
    sp, pp = write(tmp_path, snapshot3, MATCHING_PAGE)
    warnings = []
    check_performance_page(sp, pp, warnings)
    assert any("per_widget_us_python_side" in w and "9.99" in w
               for w in warnings), warnings


def test_every_substitution_the_page_declares_is_checked():
    """No declared substitution may sit outside the gate's reach.

    The page claims, in prose, exactly which of its numbers are checked. This
    asserts the mechanical half of that claim: the set of names the gate
    compares is the set of names the page declares, plus the two provenance
    tokens, which carry no machine-checkable value and are presence-checked.
    """
    from check_performance_page import REQUIRED, SUBSTITUTIONS

    checked = {name for names in SUBSTITUTIONS.values() for name in names}
    checked |= {"import_ms", "frame_ms_python_side", "per_widget_us_python_side"}
    page = (Path(__file__).resolve().parent.parent / "docs" / "performance.rst").read_text(
        encoding="utf-8"
    )
    declared = set(re.findall(r"^\.\. \|(\w+)\| replace:: ", page, re.M))
    assert declared == checked | REQUIRED, (
        "the page declares substitutions the gate does not check, or names the "
        "gate checks that the page does not define",
        sorted(declared - checked - REQUIRED),
        sorted(checked - declared),
    )


def test_no_comparison_key_is_not_invented_into_a_ratio(tmp_path):
    """python_side has no Rust twin, so there is no ratio to check.

    Asserted because the tempting mistake is to derive one from the frame
    time and print a number no measurement supports.
    """
    sp, pp = write(tmp_path, SNAPSHOT, MATCHING_PAGE)
    warnings = []
    check_performance_page(sp, pp, warnings)
    assert not any("python_side" in w and "ratio" in w for w in warnings), warnings


# The gate's one warning shape for "both sides have this number and they are
# not equal", produced by `_compare`:
#
#   docs/performance.rst: |ratio_label| says 1.99x, the snapshot says 1.37x
#   (scenario 'label', 2000 widgets/frame).
#
# Anchored and closed so it matches that shape and nothing else. Every other
# warning the gate can emit names something *missing* -- an absent scenario, an
# unparseable snapshot, an undeclared or absent token, a python_side entry
# with no python_only -- and those are defects in the gate or the page that a
# human has to fix, not benchmark noise.
_DRIFT = re.compile(
    r"^docs/performance\.rst: \|\w+\| says .+, the snapshot says .+ \(.+\)\.$"
)


def _is_drift(warning):
    return _DRIFT.match(warning) is not None


_PLACEHOLDER = "Zq7"


def _render_message(expr):
    """Render one `_warn` message expression with every hole stubbed.

    Each name the expression reads (a page path, an exception, a substitution
    name, `type(...)`) is bound to the same neutral token, and the gate's own
    module globals are supplied too so a template that interpolates a constant
    like `WIDGET_COUNT` still reads as the gate wrote it. The expression is
    then evaluated unmodified -- not rewritten -- so the prose, the
    punctuation, and any conditional the gate concatenates survive exactly as
    authored. Only the *content* of the holes is pinned down; the *shape* is
    what is under test.
    """
    import builtins

    import check_performance_page as gate

    # A hole is a name that resolves to a *runtime value*: a gate local (a page
    # path, an exception, a substitution name) or an attribute of one. Names
    # that resolve to a module constant or a builtin are part of the message's
    # shape and are left alone, so `WIDGET_COUNT` still reads as 2000 and
    # `type(...)` still formats a type name.
    resolved = set(vars(gate)) | set(vars(builtins))
    stubs = {n.id for n in ast.walk(expr)
             if isinstance(n, ast.Name) and n.id not in resolved}
    namespace = dict(vars(gate))
    namespace.update({name: _PLACEHOLDER for name in stubs})
    # An expression parsed from our own module's AST, with every name it
    # reads bound above.
    code = compile(ast.Expression(body=expr), "check_performance_page.py", "eval")
    return eval(code, namespace)  # noqa: S307


def _warn_message_templates():
    """Every `_warn` call site's message, read from the gate's source.

    Parsed rather than imported-and-exercised on purpose: this has to hold
    for messages the gate cannot reach from any fixture, and it must not read
    docs/performance.rst or bench/results/combined.json -- or it would only
    ever classify the shapes the committed pair happens to produce, which is
    the vacuity this test exists to remove.
    """
    source = Path(__file__).resolve().parent / "check_performance_page.py"
    tree = ast.parse(source.read_text(encoding="utf-8"))
    rendered = []
    for node in ast.walk(tree):
        if not (isinstance(node, ast.Call)
                and getattr(node.func, "id", None) == "_warn"):
            continue
        assert len(node.args) >= 2, (
            f"tests/check_performance_page.py:{node.lineno} calls _warn without "
            "a message; update _warn_message_templates to match"
        )
        expr = ast.parse(ast.unparse(node.args[1]), mode="eval").body
        rendered.append((node.lineno, _render_message(expr)))
    return rendered


def test_every_structural_message_is_non_drift_and_drift_still_matches():
    """Pin the drift/structural split to the gate, not to the artifacts.

    WHY THIS EXISTS, INSTEAD OF RELYING ON THE REAL PAIR:

    `test_the_real_page_and_the_real_snapshot_agree` asserts the committed
    page/snapshot pair has no structural warnings. Today that pair produces no
    warnings at all, so that assertion is `assert not []` -- it cannot detect
    anything, and it never will while the two files agree.

    The hole it leaves: if a commit broadens `_DRIFT` (drops the `\\.rst: `
    anchor, or the trailing `\\(.+\\)\\.` clause) *in the same commit* that
    introduces a structural defect, that defect reclassifies as tolerated
    drift, the structural list stays empty, and the job goes green with a
    broken gate. No other test reads `_DRIFT`, so nothing else notices.

    So this pins the classification directly, from the gate's own source:
    every message shape it can emit must be non-drift, and the one synthetic
    drift message must be drift. The shapes come from walking `_warn` call
    sites, so a new call site is classified here too -- the next structural
    message cannot escape unpinned by being added to the gate.
    """
    templates = _warn_message_templates()
    assert templates, (
        "no _warn call sites were found in tests/check_performance_page.py -- "
        "the gate's messages are no longer built through _warn, so this test "
        "has silently stopped classifying anything"
    )

    # Independent cross-check that the AST walk saw every call site: count the
    # raw textual calls too. A disagreement means a site exists in a form the
    # walk skips, and this test would be pinning a subset without saying so.
    source_text = (Path(__file__).resolve().parent
                   / "check_performance_page.py").read_text(encoding="utf-8")
    textual = len(re.findall(r"(?<!def )\b_warn\(", source_text))
    assert textual == len(templates), (
        f"found {textual} textual _warn( call sites but rendered "
        f"{len(templates)} messages; one of them is not being classified"
    )

    # The table. One row per `_warn` call site, keyed by source line so a
    # failure names the site rather than a pile of text, and each row asserted
    # individually: a single `assert not [...]` over the whole table would
    # pass if the classifier ever returned None for everything, which is
    # exactly the failure mode this test is here to catch.
    table = {f"check_performance_page.py:{ln}": (msg, _is_drift(msg))
             for ln, msg in templates}
    assert len(table) == len(templates), (
        "two _warn call sites render to the same table key, so one of them is "
        "not getting its own row"
    )

    for key, (msg, is_drift) in sorted(table.items()):
        # Exactly one site -- `_compare`'s number mismatch -- is drift.
        # Everything else the gate can say names something missing, which is a
        # defect a human has to fix.
        assert is_drift == ("says" in msg and "the snapshot says" in msg), (
            f"{key} is classified {'drift' if is_drift else 'structural'}, which "
            f"is the wrong side of the split for this message: {msg}"
        )

    drift_shaped = [msg for msg, is_drift in table.values() if is_drift]
    assert len(drift_shaped) == 1, (
        "exactly one _warn call site may produce a message _DRIFT matches -- "
        "the number mismatch in _compare. If a structural message now matches, "
        "the drift tolerance is hiding a defect that must fail the build: "
        f"{sorted(msg for msg, d in table.values() if d)}"
    )

    # And the positive direction, stated explicitly rather than only inferred
    # from the table above: a message of the documented drift shape must be
    # classified drift, or the structural half would be failing on the
    # ordinary benchmark variation this tolerance exists to absorb.
    assert _is_drift(
        "docs/performance.rst: |ratio_label| says 1.99x, the snapshot says "
        "1.33x (scenario 'label', 2000 widgets/frame)."
    ), "_DRIFT no longer matches the drift shape it exists to tolerate"
    assert not _is_drift(
        "docs/performance.rst: |ratio_label| says 1.99x, the snapshot says "
        "1.33x"
    ), (
        "_DRIFT matches an unanchored number mismatch -- a truncated message "
        "would be tolerated as drift"
    )


def test_the_real_page_and_the_real_snapshot_agree(tmp_path):
    """The committed pair, split into benchmark drift and structural defects.

    WHY THE SPLIT, AND WHY NOT `assert not warnings`:

    Between-run variation on hosted runners is 1.30x-1.55x for identical
    code, so the numbers in `docs/performance.rst` and the numbers in
    `bench/results/combined.json` will legitimately disagree after any future
    benchmark run. `examples` now runs this suite, so demanding zero warnings
    here makes every ordinary benchmark run red -- which inverts the
    warn-not-fail design of the gate itself, and teaches people to ignore a
    red build. Do not "tighten" this back to `assert not warnings`.

    So: a NUMBER THAT DIFFERS is tolerated drift; everything else -- a
    scenario missing from the snapshot, an unparseable snapshot, an absent or
    undeclared substitution token, a missing required token, a python_side
    entry without python_only -- is a structural defect and must still fail.

    The second half of the test is what keeps this from decaying into
    vacuity: it re-runs the gate against the real snapshot with one number in
    the real page perturbed, and requires that the gate notice. If the gate
    ever stopped comparing numbers, that perturbed run would warn about
    nothing and this test would fail.
    """
    root = Path(__file__).resolve().parent.parent
    snapshot = root / "bench" / "results" / "combined.json"
    page = root / "docs" / "performance.rst"

    warnings = []
    check_performance_page(snapshot, page, warnings)
    structural = [w for w in warnings if not _is_drift(w)]
    assert not structural, (
        "the committed page/snapshot pair has structural defects: "
        + "; ".join(structural)
    )

    # Non-vacuity: a wrong number must still be reported, and must land in
    # the drift bucket rather than the structural one. Perturbing a copy of
    # the real page leaves every other number exactly as committed, so any
    # warning this produces is a consequence of the perturbation alone.
    original_text = page.read_text(encoding="utf-8")
    perturbed_text = re.sub(
        r"(^\.\. \|ratio_label\| replace:: )\S+",
        r"\g<1>9.99x",
        original_text,
        count=1,
        flags=re.M,
    )
    assert perturbed_text != original_text, (
        "could not perturb |ratio_label| in the real page, so the drift check "
        "below would pass without exercising anything"
    )
    perturbed = tmp_path / "performance.rst"
    perturbed.write_text(perturbed_text, encoding="utf-8")
    drifted = []
    check_performance_page(snapshot, perturbed, drifted)
    assert drifted, (
        "the gate reported nothing for a page whose |ratio_label| is 9.99x "
        "against a snapshot saying otherwise -- it has stopped checking "
        "numbers, so the split above no longer means anything"
    )
    assert all(_is_drift(w) for w in drifted), (
        "perturbing one number produced warnings that are not number "
        "mismatches, which means the drift tolerance would be hiding a "
        f"structural defect: {drifted}"
    )


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
            "by_scenario": {"label": {"2000": {"comparison": "1.33x"}}}
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
