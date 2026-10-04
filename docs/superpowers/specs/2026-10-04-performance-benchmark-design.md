# Performance guide and second benchmark scenario — design

**Date:** 2026-10-04
**Status:** awaiting review
**Scope:** benchmark protocol (the numbers `README.md` and `TODO.md` §7 quote)
and a new `docs/performance.rst`. Not a binding-API change.

## Problem

The benchmark answers exactly one question: what does a `label` call cost
through the binding relative to the same call in Rust. It answers that well
— median 1.33x, flat across a 40x range of widget counts, all 15 trials
between 1.31x and 1.34x (run 37106337662).

An app author reading that has one follow-up question the benchmark cannot
answer: **what should I write differently?** Every number it produces is for
`label`, which is the cheapest path through the binding — it takes a single
`&str`, allocates nothing, and touches no builder. A guide built on it alone
would be advice extrapolated from the least representative widget in the crate.

Two costs an author can actually control are currently unmeasured:

1. **The `**kwargs` tax.** Seven signatures in `src/lib.rs` take `**kwargs`
   today (`run_native`, `text_edit_singleline`/`_response`,
   `text_edit_multiline`/`_response`, `image`/`_response`). pyo3 parses a
   keyword dict into typed arguments on every such call. TODO §6 is about to
   put kwargs on `Slider` (23 options), `TextEdit` (18), `DragValue` (13),
   `DatePickerButton` (12) and ~100 widgets besides, so the tax goes from
   marginal to universal. Measuring it *before* that lands is the point.
2. **The author's own Python.** `update_func` bodies do f-strings, read
   `Str.value`, do arithmetic. That time is in the frame but is not the
   binding's, and no current measurement separates it.

There is also a structural problem with gating either claim. Benchmark results
exist only as workflow artifacts — `bench/results/` is untracked — so
`tests/doc_claims.py`, which derives every other documented number from code,
has nothing to compare a performance page against.

And `.github/workflows/benchmark.yml` has a **duplicated block**: lines
144–163 repeat the `import pyegui` cold-import measurement, the
`WIDGET_COUNTS` validation and the `cargo build --release` that lines
110–141 already performed, then lines 166–203 repeat the validation and the
build *again*. Both copies run; both are idempotent; the runs are green. It
is the "duplicated container block" corruption class `docs/handoff-egui-031.md`
warns about, arrived at by editing rather than by intent.

## Decisions taken

**Where the doc lives.** `docs/performance.rst`, a new page in the existing
Sphinx toctree beside `guides.rst`. The README stays the landing page and
stays Markdown; this is reference material, and `docs/index.rst` already
includes the README through `myst-parser`.

**How numbers reach the page.** The benchmark workflow commits
`bench/results/combined.json` back to the branch. `docs/performance.rst`
renders its numeric table from that committed snapshot by substitution, so
page and snapshot cannot disagree by construction.

**How drift is reported.** Warning, not failure. `tests/doc_claims.py`
grows a `check_performance_page()` that compares the page's numbers against
the snapshot and emits `::warning::` annotations plus a printed diff line.
It does **not** append to `failures`, so `check` stays green on drift while
the drift is visible in the run summary.

This is chosen over failing deliberately. Between-run variation on hosted
runners is 1.30x–1.55x for identical code, so a page number that differs from
the last snapshot by a rounding step is *expected*, not a defect. A hard gate
would go red on ordinary benchmark runs, and a build that goes red for a
reason nobody can act on is a build people learn to ignore — which would cost
more than the gate protects. A warning is the honest failure mode: drift is
surfaced, and nothing is masked.

The existing README/CHANGELOG/TODO checks keep failing the build exactly as
they do now. Nothing is weakened; this only adds a second, softer tier.

**Scenarios.** (a) kwargs tax against matching Rust twins, and (c) Python-side
per-frame share. Not (b), container/callback nesting: it needs the same Rust
machinery as (a) and is better measured once (a) exists.

## Design

### Scenarios

Each runs inside the existing trial loop, so rows remain comparable and the
README's committed `label` baseline is unaffected.

| Scenario | Python side | Rust twin | Question |
| --- | --- | --- | --- |
| `label` (existing) | `label(f"row {i}")` | `ui.label(..)` | baseline, unchanged |
| `text_edit_plain` | `text_edit_singleline_response(s)` | `ui.text_edit_singleline(..)` | kwargs path, zero options |
| `text_edit_hint` | `text_edit_singleline_response(s, hint_text="name")` | `.. .hint_text("name")` | kwargs path, one option |
| `python_side` | f-string + `Str.value` + arithmetic, fixed small widget count | none | author's own cost |

`SCENARIO` is an env var read by both `bench/bench.py` and
`bench/src/main.rs`, so the two halves always measure the same thing — the
same discipline as the existing shared `WIDGETS`.

**Why ratio-against-ratio.** `TextEdit` is far heavier than `label`, so its
absolute microseconds say nothing about the binding. The honest comparison is
whether `text_edit_plain`'s *ratio* differs from `label`'s: if both sit at
~1.33x the kwargs path costs nothing measurable per call, and if `TextEdit`
comes in higher, kwargs are the reason. The page states whichever the data
shows; it is not written in advance.

**Why `python_side` has no Rust twin.** Its claim is "this is your code, not
the binding's", not "the binding costs Nx here". A Rust baseline would be
meaningless for it, and saying so in the page is part of the point.

