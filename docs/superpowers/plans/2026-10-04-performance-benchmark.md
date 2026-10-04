# Performance guide and second benchmark scenario — implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Measure what an app author can control — the `**kwargs` per-call tax and their own Python-side work — and publish `docs/performance.rst` from a committed benchmark snapshot that a documentation gate reads.

**Architecture:** `bench/` gains scenarios beyond the existing `label` baseline, selected by a `SCENARIO` env var that both language halves read. `text_edit_plain` / `text_edit_hint` get Rust twins and are compared ratio-against-ratio; `python_side` has no twin and is reported as an absolute per-frame cost under a separate `python_only` key with no ratio computed. The benchmark workflow commits `combined.json` back to the branch using its own `GITHUB_TOKEN`, and `docs/performance.rst` renders its numbers through RST substitutions read from that snapshot.

**Tech Stack:** Rust (eframe/egui 0.31.1, pyo3), Python 3.11, reStructuredText + Sphinx, GitHub Actions.

**Spec:** `docs/superpowers/specs/2026-10-04-performance-benchmark-design.md` — read it alongside this plan.

## Global Constraints

- **No local Rust toolchain.** No `rustup`, `cargo`, `maturin` or `docker` locally, and none may be installed. Every Rust build claim must cite a workflow run. Do not add a local build step.
- **`bench/` is a separate Cargo workspace** (`bench/Cargo.toml` declares its own `[workspace]`), so `check.yml`'s `cargo check` never compiles it. The only verification for changes under `bench/` is a `benchmark` run.
- **`origin` is read-only.** Push only to `fork` (`ChetanKnowIT/pyegui`).
- **Push once per task**, so `check` and `examples` all read the same commit.
- **`check` must be green before each commit.**
- **The documentation gate runs in the `examples` workflow, not `check`** — `examples.yml:184` runs `python tests/doc_claims.py`. Anything added to that file is verified by an `examples` run.
- **`benchmark` is `workflow_dispatch` only** and must be dispatched with `--ref feature/egui-0.31-coverage`, or it runs against `main`.
- **`GITHUB_TOKEN` commits do not trigger workflow runs** (GitHub's recursion guard). The snapshot commit fires neither `check` nor `examples`, so neither gate re-validates the snapshot it just committed.
- **No public names are added by this plan**, so `tests/expected_exports.py` is unaffected.
- **Preserve the existing claim discipline:** no launch-time figure, no first-frame figure folded into a steady-state average, no blended single score. These choices are recorded in `TODO.md` §7 and carry to the new page.

## Review Focus

Five failure modes a person would reasonably hit, which the spec implies but no test directly exercises. Most likely first.

1. **`combined.json` missing entirely** — a fresh clone has no snapshot, since a later run commits it. The gate must warn and exit 0, not raise `FileNotFoundError` and fail the `examples` job. Pinned by Task 5 Step 3.
2. **`combined.json` present but stale** — the realistic case after a human edits the page without dispatching a run. Must warn naming the scenario, not crash on a missing key. Pinned by Task 5 Step 4.
3. **`python_side` has no Rust result file** — the combine step looks up `egui-<count>-t<trial>.json` by name, which does not exist for a twin-less scenario. It must record under `python_only` and compute no ratio, rather than raising `SystemExit("missing Rust result")`. Pinned by Task 4 Step 5.
4. **The `examples` job's `doc_status`** — `examples.yml` runs `doc_claims.py || doc_status=$?` and fails the job if it is non-zero. A warn-only gate that returns 1 would silently turn `examples` red. Pinned by Task 5 Step 5.
5. **`SCENARIO` set to an unknown value** — the workflow passes it to both halves; a typo would make one side fall back to `label` and the other measure something else, which is exactly the workload drift the parity guard exists to catch. Both halves must reject it rather than default. Pinned by Task 2 Step 5.

---

### Task 1: De-duplicate the benchmark workflow and grant it write permission

Removes a real duplication bug and unlocks the snapshot commit Task 3 adds. Nothing else changes here, so a later failure cannot be blamed on this task.

**Files:**
- Modify: `.github/workflows/benchmark.yml` — the `permissions:` block (currently lines 30-31), and the duplicated block at lines 144-203

**Interfaces:**
- Consumes: nothing.
- Produces: `permissions: contents: write` at workflow level; the `import pyegui` measurement, the `WIDGET_COUNTS` validation and the release build each appearing exactly once. Task 3 appends a commit step after the existing ones.

- [ ] **Step 1: Confirm the duplication before deleting anything**

```bash
grep -c 'import pyegui:' .github/workflows/benchmark.yml
grep -c 'cargo build --release --quiet' .github/workflows/benchmark.yml
grep -c 'widget counts must be positive integers' .github/workflows/benchmark.yml
```

Expected: `3`, `2`, `3`. If any count is `1`, an earlier commit already fixed it — stop and report rather than deleting by line number.

- [ ] **Step 2: Delete the second and third copies**

Delete from the second occurrence of the comment

```
      # import cost, in a fresh interpreter so the module is not already up
```

through the second occurrence of

```
      test -x bench/target/release/pyegui-bench
```

inclusive, plus the blank lines between. The step must end with the first `WIDGET_COUNTS` validation, the first import measurement, the first `cargo build --release --quiet`, the first `test -x`, then the existing trials comment block beginning

```
          # Trials are the outer loop and widget counts the inner one, so every
```

Keep every first occurrence. Delete both later duplicates.

- [ ] **Step 3: Verify one of each remains and the YAML still parses**

```bash
grep -c 'import pyegui:' .github/workflows/benchmark.yml
grep -c 'cargo build --release --quiet' .github/workflows/benchmark.yml
grep -c 'widget counts must be positive integers' .github/workflows/benchmark.yml
python -c "import yaml; d=yaml.safe_load(open('.github/workflows/benchmark.yml')); print(len(d['jobs']['benchmark']['steps']),'steps')"
```

Expected: `1`, `1`, `1`, and a step count lower than before. A YAML parse error means the block boundary was wrong.

- [ ] **Step 4: Grant write permission**

Change:

```yaml
permissions:
  contents: read
```

to:

```yaml
permissions:
  contents: write
```

No PAT is involved. `GITHUB_TOKEN` is scoped to the repository the workflow runs in, which is the fork; the `contents: read` declaration was the only obstacle.

- [ ] **Step 5: Commit and push**

```bash
git add .github/workflows/benchmark.yml
git commit -m "De-duplicate the benchmark workflow and allow it to commit results

Lines 144-203 repeated the import-cost measurement, the widget-count
validation and the release build that lines 110-141 already performed.
Both copies ran and both were idempotent, so results were unaffected,
but it was a duplicated block arrived at by editing.

contents: write is needed by the snapshot commit added later."
git push fork HEAD:feature/egui-0.31-coverage
```

- [ ] **Step 6: Confirm only the expected workflows fired**

```bash
gh run list --repo ChetanKnowIT/pyegui --branch feature/egui-0.31-coverage -L 3 --json databaseId,name,status,conclusion
```

Expected: `check` and `examples` for the new commit. `benchmark` is `workflow_dispatch` only, so its absence is correct.

---

### Task 2: Make both benchmark halves reject an unknown SCENARIO

The two halves cannot share one list at runtime — Rust cannot read a Python file — so each declares its own list, and Task 5's gate asserts the two agree. What matters here is that both reject rather than default.

**Files:**
- Modify: `bench/bench.py` — after the `WIDGETS_PER_FRAME` constant at line 50
- Modify: `bench/src/main.rs` — after `widgets_per_frame()`, which ends at line 37

**Interfaces:**
- Consumes: nothing.
- Produces: both halves expose the same four valid scenario names — `label`, `text_edit_plain`, `text_edit_hint`, `python_side` — and exit non-zero naming them on an unknown value. `bench/bench.py` reads the selection from `SCENARIO` (default `"label"`); `bench/src/main.rs` exposes `fn scenario() -> Result<&'static str, String>`. Task 4 dispatches `SCENARIO` and Task 5 checks the two lists agree.

- [ ] **Step 1: Add the rejection to `bench/bench.py`**

After the `WIDGETS_PER_FRAME` constant, add:

```python
# The scenario is an env var for the same reason WIDGETS is: both halves read
# it, so a typo cannot make one side measure something other than the other.
# An unknown value is rejected rather than defaulted -- a silent fallback to
# `label` would produce a mismatched pair whose ratio looks like a real one.
SCENARIO = os.environ.get("SCENARIO", "label")
VALID_SCENARIOS = ["label", "text_edit_plain", "text_edit_hint", "python_side"]

if SCENARIO not in VALID_SCENARIOS:
    raise SystemExit(
        f"unknown SCENARIO '{SCENARIO}'; valid scenarios are "
        + ", ".join(VALID_SCENARIOS)
    )
```

- [ ] **Step 2: Verify the rejection works**

Run: `SCENARIO=nope python bench/bench.py`

Expected: exit non-zero with `unknown SCENARIO 'nope'; valid scenarios are label, text_edit_plain, text_edit_hint, python_side`. The module will not import (pyegui is not installed locally), so if the failure is `ModuleNotFoundError` instead, that is a pass for the purposes of this step — the check cannot run before the wheel is installed, and CI Task 4's dispatch is the real verification. Record which of the two happened.

Run: `SCENARIO=label python -c "import ast;ast.parse(open('bench/bench.py').read())"`

Expected: no output, exit 0. This is the part that is actually verifiable locally.

- [ ] **Step 3: Add the rejection to `bench/src/main.rs`**

After `widgets_per_frame()`, add:

```rust
/// The scenario under test, read from the environment for the same reason
/// `WIDGETS` is: both halves read it, so a typo cannot make one side measure
/// something other than the other.
///
/// An unknown value is rejected rather than defaulted. A silent fallback to
/// `label` would produce a mismatched pair whose ratio looks like a real one,
/// which is the failure the combine step's parity guard exists to catch.
///
/// The names are duplicated from `bench/bench.py` rather than shared, because
/// a Rust binary cannot read the Python module at runtime. `tests/doc_claims.py`
/// asserts the two lists agree, since drift here is silent otherwise.
fn scenario() -> Result<&'static str, String> {
    let raw = std::env::var("SCENARIO").unwrap_or_else(|_| "label".to_string());
    match raw.as_str() {
        "label" => Ok("label"),
        "text_edit_plain" => Ok("text_edit_plain"),
        "text_edit_hint" => Ok("text_edit_hint"),
        _ => Err(format!(
            "unknown SCENARIO '{raw}'; valid scenarios are label, \
             text_edit_plain, text_edit_hint, python_side"
        )),
    }
}
```

`python_side` is deliberately absent from the match: it has no Rust twin, so the Rust binary must reject it rather than measure a substitute workload. The error message still names it as a valid scenario name, because it is valid on the Python side.

- [ ] **Step 4: Call it from `main()` and fail the run**

In `main()`, before `eframe::run_native`, add:

```rust
    if let Err(message) = scenario() {
        eprintln!("{message}");
        std::process::exit(2);
    }
```

- [ ] **Step 5: Confirm the Rust half is syntactically plausible without cargo**

There is no local Rust toolchain, so this cannot be compiled. Verify only what is checkable:

```bash
grep -n 'fn scenario' bench/src/main.rs
grep -c 'fn scenario' bench/src/main.rs
```

Expected: one match, count `1`. A count above 1 means a duplicate definition, which would fail `cargo check` in CI. Compilation is verified by Task 4's `benchmark` dispatch.

- [ ] **Step 6: Commit and push**

```bash
git add bench/bench.py bench/src/main.rs
git commit -m "Reject an unknown SCENARIO in both benchmark halves

A typo in SCENARIO would make one side fall back to label and the other
measure something else, producing a mismatched pair whose ratio looks
like a real one. Both halves now exit non-zero naming the valid
scenarios. The names are duplicated rather than shared because a Rust
binary cannot read the Python module at runtime."
git push fork HEAD:feature/egui-0.31-coverage
```

---

### Task 3: Commit the results snapshot from the benchmark workflow

The gate in Task 5 needs a committed snapshot to read. This task adds only the commit step; no scenarios exist yet, so it commits the current `label`-only output.

**Files:**
- Modify: `.github/workflows/benchmark.yml` — a new step after `Summarise`, before `upload-artifact`

**Interfaces:**
- Consumes: `bench/results/combined.json` as written by the existing `Combine and compare` step; `permissions: contents: write` from Task 1 Step 4.
- Produces: a commit on `feature/egui-0.31-coverage` adding `bench/results/combined.json`. Task 5's gate reads that file at `bench/results/combined.json`.

- [ ] **Step 1: Confirm the results directory is currently untracked**

```bash
git ls-files bench/results
cat .gitignore 2>/dev/null | grep -n 'results'
```

Expected: `git ls-files` prints nothing. If `.gitignore` lists `bench/results`, remove that line in this task — a gitignored path cannot be committed, and the commit step would fail every run.

- [ ] **Step 2: Add the commit step**

Immediately before the existing `- uses: actions/upload-artifact@v4` step, add:

```yaml
      # Commit the snapshot the documentation gate reads. Without it the page's
      # numbers have nothing to be checked against, and the page rots silently
      # while every run passes.
      #
      # `GITHUB_TOKEN` needs no secret: it is scoped to this repository, which
      # is the fork this workflow already pushes to. `contents: write` above is
      # the only requirement.
      #
      # A commit made with `GITHUB_TOKEN` does not trigger workflow runs, by
      # GitHub's recursion guard, so this cannot loop and cannot turn its own
      # build red.
      - name: Commit the results snapshot
        run: |
          set -euo pipefail
          if git diff --quiet -- bench/results/combined.json \
             && git ls-files --error-unmatch bench/results/combined.json \
                >/dev/null 2>&1; then
            echo "snapshot unchanged"
            exit 0
          fi
          git config user.name "github-actions[bot]"
          git config user.email "41898282+github-actions[bot]@users.noreply.github.com"
          git add bench/results/combined.json
          git commit -m "benchmark: results snapshot from run ${{ github.run_id }}"
          for attempt in 1 2 3; do
            if git push; then
              echo "snapshot pushed"
              exit 0
            fi
            echo "push attempt ${attempt} failed; retrying"
            sleep $((attempt * 5))
          done
          echo "could not push the snapshot after 3 attempts" >&2
          exit 1
```

- [ ] **Step 3: Verify the step's shell logic locally**

The step cannot be run locally — it needs a `combined.json` from a benchmark run. Verify the pieces that are checkable:

```bash
python -c "
import yaml
d = yaml.safe_load(open('.github/workflows/benchmark.yml'))
steps = d['jobs']['benchmark']['steps']
names = [s.get('name', s.get('uses','')) for s in steps]
print(names)
"
```

Expected: a list whose second-to-last entry is `Commit the results snapshot` and last is `upload-artifact`. If the YAML parses and the ordering holds, the step is well-formed.

- [ ] **Step 4: Commit and push**

```bash
git add .github/workflows/benchmark.yml .gitignore
git commit -m "Commit the benchmark results snapshot from the workflow

docs/performance.rst will render its numbers from a committed snapshot,
and tests/doc_claims.py will warn when the page and the data disagree.
Without a committed file the gate has nothing to read and the page rots
while every run passes."
git push fork HEAD:feature/egui-0.31-coverage
```

Only include `.gitignore` in this commit if Step 1 found and you removed a `bench/results` entry.

- [ ] **Step 5: Verify by dispatching a benchmark run**

```bash
gh workflow run benchmark --repo ChetanKnowIT/pyegui \
  --ref feature/egui-0.31-coverage -f widgets='50' -f trials='1'
```

Then watch it:

```bash
gh run list --repo ChetanKnowIT/pyegui --branch feature/egui-0.31-coverage -L 1 \
  --json databaseId,name,status,conclusion
gh run watch <run-id> --repo ChetanKnowIT/pyegui --exit-status
```

Expected: success, and `git fetch fork && git log fork/feature/egui-0.31-coverage -1 --stat` shows a commit by `github-actions[bot]` adding `bench/results/combined.json`.

This run is also the first verification of Task 1 and Task 2's Rust half, since `bench/` only compiles here. If it fails on `fn scenario`, the error is in Task 2 Step 3.

---

### Task 4: Add the text_edit and python_side scenarios

**Files:**
- Modify: `bench/bench.py` — `make_update()`, lines 58-98
- Modify: `bench/src/main.rs` — `impl eframe::App for Bench::update`, lines 50-84
- Modify: `.github/workflows/benchmark.yml` — the `Run both benchmarks` step, to pass `SCENARIO`

**Interfaces:**
- Consumes: `SCENARIO` validation from Task 2; the snapshot commit from Task 3.
- Produces: every result payload gains `"scenario": "<name>"` and `"has_rust_twin": <bool>`. `bench/bench.py`'s `main()` gains a `python_side` branch whose payload sets `has_rust_twin: false`. `combined.json` gains `by_scenario` alongside `by_widget_count`, and a `python_only` key for twin-less scenarios. Task 5's gate reads `by_scenario`.

- [ ] **Step 1: Dispatch the `text_edit` scenarios from the workflow**

In the `Run both benchmarks` step's `env:` block, after `TRIALS:`, add:

```yaml
          SCENARIO: ${{ inputs.scenario }}
```

And add a third dispatch input to the workflow's `on.workflow_dispatch.inputs`, after `trials`:

```yaml
      scenario:
        description: 'Benchmark scenario: label, text_edit_plain, text_edit_hint or python_side'
        required: false
        default: 'label'
```

- [ ] **Step 2: Make `bench/bench.py` measure each scenario**

Replace the body of `contents` inside `make_update` with:

```python
    def contents():
        # The widgets are the thing being measured. They need a Ui, and this
        # function composes its own frame, so the labels go inside a panel
        # rather than at the top level.
        t0 = time.perf_counter()

        if SCENARIO == "label":
            for i in range(WIDGETS_PER_FRAME):
                pyegui.label(f"row {i}")
        elif SCENARIO == "text_edit_plain":
            # No options passed: the kwargs path is still taken, because the
            # signature takes **kwargs whether or not the caller supplies any.
            for _ in range(WIDGETS_PER_FRAME):
                pyegui.text_edit_singleline_response(name)
        elif SCENARIO == "text_edit_hint":
            for _ in range(WIDGETS_PER_FRAME):
                pyegui.text_edit_singleline_response(name, hint_text="name")
        elif SCENARIO == "python_side":
            for i in range(WIDGETS_PER_FRAME):
                # Author-side work, not binding work: an f-string, an
                # attribute read and arithmetic, accumulating into a value
                # that is read back so nothing is optimised away.
                python_side_sink = f"{name.value}-{i}" + str(i)
                name.value = python_side_sink[:0] or str(i)
        else:  # pragma: no cover - Task 2 rejects this at import
            raise SystemExit(f"unhandled SCENARIO {SCENARIO!r}")

        elapsed = (time.perf_counter() - t0) * 1000
```

`name` is a module-level `pyegui.Str("")` added next to `WIDGETS_PER_FRAME`, since `text_edit_*` takes a `Str` and `python_side` reads `.value`.

- [ ] **Step 3: Report `scenario` and `has_rust_twin` in the payload**

In `main()`'s `results` dict, after `"widgets_per_frame"`, add:

```python
        "scenario": SCENARIO,
        # python_side measures the author's own work, so there is no Rust
        # twin to divide by. The combine step must not invent a ratio for it.
        "has_rust_twin": SCENARIO != "python_side",
```

- [ ] **Step 4: Make `bench/src/main.rs` measure each scenario**

In `Bench`, add a field beside `widgets`:

```rust
    // Read once in main, not per frame: `std::env` in the measured path would
    // be this benchmark's own overhead rather than the binding's.
    scenario: &'static str,
```

In `main()`, inside the `Bench { .. }` literal, add `scenario: scenario()?` — note the `?` propagates the `Result` out of `main`, which already returns `eframe::Result`.

Replace the body of the `CentralPanel::default().show` closure:

```rust
        egui::CentralPanel::default().show(ctx, |ui| {
            let t0 = Instant::now();

            match self.scenario {
                "label" => {
                    for i in 0..self.widgets {
                        ui.label(format!("row {i}"));
                    }
                }
                "text_edit_plain" => {
                    for _ in 0..self.widgets {
                        ui.text_edit_singleline(&mut name);
                    }
                }
                "text_edit_hint" => {
                    for _ in 0..self.widgets {
                        ui.text_edit_singleline(&mut name).hint_text("name");
                    }
                }
                _ => {}
            }

            let elapsed = t0.elapsed().as_secs_f64() * 1000.0;
```

`name` is `let mut name = String::new();` declared above the `eframe::run_native` call in `main`, alongside `widgets_per_frame()`'s result — it must be a single binding mutated across frames, matching the Python side's `Str`.

An empty `_ => {}` arm is correct rather than defensive: `scenario()` already rejected every other value, and this arm makes the match total without re-checking.

- [ ] **Step 5: Teach the combine step about scenarios and twin-less entries**

In the `Combine and compare` heredoc, after the line that reads `imp = json.loads(...)`, add:

```python
          # Scenarios without a Rust twin are recorded, not compared. A ratio
          # against a baseline that does not exist is exactly the meaningless
          # number the parity guard below exists to prevent, and `python_side`
          # would otherwise trip the "missing Rust result" exit.
          TWINLESS = {"python_side"}
```

Then, immediately after the existing `for field in ("widgets_per_frame", "frames_measured"):` parity check, add:

```python
                  if pyegui["scenario"] != egui["scenario"]:
                      raise SystemExit(
                          f"scenario mismatch at {count} widgets, trial "
                          f"{trial}: pyegui measured "
                          f"{pyegui['scenario']}, egui measured "
                          f"{egui['scenario']} -- the ratio would be "
                          "comparing two different workloads"
                      )
```

And change the trial-glob patterns so the scenario is in the filename, matching the new files `pyegui-<scenario>-<count>-t<trial>.json`:

```python
          pattern = re.compile(r"pyegui-(\w+)-(\d+)-t(\d+)\.json$")
```

with the existing assignment changed to:

```python
              scenario, count, trial = m.group(1), int(m.group(2)), int(m.group(3))
              trials.setdefault(scenario, {}).setdefault(count, {})[trial] = json.loads(path.read_text())
```

This changes the shape of `trials` from `{count: {trial: …}}` to `{scenario: {count: {trial: …}}}`, so **every** loop over `trials` in the step needs the extra nesting level: the `for count in sorted(trials)` header becomes `for scenario in sorted(trials):` wrapping a `for count in sorted(trials[scenario]):`, and the legacy single-trial fallback assigns into `trials.setdefault("label", {})`. The `by_count[str(count)] = entry` assignment moves under a new outer `by_scenario` dict:

```python
          combined = {"import": imp, "by_scenario": {}, "by_widget_count": {}}
```

with the existing `by_widget_count` assignment retained unchanged, so the README's existing table keeps working. For a twin-less scenario, write to `combined["by_scenario"][scenario]["python_only"]` with the median per-frame ms and **no** `comparison` key.

- [ ] **Step 6: Dispatch a benchmark run and read the result**

```bash
git add bench/bench.py bench/src/main.rs .github/workflows/benchmark.yml
git commit -m "Measure the kwargs tax and the author's own Python per frame

label is the cheapest path through the binding, so the existing
benchmark cannot answer what an app author should write differently.
text_edit_plain and text_edit_hint measure the **kwargs path with and
without an option, each against a matching Rust twin; python_side
measures the author's own f-string and attribute work with no twin,
since its claim is about their code rather than the binding's."
git push fork HEAD:feature/egui-0.31-coverage
gh workflow run benchmark --repo ChetanKnowIT/pyegui \
  --ref feature/egui-0.31-coverage -f widgets='500' -f trials='1' \
  -f scenario='text_edit_plain'
gh run list --repo ChetanKnowIT/pyegui -L 1 \
  --json databaseId,name,status,conclusion
gh run watch <run-id> --repo ChetanKnowIT/pyegui --exit-status
```

Expected: success. Repeat for `text_edit_hint` and `python_side`. `bench/` is only compiled here, so this dispatch is the verification for every Rust change in this task.

If the run fails, read the step log rather than guessing: `gh run view <run-id> --repo ChetanKnowIT/pyegui --log-failed`.

---

### Task 5: Gate the performance page against the snapshot, warn-not-fail

The gate is the point of the whole exercise: a page whose numbers cannot be checked against data will rot the way the README's Performance section did.

**Files:**
- Create: `docs/performance.rst`
- Modify: `docs/index.rst` — add `performance` to the first toctree, after `guides`
- Create: `tests/check_performance_page.py`
- Modify: `tests/doc_claims.py` — call the new check from `main()`

**Interfaces:**
- Consumes: `bench/results/combined.json` (Task 3), its `by_scenario` and `python_only` keys (Task 4). The two `VALID_SCENARIOS` lists from Task 2.
- Produces: `check_performance_page(snapshot_path, page_path, warnings) -> None` in `tests/check_performance_page.py`, which appends human-readable strings to `warnings` and never raises for drift. `docs/performance.rst` defines one RST substitution per number, named `|key|`, read from the snapshot.

- [ ] **Step 1: Write the failing test**

Create `tests/test_check_performance_page.py`:

```python
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from check_performance_page import check_performance_page


SNAPSHOT = {
    "import": {"import_pyegui_ms": 7.277},
    "by_scenario": {
        "label": {
            "50": {
                "comparison": {
                    "ratio_median": 1.33,
                    "ratio_min": 1.31,
                    "ratio_max": 1.34,
                }
            }
        }
    },
}


def write(tmp_path, snapshot, page):
    sp = tmp_path / "combined.json"
    sp.write_text(json.dumps(snapshot))
    pp = tmp_path / "performance.rst"
    pp.write_text(page)
    return sp, pp


def test_missing_snapshot_warns_and_does_not_raise(tmp_path):
    page = tmp_path / "performance.rst"
    page.write_text(".. |ratio_label| replace:: 1.33x\n")
    warnings = []
    check_performance_page(tmp_path / "nope.json", page, warnings)
    assert warnings, "a missing snapshot must warn, not pass silently"


def test_matching_page_warns_about_nothing(tmp_path):
    page = (
        ".. |ratio_label| replace:: 1.33x\n"
        ".. |ratio_label_range| replace:: 1.31x-1.34x\n"
    )
    sp, pp = write(tmp_path, SNAPSHOT, page)
    warnings = []
    check_performance_page(sp, pp, warnings)
    assert not warnings, warnings


def test_drifted_page_warns_with_both_values(tmp_path):
    page = (
        ".. |ratio_label| replace:: 9.99x\n"
        ".. |ratio_label_range| replace:: 1.31x-1.34x\n"
    )
    sp, pp = write(tmp_path, SNAPSHOT, page)
    warnings = []
    check_performance_page(sp, pp, warnings)
    assert any("9.99x" in w and "1.33" in w for w in warnings), warnings


def test_snapshot_without_by_scenario_warns_and_does_not_raise(tmp_path):
    sp, pp = write(tmp_path, {"import": {}}, "nothing here\n")
    warnings = []
    check_performance_page(sp, pp, warnings)
    assert any("by_scenario" in w for w in warnings), warnings
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `python -m pytest tests/test_check_performance_page.py -v`

Expected: FAIL with `ModuleNotFoundError: No module named 'check_performance_page'`.

- [ ] **Step 3: Implement the check so it cannot raise on a missing snapshot**

Create `tests/check_performance_page.py`:

```python
"""Check docs/performance.rst against the benchmark snapshot.

Warn, never fail. Between-run variation on hosted runners is 1.30x-1.55x for
identical code, so a page number that differs from the last snapshot by a
rounding step is expected rather than defective. A hard gate here would go red
on ordinary benchmark runs, and a build that goes red for a reason nobody can
act on is a build people learn to ignore -- which would cost more than the
gate protects.

The README, CHANGELOG and TODO checks still fail the build. This is a second,
softer tier underneath them.
"""

import json
import re
from pathlib import Path

# scenario -> page substitution names. Ordered so the rendered page reads in
# the same order as the scenario table.
SUBSTITUTIONS = {
    "label": ("ratio_label", "ratio_label_range"),
    "text_edit_plain": ("ratio_text_edit_plain", "ratio_text_edit_plain_range"),
    "text_edit_hint": ("ratio_text_edit_hint", "ratio_text_edit_hint_range"),
}


def _substitutions(page_text):
    return dict(
        re.findall(r"^\.\. \|(\w+)\| replace:: (.+?)\s*$", page_text, re.M)
    )


def _fmt_ratio(value):
    return f"{value}x"


def _fmt_range(low, high):
    return f"{low}x-{high}x"


def check_performance_page(snapshot_path, page_path, warnings):
    """Append warnings about page/snapshot disagreement. Never raises."""
    snapshot_path = Path(snapshot_path)
    page_path = Path(page_path)

    if not page_path.exists():
        return

    page_text = page_path.read_text(encoding="utf-8")
    declared = _substitutions(page_text)

    if not snapshot_path.exists():
        warnings.append(
            f"docs/performance.rst: no benchmark snapshot at {snapshot_path}, "
            "so its numbers cannot be checked. Dispatch the `benchmark` "
            "workflow to produce one."
        )
        return

    try:
        snapshot = json.loads(snapshot_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        warnings.append(f"docs/performance.rst: snapshot is not valid JSON ({exc})")
        return

    by_scenario = snapshot.get("by_scenario")
    if by_scenario is None:
        warnings.append(
            "docs/performance.rst: the snapshot predates by_scenario, so the "
            "page cannot be checked against it. Re-run `benchmark`."
        )
        return

    for scenario, names in SUBSTITUTIONS.items():
        if scenario not in by_scenario:
            warnings.append(
                f"docs/performance.rst: the snapshot has no '{scenario}' "
                "scenario. Re-run `benchmark`."
            )
            continue
        for entry in names:
            if entry not in declared:
                warnings.append(
                    f"docs/performance.rst: |{entry}| is never defined, so the "
                    f"page has no number for '{scenario}'."
                )

    label = by_scenario.get("label")
    if isinstance(label, dict):
        for count, entry in label.items():
            comparison = (entry or {}).get("comparison")
            if not comparison:
                continue
            expected = {
                "ratio_label": _fmt_ratio(comparison["ratio_median"]),
                "ratio_label_range": _fmt_range(
                    comparison["ratio_min"], comparison["ratio_max"]
                ),
            }
            for entry_name, value in expected.items():
                actual = declared.get(entry_name)
                if actual is not None and actual != value:
                    warnings.append(
                        f"docs/performance.rst: |{entry_name}| says {actual}, "
                        f"the snapshot says {value} "
                        f"(scenario 'label', {count} widgets/frame)."
                    )
```

The `warnings` list is the whole output contract. Nothing here raises, and nothing appends to a `failures` list.

- [ ] **Step 4: Run the tests to verify they pass**

Run: `python -m pytest tests/test_check_performance_page.py -v`

Expected: PASS, 4 tests.

- [ ] **Step 5: Wire it into `doc_claims.py` and keep the exit code at 0**

In `tests/doc_claims.py`, add to the imports near the top:

```python
from check_performance_page import check_performance_page
```

`REPO_ROOT / "tests"` is already on `sys.path` (line 23), so the import resolves.

In `main()`, after the existing TODO claim checks and before the failure reporting, add:

```python
    # The performance page is a second, softer tier: warn, never fail. See
    # tests/check_performance_page.py for why failing here would be wrong.
    performance_warnings = []
    check_performance_page(
        REPO_ROOT / "bench" / "results" / "combined.json",
        REPO_ROOT / "docs" / "performance.rst",
        performance_warnings,
    )
```

Then, after the existing `for failure in failures:` loop and before the success `print`, add:

```python
    for warning in performance_warnings:
        print(f"WARN {warning}", file=sys.stderr)
        print(f"::warning::{warning}", file=sys.stderr)
    if performance_warnings:
        print(f"{len(performance_warnings)} performance page warning(s)")
```

Do **not** add these to `failures` and do **not** change the return value. `examples.yml` runs `doc_claims.py || doc_status=$?`, so returning 1 here would turn the whole `examples` job red over a documentation warning.

- [ ] **Step 6: Verify the gate end to end, including the exit code**

```bash
python tests/doc_claims.py; echo "exit=$?"
```

Expected: exit `0`, plus the existing `module exports 174 names` line. With no snapshot present yet, one WARN about the missing snapshot — which is the Review Focus item 1 case, verified rather than assumed.

- [ ] **Step 6b: Assert the two halves' scenario lists agree**

Task 2 duplicates the valid-scenario names in Python and Rust because a Rust binary cannot read the Python module. That duplication is only safe if something checks it, so add `tests/test_scenario_names_agree.py`:

```python
import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(REPO_ROOT / "tests"))

import bench_names  # noqa: E402


def test_rust_lists_the_same_scenarios_as_python():
    rust = (REPO_ROOT / "bench" / "src" / "main.rs").read_text(encoding="utf-8")
    # The names appear in the `scenario()` match arms.
    arms = set(re.findall(r'"(\w+)" => Ok', rust))
    # python_side is deliberately absent from the Rust match: it has no twin.
    assert arms == {"label", "text_edit_plain", "text_edit_hint"}
    assert arms | {"python_side"} == bench_names.VALID_SCENARIOS


def test_python_side_rejects_its_own_valid_name():
    # Asserted because it is the one asymmetry in the design: python_side is
    # valid on the Python side and rejected by Rust. If that ever changes, the
    # assertion above is what notices.
    assert "python_side" not in re.findall(
        r'"(\w+)" => Ok',
        (REPO_ROOT / "bench" / "src" / "main.rs").read_text(encoding="utf-8"),
    )
```

Extract the Python list into a small module the Rust-parsing test can import, by adding `tests/bench_names.py`:

```python
"""The benchmark scenario names, on the Python side.

Duplicated in `bench/src/main.rs`'s `scenario()` because a Rust binary cannot
read a Python module at runtime. `tests/test_scenario_names_agree.py` asserts
the two lists agree, since drift here is otherwise silent: both halves would
each reject a scenario the other accepts, and the failure would look like a
benchmark bug.
"""

VALID_SCENARIOS = ["label", "text_edit_plain", "text_edit_hint", "python_side"]
```

Then have `bench/bench.py` keep its own `VALID_SCENARIOS` as written in Task 2 — it cannot import from `tests/` at runtime, since `bench/bench.py` runs from the repo root against an installed wheel, not against the test tree.

Run: `python -m pytest tests/test_scenario_names_agree.py -v`

Expected: PASS, 2 tests.

- [ ] **Step 7: Write the page with substitutions, and add it to the toctree**

Create `docs/performance.rst` with the structure below. Every number is a substitution the gate checks; every prose claim is one the gate cannot check and a human therefore owns.

Sections: what the binding costs (linking to the README's table rather than restating it), what `**kwargs` costs, what your own Python costs, and what is not claimed (launch time, first frame, a blended score, and that `label` is the cheapest widget and may flatter the binding).

Define the substitutions the gate reads — `|ratio_label|`, `|ratio_label_range|`, `|ratio_text_edit_plain|`, `|ratio_text_edit_plain_range|`, `|ratio_text_edit_hint|`, `|ratio_text_edit_hint_range|` — plus `|import_ms|`, `|frame_ms_python_side|`, `|last_verified_run|` and `|last_verified_commit|`. The last two are what the page states in its "last verified" line, since the snapshot commit fires no workflow and nothing else will remind anyone to re-run.

In `docs/index.rst`, add `performance` to the first toctree after `guides`.

- [ ] **Step 8: Verify the page renders and the gate is clean against real data**

```bash
git fetch fork
git show fork/feature/egui-0.31-coverage:bench/results/combined.json | head -40
python tests/doc_claims.py; echo "exit=$?"
```

Expected: the snapshot exists with a `by_scenario` key, and the gate exits `0` with no WARN once the substitutions match. If it warns, the number in the page is wrong — fix the page.

- [ ] **Step 9: Commit and push**

```bash
git add tests/check_performance_page.py tests/test_check_performance_page.py \
  tests/doc_claims.py docs/performance.rst docs/index.rst
git commit -m "Gate docs/performance.rst against the benchmark snapshot

The page renders its numbers from bench/results/combined.json through RST
substitutions, and tests/check_performance_page.py warns when the page
and the snapshot disagree. It warns rather than fails: between-run
variation is 1.30x-1.55x for identical code, so a hard gate would go red
on ordinary runs. The README, CHANGELOG and TODO checks still fail the
build as before."
git push fork HEAD:feature/egui-0.31-coverage
```

- [ ] **Step 10: Confirm the gate runs in CI, and in the right workflow**

```bash
gh run list --repo ChetanKnowIT/pyegui --branch feature/egui-0.31-coverage -L 3 \
  --json databaseId,name,status,conclusion
gh run watch <examples-run-id> --repo ChetanKnowIT/pyegui --exit-status
```

Expected: both green. The warnings appear in the `examples` job log, since `examples.yml:184` is where `doc_claims.py` runs — not in `check`.
