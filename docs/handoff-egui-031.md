# Handoff — egui 0.31.1 coverage

## Current state

Branch `feature/egui-0.31-coverage`, 13 commits ahead of `main`. `main` is
deliberately untouched at `1e16c16` and must stay that way until the 0.31.1
work is done; see "Branch policy" below.

Latest runs, all green: `check` 37104300409, `examples` 37104300424,
`benchmark` 37101858384.

**174 exported names** — 18 classes, 156 functions (110 plain + 46
`*_response`). Clippy is **not** at 0 diagnostics: 29 lints and 2 errors,
hidden by two `check.yml` defects — see the Clippy section below.

TODO totals: **33 open, 7 partial, 41 done.** §3 (containers) is closed.

| Section | open | partial | done |
| --- | ---: | ---: | ---: |
| §1 Response: interaction state | 3 | 0 | 7 |
| §2 Missing widgets | 1 | 4 | 5 |
| §3 Missing containers | 0 | 0 | 12 |
| §4 Layout, sizing and geometry | 5 | 0 | 5 |
| §5 Context API | 9 | 2 | 0 |
| §6 Builder options on existing widgets | 7 | 0 | 0 |
| §7 egui_extras | 4 | 1 | 0 |
| §7 Benchmarks | 4 | 0 | 12 |

Shipped since the previous handoff:

| Commit | What |
| --- | --- |
| `d74acd3` | Containers completed: `frame` + 8 presets, `scroll_area_both`, builder options on all scroll areas, `collapsing_response`, `menu_button` / `menu_image_button` / `menu_image_text_button`. 130 → 144 names. |
| `612afce` | Three compile errors fixed (see "0.31.1 facts"). |
| `d653011` | Recorded the 14 new names; the doc-claims gate caught the count. |
| `919e1dd` | Corrected five wrong numbers in the docs and gated the two that rot. |
| `11f4119` | `scene` + the `Rect` class. Last container. 144 → 146. |
| `851cdc5` | Scene screenshot into the README and gallery. |
| `9098a27` | Layout, sizing, measurement, `push_id`, `columns`. 146 → 174. |
| `2a67f6f` | README renamed to Markdown. |
| `95cea3c` | Handoff doc and TODO §3 brought up to date. |
| `b0918e7` | Benchmark measures 5 trials per count and reports median + spread. |
| `README` rewrite | Performance section now quotes the 5-trial table: median ratios 1.33x/1.33x/1.32x, per-trial ranges, the within-run spread vs the between-run spread, and the note that the microseconds are runner-specific. |

## Constraints

**There is no local Rust toolchain and there must not be one.** No `rustup`,
no `cargo`, no `docker`, no `maturin` in the venv. A rustup install was
started at one point and then deleted (`~/.cargo`, `~/.rustup` removed).
Do not reinstall it. GitHub Actions is the only place this crate compiles, so
every build claim must cite a workflow run. Full writeup in
`docs/development.rst`.

**`origin` is read-only.** The account has `READ` on `snappercayt/pyegui`, so
all work goes to the fork:

```
origin  https://github.com/snappercayt/pyegui.git      (read-only)
fork    https://github.com/ChetanKnowIT/pyegui.git     (push target, runs Actions)
```

**`regen-lockfile.yml` is `workflow_dispatch` only.** It uploads a
`Cargo.lock` artifact after `cargo update`, which is how lockfile changes get
made without a local cargo. It used to fire on every push, which spent a
runner per commit to produce a lockfile nobody read: the resolution has not
moved since the egui `=0.31.1` pin. Delete it once dependency work settles --
TODO §7 plans to enable `egui_extras`' `syntect` feature, which needs it once.

**Clippy is advisory, not a gate — and it is NOT currently clean.** An
earlier revision of this file claimed a baseline of **0 diagnostics**
"re-verified against run 37104300409". That was wrong, and the way it was
wrong is worth knowing.

Measured from the raw run logs on 2026-10-04 (`gh api
repos/ChetanKnowIT/pyegui/actions/runs/<id>/logs`, because `gh run view
--log` returns nothing for some runs), runs 37216647892 and 37222603927
each report **29 clippy lints across 14 distinct lints, plus 2 hard
errors**:

