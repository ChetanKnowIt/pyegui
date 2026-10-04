"""The benchmark scenario names, on the Python side.

Duplicated in `bench/src/main.rs`'s `scenario()` because a Rust binary cannot
read a Python module at runtime. `tests/test_scenario_names_agree.py` asserts
*those two* lists agree, since drift between them is otherwise silent: both
halves would each reject a scenario the other accepts, and the failure would
look like a benchmark bug rather than a naming mismatch.

This is one of four copies of the list. The others are this file,
`bench/bench.py`, `scenario()` in `bench/src/main.rs`, and the `VALID_SCENARIOS`
and `TWINLESS` lists in `.github/workflows/benchmark.yml`. The test suite only
covers `bench/bench.py` against Rust; the shell lists and this file are not
compared to anything today, and unifying all of them is a design change
rather than a documentation fix.
"""

VALID_SCENARIOS = [
    "label",
    "text_edit_plain",
    "text_edit_hint",
    "slider_many_options",
    "python_side",
]