### Snapshot and page rendering

`bench/results/combined.json` gains a `by_scenario` key alongside the existing
`by_widget_count`, and is committed at the end of a successful run.

The commit uses the workflow's own `GITHUB_TOKEN`, which needs no PAT: the
workflow runs in the fork, so the token is scoped to the repository it pushes
to. The only obstacle was the workflow's own `permissions: contents: read` at
line 31, which is changed to `contents: write`.

A `GITHUB_TOKEN` commit does not trigger further workflow runs — GitHub's
recursion guard, so a bot cannot loop. The snapshot commit therefore does not
fire `check`, which is convenient here: the gate's job is to catch a *human*
editing the page's numbers without dispatching a run, which is exactly the
drift it warns about.

The page renders its table by substitution rather than by transcription:

```rst
.. |ratio_label| replace:: 1.33x
.. |ratio_label_range| replace:: 1.31x-1.34x
.. |extra_label| replace:: 0.151
```

Every number in the page is one of these substitutions. That is what makes the
gate checkable at all — `check_performance_page()` reads the same tokens the
page renders, so it is comparing the page against the snapshot rather than
against prose someone typed. Prose claims ("the ratio is flat across a 40x
range of widget counts") stay human-written and stay ungated; they are the
claims a machine cannot check.

### Gate

`tests/check_performance_page.py`, called from `tests/doc_claims.py` so the
one command still reports everything:

- Missing snapshot → warning, not failure. A fresh clone has no artifact and
  must not break `check`.
- Number in the page that the snapshot does not support → warning naming the
  scenario and both values.
- Substitution tokens present but unrendered → warning. A page that forgot to
  define one is a page that never got its number.

The workflow additionally prints a one-line "page is N commits behind the
snapshot" note to the step summary.

### Bug fix folded in

Remove the duplicated validation/import/build block in `benchmark.yml`
lines 144–203, keeping the first copy. This ships in the first commit,
before any scenario changes, so the CI plumbing is de-risked independently of
the measurement work.

## Sequencing

Three commits, one push each, so every workflow reads the same commit.

1. **Plumbing.** De-duplicate `benchmark.yml`; change `contents: read` to
   `contents: write`; add the snapshot commit; add `docs/performance.rst` to
   the toctree with the substitution machinery and the gate reading it. No new
   scenarios — the page renders an empty table and the gate warns on the human
   push that lands it.
2. **Scenarios.** `bench/src/main.rs` and `bench/bench.py` gain the
   `text_edit_*` and `python_side` scenarios with Rust twins. Dispatch
   `benchmark` with `--ref feature/egui-0.31-coverage`.
3. **Populate.** Fill the page from the resulting snapshot.

Commit 3 cannot be written until commit 2's run finishes, and its content
depends on what the data shows. The page's prose about the kwargs tax is
therefore written after the measurement, not before — which is the point of
gating on a snapshot rather than asserting a number in advance.

## Verification

CI-only, per the house rules: no local Rust toolchain, every build claim cites
a workflow run.

- `python tests/doc_claims.py` — the gate, runs locally, needs no extension.
- `check` must be green before each commit.
- `benchmark` dispatched by hand with `--ref feature/egui-0.31-coverage`;
  `check` and `examples` fire on every push.

The combine step's existing mismatch guard is extended to check `scenario`
alongside `widgets_per_frame`. That guard is what caught the two benchmark
halves drifting apart earlier (one repainted, one did not), and a scenario
mismatch is the same failure with a wider blast radius.

`python_side` is the exception and needs stating explicitly, or the guard will
either crash on a missing Rust file or special-case its way past the check it
exists to perform. Each scenario declares `has_rust_twin` in the payload. When
it is false the combine step records the Python figure under a separate
`python_only` key and computes **no** ratio — a ratio against a Rust baseline
that does not exist is exactly the meaningless number the parity guard was
built to prevent. `python_side` is reported as an absolute per-frame cost
against the `label` baseline's frame time, which is a comparison of two Python
figures and needs no twin.

## Risks

**The bot needs write permission in its own workflow.** The token is
`GITHUB_TOKEN` and no PAT is involved — the workflow runs in the fork, so the
token can push there. The change required is the workflow's
`permissions: contents: read` becoming `contents: write`. Verified against the
file rather than assumed.

**The bot's commit does not fire `check`.** GitHub does not trigger workflow
runs for commits made with `GITHUB_TOKEN`, so there is no recursion and no
self-inflicted red build. The gate therefore only ever fires on human pushes.

**§6 changes what the kwargs numbers mean.** Once `Slider` and `TextEdit`
carry 20+ options, the per-call tax grows. The page states the measurement's
date and the option count it was taken at, and the scenario's value is
precisely that it gives a before-number to compare against.

**`label` may flatter the binding.** Unresolved and stated in the page under
"what is not claimed". The `text_edit` scenarios are the partial answer; a
full one needs image, drag and layout, which is TODO's remaining benchmark
item.

## Out of scope

- No binding API change. No new widget options — that is §6.
- No `bench/` scenario for container nesting (option b).
- No `docs/performance.rst` claim about first-frame cost, launch time or
  renderer selection. Those measure the window system.
- No change to `README.md`'s existing Performance section, which continues to
  quote the `label` baseline.