```
error: mutable borrow from immutable input(s)
  --> src/lib.rs:1237   unsafe fn ui_stack(ui: &*mut Vec<*mut egui::Ui>) -> PyResult<&mut Vec<...>>
  --> src/lib.rs:1252   unsafe fn current_ui(...)  (same shape)
error: could not compile `pyegui` (lib) due to 2 previous errors; 29 warnings emitted
```

`clippy::mut_from_ref` is **deny-by-default**, so clippy aborts before
linting finishes. Both functions date to `2691c95` (the initial commit),
so they are pre-existing, not introduced by the 0.31.1 work.

**Two independent defects in `check.yml` hide this**, and both are worth
fixing in their own change:

1. The clippy step is `cargo clippy --locked --all-targets 2>&1 | tee
   clippy.log` with no `pipefail`, so clippy failing to compile exits 0
   and the job concludes SUCCESS.
2. The "Report clippy delta" step counts with `grep -cE
   '^(warning|error)(\[|:)'`, anchored at the start of the line. Every
   clippy diagnostic is indented, so the count is always **0** and the
   summary prints "clippy diagnostic lines: 0" regardless of the truth.

So the "no new lints" property holds only by accident: Task 3's clippy
step is byte-identical to Task 2's apart from line-number shifts, which
was verified by diffing the two step blocks with integers normalised. The
gate is not measuring what it claims to measure.

**Do not promote clippy to `-- -D warnings` yet.** The handoff previously
recommended exactly that on the strength of the false zero. The real
prerequisite is fixing the two defects above, then working the 29 down —
starting with the two `mut_from_ref` errors, which are the only ones
clippy treats as fatal.

**Counting diagnostics:** do it from the raw log zip, strip ANSI codes
first (`CARGO_TERM_COLOR: always` is set), and count distinct lint
help-URLs (`index.html#<lint>`). Counting lines overcounts, because
clippy repeats warnings across `lib` and `lib test` targets.

## Branch policy

`fork/main` is frozen at `1e16c16` and `feature/egui-0.31-coverage` carries
all 0.31.1 work. The merge base is `1e16c16`, which is `main`'s tip — so the
feature branch is a straight-line descendant and the eventual merge **is** a
fast-forward, with no two-line history to reconcile.

The 7 commits the two branches shared earlier were cherry-picked rather than
shared objects; that is why `git log main..feature` once showed duplicates.
The local branch `main-sync` is stale at `39b7515` and does not track
`fork/main` — check `fork/main`, not the local name, before making claims about
branch state.

When the 0.31.1 TODOs are done, merge to `main` then.

Push to `fork` only, and prefer a single push per change so `check`,
`examples`, `screenshot` and `benchmark` all read the same commit.

## 0.31.1 API facts that cost a CI run each

**`tests/expected_exports.py` is the export gate, not `check.yml`.** It was a
120-line Python set literal inside a YAML block scalar, which I broke three
times editing it — twice from a doubled comma, once by removing the commas
entirely, which Python reads as implicit string concatenation and which
therefore passed every naive "does it parse" check while asserting nothing.
As a real file it is lintable, diffable and executable. Do not move it back.

It asserts in both directions: `check()` fails on a declared name the module
does not export, `check_no_unexpected()` fails on an exported name the file
does not declare. The second exists so a widget cannot be deleted from both
places at once and leave CI green. It subtracts an explicit
`_INTERPRETER_NAMES` set — notably `pyegui` itself, which pyo3 binds on the
module object, so `dir(pyegui)` contains it.

- `Ui::add` returns `Response`, not `InnerResponse<Response>`. There is no
  `.inner` to take. (`Ui::horizontal` and `Ui::scope` *do* return
  `InnerResponse`, so both spellings are correct in the same file.)
- `Hsva` is at `egui::ecolor::Hsva`, not `egui::Hsva`. egui re-exports
  `Color32` and `Rgba` at the root but not `Hsva`.
