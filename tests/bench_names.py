"""The benchmark scenario names, on the Python side.

Duplicated in `bench/src/main.rs`'s `scenario()` because a Rust binary cannot
read a Python module at runtime. `tests/test_scenario_names_agree.py` asserts
the two lists agree, since drift here is otherwise silent: both halves would
each reject a scenario the other accepts, and the failure would look like a
benchmark bug rather than a naming mismatch.
"""

VALID_SCENARIOS = ["label", "text_edit_plain", "text_edit_hint", "python_side"]