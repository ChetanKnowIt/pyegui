"""The two halves of the benchmark must accept the same scenario names.

`bench/bench.py` and `bench/src/main.rs` each carry their own list of valid
scenario names, because the Rust binary is launched as a subprocess and cannot
import the Python module that defines them. The duplication is unavoidable; the
divergence is not, and it would be silent: each half would reject a name the
other accepts, and the failure would surface as a benchmark that mysteriously
refuses to run.

`python_side` is the one deliberate asymmetry, asserted explicitly below so
that a future change to it is a test failure rather than a silent widening of
what the Rust binary pretends to support.
"""

import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(REPO_ROOT / "tests"))

import bench_names  # noqa: E402


def _rust_scenario_arms():
    rust = (REPO_ROOT / "bench" / "src" / "main.rs").read_text(encoding="utf-8")
    # The names appear as the match arms of `fn scenario()`.
    return set(re.findall(r'"(\w+)" => Ok', rust))


def test_rust_lists_the_same_scenarios_as_python():
    arms = _rust_scenario_arms()
    # python_side is deliberately absent from the Rust match: it has no twin.
    assert arms == {"label", "text_edit_plain", "text_edit_hint"}
    assert arms | {"python_side"} == set(bench_names.VALID_SCENARIOS)


def test_python_side_rejects_its_own_valid_name():
    # Asserted because it is the one asymmetry in the design: python_side is
    # valid on the Python side and rejected by Rust. If that ever changes, the
    # assertion above is what notices.
    assert "python_side" not in _rust_scenario_arms()


def test_bench_py_agrees_with_the_shared_list():
    """`bench/bench.py` cannot import from tests/ at runtime -- it runs from the
    repo root against an installed wheel -- so its list is a third copy. Checked
    here rather than by importing it, since importing bench.py would run it.
    """
    source = (REPO_ROOT / "bench" / "bench.py").read_text(encoding="utf-8")
    match = re.search(r"^VALID_SCENARIOS = (\[[^\]]*\])", source, re.M)
    assert match, "bench/bench.py no longer defines VALID_SCENARIOS"
    assert eval(match.group(1)) == bench_names.VALID_SCENARIOS  # noqa: S307