- `Color32` is a newtype over a private `[u8; 4]`; build and read it with
  `from_rgba_unmultiplied` / `to_srgba_unmultiplied`. The `[u8; N]`-taking
  pickers take plain arrays instead, so only `_srgba` needs the dance.
- `drag_released` / `drag_released_by` are deprecated in 0.31.1 in favour of
  `drag_stopped` / `drag_stopped_by`.
- `total_drag_delta` does not exist in 0.31.1 at all. An earlier revision of
  TODO.md invented it.
- **Tooltips and context menus need their own callback wrapper:** egui types
  them `impl FnOnce(&mut Ui)`, returning `()`, but `run_nested_update_func`
  returns `PyResult<()>`. Those call sites use `run_nested_update_func_lossy`,
  which drops the error. The Python exception has already been displayed.
- `on_hover_text` / `on_hover_ui` consume the `Response` in egui. They take
  `&mut self` here and reassign from `self.inner.clone()...`, because
  `egui::Response` has no `Default` and cannot be moved out of a `&mut`.
- **`Popup` and `MenuBar` do not exist in egui 0.31.1.** `containers/popup.rs`
  at that tag defines only free functions (`show_tooltip_text`,
  `was_tooltip_open_last_frame`, …) and `lib.rs` exports no `Popup` type.
  `MenuBar` appears in none of `lib.rs`, `ui.rs` or `menu.rs`. Both are
  recorded in TODO §3 as dropped rather than shipped, with the evidence — the
  same treatment `total_drag_delta` got. Do not re-add them without moving the
  pin.
- **`ScrollBarVisibility` is not reachable as `egui::ScrollBarVisibility`.**
  It is declared `pub` in `containers::scroll_area`, but `containers/mod.rs`
  only re-exports `ScrollArea`, so it never reaches the crate root. It needs the
  full `egui::containers::scroll_area::ScrollBarVisibility`. "pub in a pub
  module" is not "reachable from the root".
- **`Color32` cannot be `extract`ed.** It is a plain `#[pyclass]` with no
  `extract` impl, hence no `FromPyObjectBound`. Use `downcast` + `borrow`.
  Every other pyclass here is passed by reference and mutated, so there is no
  existing pattern to copy.
- **A `&mut T` does not reborrow to `&T` for trait resolution.** Reborrow
  coercion applies to function arguments, not to choosing a `From` impl, so
  `egui::Rect::from(&mut rect)` fails against an impl for `&Rect`. Name the
  reborrow: `egui::Rect::from(&*rect)`.
- **Edition 2021, so an `unsafe fn` body is an implicit unsafe block.** The
  `apply_*_options` helpers are `unsafe fn` because every `opt_*` helper is;
  they need no inner `unsafe {}` and have none.
- `CollapsingResponse.openness` is an `f32` (1.0 open, 0.0 closed, between
  while animating), not an `Option<bool>`. Folding it into a `Bool` means
  comparing at 0.5, not storing the value.
- `Scene::show` takes `(parent_ui, &mut Rect, FnOnce(&mut Ui))`. The callback
  gets only a `Ui`, like every other binding. An earlier claim in this file
  that it hands over a `TSTransform` was wrong — `TSTransform` appears only in
  the private `show_global_transform`. The real subtlety is the `Rect`, which
  egui reads and writes, so it must be the same Python object every frame.

## Documentation gates

**`tests/doc_claims.py` derives the numbers from the code, not the prose.**
It checks the README's name count, every name the CHANGELOG presents as
shipped, and — added at `919e1dd` — the `Response` method count and the
`*_response` variant count, which is what let three stale counts sit unnoticed
while the name total was kept correct.

Verify it locally before pushing; it needs no compiled extension:

```bash
python tests/doc_claims.py
```

It caught real drift on nearly every batch. When it fails on a count, the
number in the prose is wrong and the code is right — fix the prose.

**The README is Markdown and must stay that way.** GitHub wraps a `.rst`
README in `<div class="plain"><pre>` and processes no markup at all — not RST,
not Markdown, not raw HTML. Verified with
`gh api -H 'Accept: application/vnd.github.html'`, which returned zero `<img>`
tags for the `.rst` version. `tools/rst_to_md.py` did the conversion and
counts every construct before and after, failing on any surviving RST; it
caught six real bugs during that run, so if the README is ever restructured,
run it rather than hand-editing.

`docs/gallery.rst` keeps the same screenshots as RST `.. figure::` directives,
which is what Sphinx resolves natively. Sphinx reads the README itself through
`myst-parser`.

## Verification loop

```bash
git push fork HEAD:feature/egui-0.31-coverage
gh run list --repo ChetanKnowIT/pyegui --branch feature/egui-0.31-coverage \
  --json databaseId,name,status,conclusion -q '.[] | "\(.databaseId) \(.name) \(.status)"'
gh run watch <check-run-id> --repo ChetanKnowIT/pyegui --exit-status
```

`check` and `examples` fire on every push. `screenshot` fires only when
`examples/gallery.py`, `src/lib.rs` or its own workflow file changes.
`benchmark` is `workflow_dispatch` only, so it must be dispatched by hand with
`--ref feature/egui-0.31-coverage`; without `--ref` it runs against `main`,
which no longer has the widget-count work.

```bash
gh workflow run benchmark --repo ChetanKnowIT/pyegui \
  --ref feature/egui-0.31-coverage -f widgets='50,500,2000'
```

`check.yml` runs: `cargo check --locked --all-targets` → advisory clippy →
assert `cargo tree -p {egui,eframe,egui_extras}` all resolve to exactly
`0.31.1` → `maturin build` + `pip install` → the export gate.

**Every new public name must be added to `tests/expected_exports.py` in the
same change.** Otherwise the gate fails, which is the point. Verify the two
lists agree before pushing:

```bash
python - <<'PY'
import re, ast, pathlib
lib = pathlib.Path("src/lib.rs").read_text()
i = lib.index("#[pymodule]")
reg = set(re.findall(r"wrap_pyfunction!\((?:crate::)?(\w+), m\)", lib[i:]))
cls = set(re.findall(r"m\.add_class::<(\w+)>", lib[i:]))
mod = ast.parse(pathlib.Path("tests/expected_exports.py").read_text())
gated = set()
for node in mod.body:
    if isinstance(node, ast.Assign) and getattr(node.targets[0], "id", "") in (
        "CLASSES", "FUNCTIONS", "RESPONSE_VARIANTS"):
        gated |= set(ast.literal_eval(node.value))
print("match:", reg | cls == gated, sorted((reg | cls) ^ gated))
PY
```

**`bench/` is a separate Cargo workspace** (`bench/Cargo.toml` declares its own
`[workspace]`), so `check.yml`'s `cargo check` does not compile it. The only
verification for Rust changes under `bench/` is a `benchmark` run.

Roughly 3 minutes for `check`, 5–6 for `benchmark`. `make build`,
`make build-manylinux`, `make develop` and `make doc` all cannot run locally.

## The benchmark: solved, and what it says

Measured over 5 trials at 3 widget counts, run 37106337662:

| widgets/frame | pyegui us | egui us | ratio median | min-max | spread |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 50 | 0.600 | 0.453 | 1.33x | 1.31x-1.34x | 1.7% |
| 500 | 0.487 | 0.365 | 1.33x | 1.33x-1.34x | 0.8% |
| 2000 | 0.484 | 0.365 | 1.32x | 1.31x-1.34x | 2.3% |

`import pyegui` 7.277 ms. Reproduce with:

```bash
gh workflow run benchmark --repo ChetanKnowIT/pyegui \
  --ref feature/egui-0.31-coverage -f widgets='50,500,2000' -f trials='5'
```

**Two findings.**

The ratio is flat -- 1.33x, 1.33x, 1.32x across a 40x range of widget counts.
A fixed per-call toll was expected to amortise as frames grow, so the ratio
should have fallen. It does not, so the binding is not a per-call tax that
widget density dilutes.

Within a run the spread is under 2.5%. The 1.30x / 1.52x / 1.38x-1.55x range
across three earlier runs was therefore **not** within-run noise -- it was
variation between hosted machines. That is the better outcome: the ratio is a
real, stable property of the binding, while the absolute microseconds are
specific to a runner and do not transfer.

Two design decisions in the workflow are what made this visible, and both are
easy to undo by accident:

- **Trials are the outer loop, widget counts the inner.** Counts-outer measures
  50 widgets on a cool machine and 2000 on a warm one, and that gradient looks
  exactly like a real effect of widget count. Interleaving is the only reason
  the flatness can be trusted.
- **The median with its min-max, not a mean.** A shared runner has outliers;
  one slow trial drags a mean away from the typical case.

Files are `pyegui-<count>-t<trial>.json`, and the combine step re-reads the
count from the payload to check it against the filename.

## Settled decisions

These were open at `e0beb36` and decided on 2026-10-02. Do not re-open them
without new information.

1. **Nesting: additive `_response` suffix.** New `button_response()` etc. sit
   alongside `button_clicked()`; nothing existing changes signature. Chosen
   over returning a falsy `Response` from the bool helpers, which would break
   `if button_clicked():` for current users.
2. **Drag-and-drop payloads are deferred.** Ship the `Response` class and the
   `*_response` variants without DnD. `dnd_set_drag_payload` takes
   `Arc<dyn Any + Send + Sync>`, which does not map onto Python objects; that
   bridging is its own design problem. TODO §1 keeps the DnD items unchecked.
3. **Batch order, as executed.** D minus `colored_label` shipped first (five
   trivial functions, no new classes) to calibrate the loop; then A+B as one
   change, since A alone verifies little; then C.
4. **`columns` takes a list of callables**, one per column, not a column
   object. egui's callback receives `&mut [Ui]`; a Python callable cannot
   receive a slice of `Ui`s, and a `Column` class would be a second thing to
   learn for no gain.
5. **Geometry options take tuples, not a `Vec2` class.** `(400.0, 300.0)` for
   sizes and positions; a single number or 2-/4-sequence for margins and
   corner radii. Colours take a `Color32` *or* an `(r, g, b, a)` tuple.
   `Rect` is a class because egui's is two corners rather than a position and a
   size, which does not fit that shape.

## Next up, in order

1. **Builder options (§6)** — the largest single block: `Slider` 23 options,
   `TextEdit` 18, `DragValue` 13, `DatePickerButton` 12, `run_native` viewport
   kwargs 20, `NativeOptions` 11, plus `App::save` / `Storage`.
2. **Context API (§5)** — 9 open, largely greenfield. `Context` reaches 10 of
   egui's 148 public methods.
4. **egui_extras (§7)** — `Table` / `TableBuilder`, `StripBuilder`, `Sizing`,
   and `code_view_ui`, which needs the `syntect` feature and therefore the
   lockfile workflow.
4. **`colored_label`** (§2) — needs a `RichText` wrapper.
5. **§4 remainder** — `with_layout` / `wrap_mode` (needs a whole
   `egui::Layout`, which the existing `Layout` / `LayoutType` classes do not
   cover), `UiBuilder` / `scope_builder` / `new_child`, `interact`, and
   `painter` access for custom drawing.
6. **Benchmark, one heavier widget** — `label` is the cheapest path through the
   binding and may flatter it. Lower priority now that the ratio is known to be
   stable, since the question it answers is whether the cost scales with widget
   complexity rather than count.

## House rules for this repo

- Do not add local build steps. If it cannot run in CI, it does not run.
- Do not trust build output that did not come from a workflow run.
- Keep the lockfile authoritative; CI builds with `--locked`.
- Commit only once `check` is green.
- Update `TODO.md` and the README in the same change as the code. The
  `doc_claims.py` gate will fail the build if the numbers disagree.
- Prefer one push per change, so every workflow reads the same commit.
- Counting is not optional. Several silent corruptions in this file's history
  — a duplicated container block, a deleted function, a doc comment truncated
  by a bad edit — were invisible until a per-name count was run.
