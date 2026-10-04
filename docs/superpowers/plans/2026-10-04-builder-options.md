# Builder options on existing widgets (§6) — implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give `Slider`, `DragValue`, `TextEdit`, `DatePickerButton`, `run_native`'s viewport and `NativeOptions` their egui builder options — named parameters for the common ones, `**options` for the tail — and reject any option name a widget does not implement.

**Architecture:** The `opt_*` helper convention already exists (ten of them) and already backs every container's options. This plan adds a `used: &mut OptNames` tracker parameter to each helper so the set of consumed names is known without mutating the caller's dict, then a `reject_unknown_options()` that errors on leftovers. Six target groups get options; the existing ~19 option-consuming functions are deliberately left alone.

**Tech Stack:** Rust (eframe/egui `=0.31.1`, pyo3 0.24.1), Python 3.11, GitHub Actions.

**Spec:** `docs/superpowers/specs/2026-10-04-builder-options-design.md` — read it first; it carries the settled decisions and the rejected `remove_item` approach, which must not be reintroduced.

## Global Constraints

- **No local Rust toolchain.** No `rustup`, `cargo`, `maturin` or `docker`; none may be installed. Every Rust claim must cite a workflow run. Do not add a local build step.
- **`bench/` is a separate Cargo workspace**; `check.yml` never compiles it. A `benchmark` dispatch is the only verification there.
- **`origin` is read-only.** Push only to `fork` (`ChetanKnowIT/pyegui`).
- **`check` and `examples` must both be green before every commit.** `examples` matters as much as `check` here: it is where `tests/doc_claims.py` runs and where every app and README snippet is executed.
- **`bench/**` is now in `examples.yml`'s path filters** (commit `c2cdae2`), so a `bench/` change runs pytest too.
- **No new exported names.** §6 changes signatures, not the export list. `tests/expected_exports.py` must be verified unchanged, not assumed unchanged.
- **Never mutate the caller's options dict.** `get_item` only. The `remove_item` approach was rejected; see the spec.
- **All 14 currently-accepted option names keep working, spelled exactly as today.** `text_edit_*`: `hint_text`, `editable`. `image`: `max_width`, `max_height`. `run_native`: `inner_width`, `inner_height`, `min_inner_width`, `min_inner_height`, `max_inner_width`, `max_inner_height`, `fullscreen`, `maximized`, `resizable`, `transparent`, `icon_path`.
- **egui's option names are the names.** `step_by` not `step`, `fixed_decimals` not `decimals`, `drag_value_speed` not `speed`. A rename is a silent breaking change.
- **Positional order is frozen.** `slider_float(value, min, max, text)` and `drag_float(value, min, max, speed)` are existing usage; new parameters go after them.
- **The option inventory is pinned to egui 0.31.1.** Every option list below must be checked against that tag's sources during implementation, not trusted because it appears in the spec.
- **`custom_formatter` / `custom_parser` are Rust closures and are NOT implemented.** No stubs. `NativeOptions`' `glow_options` / `wgpu_options` are deferred. Both are recorded in `TODO.md`.

## Review Focus

Failure modes a person would hit that no signature-level test exercises. Most likely first.

1. **A delegated option path rejects a valid option.** The check fires on "keys present but not recorded", so a helper that reads options without threading `used` makes a *working* option look unknown. Existing containers are the likely victims — `window`, `collapsing_response` and `frame` read options in their own functions. Pinned by Task 1 Step 6.
2. **A named parameter silently shadows a dict key.** `slider_float(v, 0, 1, suffix="ms", **{"suffix": "s"})` must raise, not pick one. §6 introduces the first signature where a name can arrive twice. Pinned by Task 2 Step 7.
3. **An option is accepted and then dropped.** Passing `inner_width=800` and getting a 640px window. A resolution-only test passes here; the behavioural test must not. Pinned by Task 5 Step 4.
4. **The caller's dict is mutated.** A caller reusing one dict across two widgets sees the second silently lose every option. This is the exact regression the spec rejected; pinned by Task 1 Step 5.
5. **A `Slider` positional argument shifts.** `slider_float(v, 0, 50, "Gain")` is existing usage. pyo3 keeps positional order, but nothing else guarantees it. Pinned by Task 2 Step 6.
6. **An enum-valued option accepts a wrong-case string** (`"verts"`) or rejects a valid one (`"circle"`). Hand-written variant lists get this wrong; egui 0.31.1's `Slider` handle shapes include both `Verts` and `Circle`. Pinned by Task 3 Step 5.

---

### Task 1: The consumed-key tracker and `reject_unknown_options`

Everything else depends on this. It touches code that already works, so it ships and is verified alone.

**Files:**
- Modify: `src/lib.rs` — the ten `opt_*` helpers, plus one new type alias and one new function

**Interfaces:**
- Consumes: nothing.
- Produces: `type OptNames = std::collections::HashSet<String>`, and every `opt_*` helper gains a third parameter `used: &mut OptNames`. Also `unsafe fn reject_unknown_options(opts: &Bound<'_, PyDict>, used: &OptNames, widget: &str) -> PyResult<()>`.

- [ ] **Step 0: Confirm the plan's verified inventory still holds**

These were read out of `src/lib.rs` when this plan was written:

- ten `opt_*` helpers: `opt_bool`, `opt_color32`, `opt_corner_radius`, `opt_f32`,
  `opt_margin`, `opt_order`, `opt_range`, `opt_scroll_bar_visibility`,
  `opt_stroke`, `opt_vec2`
- 25 occurrences of `Option<&Bound<'_, PyDict>>`
- `opt_string`, `opt_i32`, `opt_usize` do **not** exist yet
- `date_picker_button` and `date_picker_button_response` both exist
- `run_native`'s recognised kwargs: `fullscreen`, `icon_path`, `inner_height`,
  `inner_width`, `max_*_inner_*` (height and width), `maximized`,
  `min_*_inner_*`, `resizable`, `transparent` — plus two unrelated literals
  (`codeish`, `hello`) that are debug text, not option names

If any of these has changed, stop and re-read: the counts are what make the
remaining steps mechanical.

- [ ] **Step 1: Record the current helper inventory**

```bash
grep -n 'unsafe fn opt_' src/lib.rs
grep -c 'Option<&Bound' src/lib.rs
```

Expected: exactly 10 `opt_*` lines (`opt_bool`, `opt_color32`, `opt_corner_radius`, `opt_f32`, `opt_margin`, `opt_order`, `opt_range`, `opt_scroll_bar_visibility`, `opt_stroke`, `opt_vec2`) and 25 `Option<&Bound` occurrences. Any other number means the file moved since this plan was written — stop and re-read before editing.

- [ ] **Step 2: Add the type alias and the check function**

Add near the existing `opt_*` helpers:

```rust
/// The set of option names a widget has consumed.
///
/// The unknown-option check needs to know which keys were read. `opt_*` helpers
/// deliberately leave the dict untouched — mutating it would break callers who
/// reuse one dict across several widgets — so the record lives here instead.
type OptNames = std::collections::HashSet<String>;

/// Fail on any option key the widget did not consume.
///
/// Decided 2026-10-04: a misspelled option must be an error, not a silent
/// no-op. Any key in `opts` outside `used` is either a typo or an option this
/// binding does not expose, and silently doing nothing is the worst possible
/// answer.
///
/// Relies on this contract: every key supplied to a widget is consumed exactly
/// once by that widget's own option-processing path. A helper that reads
/// options without threading `used` through will make a supported option look
/// unknown, so delegated paths must pass it down.
unsafe fn reject_unknown_options(
    opts: &Bound<'_, PyDict>,
    used: &OptNames,
    widget: &str,
) -> PyResult<()> {
    let mut unknown: Vec<String> = opts
        .keys()
        .iter()
        .filter_map(|k| k.extract::<String>().ok())
        .filter(|k| !used.contains(k))
        .collect();
    if unknown.is_empty() {
        return Ok(());
    }
    unknown.sort();
    Err(PyValueError::new_err(format!(
        "{widget} got unknown option(s): {}. See the documentation for the \
         options this widget accepts.",
        unknown.join(", ")
    )))
}
```

`PyValueError` is already imported (it is used by the existing option helpers).

- [ ] **Step 3: Thread `used` through all ten helpers**

For each of the ten, add the parameter and record the name **before** reading, so a declared-but-absent option still counts as known:

```rust
unsafe fn opt_bool(opts: &Bound<'_, PyDict>, name: &str, used: &mut OptNames) -> PyResult<Option<bool>> {
    used.insert(name.to_string());
    match opts.get_item(name)? {
        Some(value) => Ok(Some(value.extract()?)),
        None => Ok(None),
    }
}
```

`get_item` stays. Do **not** change it to `remove_item`.

- [ ] **Step 4: Update every existing call site to pass a tracker**

All ~19 existing option-consuming functions must compile after this. Each creates a local `let mut used = OptNames::new();`, passes `&mut used` to its `opt_*` calls, and — **only in the six §6 target functions** — calls `reject_unknown_options` at the end.

For the existing containers, do NOT add the rejection call: that would surface unknown names in code that ships today, which is a separate change (recorded in `TODO.md`). They pass a tracker purely to compile.

```bash
cargo --version 2>/dev/null || echo "no local cargo (expected) - the check job is the verdict"
grep -c 'let mut used = OptNames::new();' src/lib.rs
```

Expected: no local cargo; the `used` count equals the number of functions that call `opt_*`.

- [ ] **Step 5: Prove the dict is not mutated**

Add to `tests/doc_snippets.py`'s pattern or as a new snippet file, a check that reuses one dict across two calls and asserts both still see every key. The shape:

```python
shared = {"hint_text": "name"}
text_edit_singleline(a, **shared)
text_edit_singleline(b, **shared)   # must still see hint_text
```

Assert the second call does not raise. If it does, the dict was mutated.

- [ ] **Step 6: Prove a delegated path does not falsely reject**

Add a snippet exercising a container that reads options in a helper — `frame(...)` and `window(...)` both do — passing an option each genuinely supports, and assert no error. This is Review Focus item 1 and the most likely regression in this task.

- [ ] **Step 7: Verify with a real compile and the examples**

```bash
git add src/lib.rs tests/
git commit -m "Add a consumed-key tracker for the unknown-option check

The check needs to know which option keys were read. Mutating the dict
with remove_item would break callers who reuse one options dict across
widgets, and would need &mut PyDict in all 25 signatures, so the helpers
record consumed names in a tracker and leave the dict alone."
git push fork HEAD:feature/egui-0.31-coverage
```

Then watch both jobs — `check` compiles, `examples` runs every app:

```bash
gh run list --repo ChetanKnowIT/pyegui --branch feature/egui-0.31-coverage -L 2 \
  --json databaseId,name,status,conclusion
gh run watch <check-run> --repo ChetanKnowIT/pyegui --exit-status
gh run watch <examples-run> --repo ChetanKnowIT/pyegui --exit-status
```

Expected: both green. A red `examples` here means a container's options broke — read the log for which app.

---

### Task 2: Slider options

**Files:**
- Modify: `src/lib.rs` — `slider_float`, `slider_float_response`, `slider_int`, `slider_int_response`

**Interfaces:**
- Consumes: `OptNames` and `reject_unknown_options` from Task 1.
- Produces: `slider_float_response(value, min, max, text, suffix=None, prefix=None, step_by=None, logarithmic=None, clamping=None, binary=None, **options)`, and the same shape on the other three. `slider_float` and `slider_int` delegate exactly as they do today.

- [ ] **Step 1: Verify the inventory against egui 0.31.1**

`TODO.md` §6 lists 23 `Slider` options. Check each against the 0.31.1 source (`egui-0.31.1/src/widgets/slider.rs`) and mark any that do not exist. Do not implement a name the pinned release does not have.

- [ ] **Step 2: Add the named parameters to the two `_response` functions**

```rust
#[pyo3(signature = (value, min, max, text, suffix=None, prefix=None, step_by=None, logarithmic=None, clamping=None, binary=None, **options))]
unsafe fn slider_float_response(
    value: &mut Float,
    min: f32,
    max: f32,
    text: &str,
    suffix: Option<&str>,
    prefix: Option<&str>,
    step_by: Option<f32>,
    logarithmic: Option<bool>,
    clamping: Option<bool>,
    binary: Option<bool>,
    options: Option<&Bound<'_, PyDict>>,
) -> PyResult<Response> {
    let ui = current_ui(&UI)?;
    let mut used = OptNames::new();
    let mut s = egui::Slider::new(&mut value.value, min..=max).text(text);

    if let Some(v) = suffix { used.insert("suffix".into()); s = s.suffix(v); }
    if let Some(v) = prefix { used.insert("prefix".into()); s = s.prefix(v); }
    if let Some(v) = step_by { used.insert("step_by".into()); s = s.step_by(v); }
    if let Some(v) = logarithmic { used.insert("logarithmic".into()); s = s.logarithmic(v); }
    if let Some(v) = clamping { used.insert("clamping".into()); s = s.clamping(v); }
    if let Some(v) = binary { used.insert("binary".into()); s = s.binary(v); }

    if let Some(o) = options {
        if let Some(v) = opt_string(o, "handle_shape", &mut used)? { s = s.handle_shape(v); }
        if let Some(v) = opt_f32(o, "drag_value_speed", &mut used)? { s = s.drag_value_speed(v); }
        if let Some(v) = opt_bool(o, "vertical", &mut used)? { s = s.vertical(v); }
        if let Some(v) = opt_bool(o, "show_value", &mut used)? { s = s.show_value(v); }
        if let Some(v) = opt_bool(o, "trailing_fill", &mut used)? { s = s.trailing_fill(v); }
        if let Some(v) = opt_bool(o, "update_while_editing", &mut used)? { s = s.update_while_editing(v); }
        if let Some(v) = opt_color32(o, "text_color", &mut used)? { s = s.text_color(v); }
        if let Some(v) = opt_i32(o, "fixed_decimals", &mut used)? { s = s.fixed_decimals(v); }
        if let Some(v) = opt_i32(o, "min_decimals", &mut used)? { s = s.min_decimals(v); }
        if let Some(v) = opt_i32(o, "max_decimals", &mut used)? { s = s.max_decimals(v); }
        reject_unknown_options(o, &used, "slider_float")?;
    }

    Ok(Response { inner: ui.add(s) })
}
```

`opt_string` and `opt_i32` are new — add them alongside the existing helpers, same shape. `slider_int_response` is the same with `.integer()` preserved; `binary` does not apply to the int slider in egui, so omit it there rather than accepting a name that does nothing.

- [ ] **Step 3: Update the two boolean wrappers**

`slider_float` and `slider_int` forward every new parameter to their `_response` twin, exactly as they forward `text` today. They keep returning `PyResult<()>`.

- [ ] **Step 4: Check `expected_exports.py` is untouched**

```bash
python tests/expected_exports.py 2>/dev/null || python -c "
import sys; sys.path.insert(0,'tests'); import expected_exports as m
print('total', len(m.CLASSES)+len(m.FUNCTIONS)+len(m.RESPONSE_VARIANTS))"
git diff --stat tests/expected_exports.py
```

Expected: 174 names, and no diff to `expected_exports.py`. §6 adds no exported name.

- [ ] **Step 5: Add the per-group tests**

```bash
grep -n 'slider_float_response\|slider_int_response' src/lib.rs | head
```

Add snippets: `suffix` renders in the slider text; `step_by` accepts a float; an unknown option (`sufix`) raises and names `slider_float`; a duplicate keyword raises.

- [ ] **Step 6: Assert the positional signature did not shift**

```python
slider_float(data, 0, 50, "Gain")          # existing usage, must still work
slider_float(data, min=0, max=50, text="Gain")
```

Both must succeed. `text` stays fourth positional.

- [ ] **Step 7: Assert a duplicate keyword raises**

```python
try:
    slider_float(d, 0, 1, "x", suffix="ms", **{"suffix": "s"})
except TypeError:
    pass   # expected
else:
    raise AssertionError("duplicate keyword silently accepted")
```

- [ ] **Step 8: Verify with `check` and `examples`, then commit**

Both green before committing. Commit message names the option count and the fact that `custom_formatter`/`custom_parser` are deliberately absent.

---

### Task 3: DragValue and TextEdit options, with the enum-string rule

**Files:**
- Modify: `src/lib.rs` — `drag_float*`, `drag_int*`, `text_edit_singleline*`, `text_edit_multiline*`

**Interfaces:**
- Consumes: Task 1's tracker; `opt_string`/`opt_i32`/`opt_usize` from Task 2.
- Produces: `drag_*_response(value, min, max, speed, suffix=None, prefix=None, clamping=None, **options)`; `text_edit_*_response(text, desired_width=None, desired_rows=None, char_limit=None, password=None, interactive=None, clip_text=None, **options)`.

- [ ] **Step 1: DragValue — named parameters and tail**

`suffix`, `prefix`, `clamping` named; `binary`, `hexadecimal`, `octal`, `fixed_decimals`, `min_decimals`, `max_decimals`, `update_while_editing`, `clamp_existing_to_range` via `opt_*`. `custom_formatter`/`custom_parser` are **not** implemented — closures, recorded in `TODO.md` with the reason, no stub.

- [ ] **Step 2: TextEdit — named parameters and tail**

`desired_width`, `desired_rows`, `char_limit`, `password`, `interactive`, `clip_text` named; `margin` via the existing `opt_margin`, `font` deferred (needs a `TextStyle` class, TODO §4), `background_color` via `opt_color32`, `lock_focus`/`frame`/`return_key`/`load_state`/`store_state`/`horizontal_align`/`vertical_align` checked against 0.31.1 and implemented only if they exist there.

- [ ] **Step 3: Add `opt_string`, `opt_i32`, `opt_usize` if Task 2 did not**

Same shape as the existing ten, recording into `used`.

- [ ] **Step 4: The enum-string rule, applied**

`horizontal_align` and `vertical_align` take strings matching egui's variant names. The error message must list the accepted values. Add a helper if it keeps the messages consistent:

```rust
/// Map a Python string to an egui enum variant, listing the accepted values on
/// failure. Case-exact: "Horizontal" is accepted, "horizontal" is not.
macro_rules! opt_enum {
    ($opts:expr, $name:expr, $used:expr, $target:ty, $($variant:ident),+ $(,)?) => { ... };
}
```

A macro is the right shape here because four options across two widgets need identical error text; if it fights borrow-checking, write them out — correctness first.

- [ ] **Step 5: Test enum strings both ways**

For each enum-valued option: every accepted spelling succeeds; `"horizontal"`, `"Horizontal "` and `""` are rejected; and the error message contains at least one valid variant name so the caller can self-correct.

- [ ] **Step 6: Verify `hint_text` and `editable` still work**

```python
text_edit_singleline(t, hint_text="name")
text_edit_singleline(t, editable=False)
```

Both are pre-existing public names. This is the preservation test for this group, and it must be behavioural where possible — `hint_text` is visible in the rendered frame.

- [ ] **Step 7: Verify both jobs, then commit**

---

### Task 4: DatePickerButton options

**Files:**
- Modify: `src/lib.rs` — `date_picker_button` and `date_picker_button_response` (both exist; verified)

**Interfaces:**
- Consumes: Task 1's tracker; the enum-string helper from Task 3.
- Produces: `date_picker_button(selection, format=None, show_icon=None, highlight_weekends=None, id_salt=None, **options)` and the same shape on `date_picker_button_response`.

- [ ] **Step 1: Verify the 12 options against egui 0.31.1**

`egui_extras` is pinned `=0.31.1` with the `datepicker` feature. Check each name in `egui_extras-0.31.1/src/datepicker.rs`. Implement only what is there.

- [ ] **Step 2: Named parameters and tail**

`format`, `show_icon`, `highlight_weekends`, `id_salt` named; `calendar`, `arrows`, `combo_boxes`, `calendar_week`, `year_scroll_to`, `start_end_years`, `reverse_years` via `opt_*`. `calendar` and `arrows` are enums and take strings under the Task 3 rule.

- [ ] **Step 3: Test both a valid and an invalid enum, and an unknown option**

- [ ] **Step 4: Verify both jobs, then commit**

---

### Task 5: `run_native` viewport and `NativeOptions`

The highest-risk group: `run_native` already takes `**kwargs`, so it is the one function where a mistake breaks every app in the repo.

**Files:**
- Modify: `src/lib.rs` — `run_native`, and `NativeOptions` handling

**Interfaces:**
- Consumes: Task 1's tracker.
- Produces: `run_native(app_name, update_func, title=None, position=None, visible=None, always_on_top=None, decorations=None, app_id=None, **kwargs)` with all eleven existing names preserved verbatim. `NativeOptions` gains `persistence_path`, `persist_window`, `centered`, `multisampling`.

- [ ] **Step 1: Record the exact existing kwargs names before touching anything**

```bash
python - <<'PY'
import re, pathlib
lib = pathlib.Path("src/lib.rs").read_text()
i = lib.index("fn run_native(")
print(sorted(set(re.findall(r'"(\w+)"', lib[i:i+6000]))))
PY
```

Expected to include: `fullscreen`, `icon_path`, `inner_height`, `inner_width`, `max_inner_height`, `max_inner_width`, `maximized`, `min_inner_height`, `min_inner_width`, `resizable`, `transparent`. These eleven must all still work.

- [ ] **Step 2: Add named parameters, keeping every existing kwarg name**

`title`, `position`, `visible`, `always_on_top`, `decorations`, `app_id` as named parameters; the fourteen missing viewport options through `opt_*`. The existing eleven keep arriving through `**kwargs` and are read exactly as today — do **not** rename `inner_width` to `width`.

- [ ] **Step 3: Add the rejection call, and check nothing else does**

`run_native` gains `reject_unknown_options(kwargs, &used, "run_native")?`. Every app in `examples/` and `guides/` calls `run_native`, so `examples` is the test: if a name were misspelled or dropped, every app fails.

- [ ] **Step 4: Behavioural preservation test, not resolution**

Assert the option reaches egui. The screenshot workflow can capture it, or an example can assert the viewport reports the requested size via `ctx`. What must NOT be the test: "the call does not raise" — that passes even if the value is dropped before `ViewportBuilder`.

- [ ] **Step 5: `NativeOptions` — the four reachable options**

`persistence_path`, `persist_window`, `centered`, `multisampling`. `glow_options` and `wgpu_options` are **deferred** — large nested structs, half of each irrelevant to the chosen renderer. Record in `TODO.md`; no stubs.

- [ ] **Step 6: Verify all eleven existing names still work**

```python
run_native("t", update, inner_width=800, inner_height=600, resizable=False)
run_native("t", update, fullscreen=True)
run_native("t", update, transparent=True)
```

- [ ] **Step 7: Verify both jobs, then commit**

---

### Task 6: Tests, docs, and the many-options benchmark

**Files:**
- Modify: `tests/doc_snippets.py` or new files under `tests/`
- Modify: `bench/bench.py`, `bench/src/main.rs`, `.github/workflows/benchmark.yml`
- Modify: `README.md`, `TODO.md`, `CHANGELOG.md`

**Interfaces:**
- Consumes: everything above.
- Produces: per-group behavioural tests; a `slider_many_options` scenario; §6 marked done in `TODO.md`.

- [ ] **Step 1: Add the six per-group behavioural tests**

One per group, each exercising a representative option end to end, so a failure names a group rather than §6. Review Focus item 3 is the shape to avoid: assert the observation, not that the call returned.

- [ ] **Step 2: Add the `slider_many_options` benchmark scenario**

`docs/performance.rst` measures one `hint_text` (~0.17 µs). An option lookup is paid once per *declared* option, not per supplied one, so 23 declared options on a call passing none is the worst case and is not what the existing number measures.

Add to both halves, following the `text_edit_*` precedent:

- Python: a slider passing every available option
- Rust: the same slider configured the same way
- Register the name in the Python and Rust scenario lists, the shell `VALID_SCENARIOS`, and `TWINLESS` if it has no Rust twin
- **Five coordinated edit sites** — nothing enforces the count, so verify all five

```bash
grep -rn 'text_edit_hint' bench/bench.py bench/src/main.rs .github/workflows/benchmark.yml | wc -l
```

That count is the number of places a new scenario must appear. Match it.

- [ ] **Step 3: Dispatch the benchmark and update the page**

```bash
gh workflow run benchmark --repo ChetanKnowIT/pyegui \
  --ref feature/egui-0.31-coverage -f widgets='50,500,2000' -f trials='5'
gh run watch <run-id> --repo ChetanKnowIT/pyegui --exit-status
```

Then add substitutions for the new scenario's ratio and overhead to `docs/performance.rst`. The gate warns rather than fails, so also read the page's numbers against the new snapshot yourself — a warning is easy to miss in a summary.

- [ ] **Step 4: Update `TODO.md` §6**

Mark each shipped option `[x]`. Record as **not implemented, with the reason**: `custom_formatter`/`custom_parser` (Rust closures), `glow_options`/`wgpu_options` (nested renderer structs), `font` on `TextEdit` (needs `TextStyle`, TODO §4). Add the follow-up: should `reject_unknown_options` be applied to the ~19 existing option-consuming functions, which are currently unvalidated?

- [ ] **Step 5: Update `README.md` and `CHANGELOG.md`**

A CHANGELOG entry for §6. The README's example section gains one option per widget family. Both are gated by `doc_claims.py`, which checks that every name the CHANGELOG presents as shipped is actually exported.

```bash
python tests/doc_claims.py; echo "exit=$?"
```

Expected: exit 0.

- [ ] **Step 6: Verify both jobs green, then commit**

## Sequencing

Six commits, one push each, so every workflow reads the same commit. Task 1 ships alone because it touches working code; Tasks 2–5 are independent once it lands; Task 6 needs the measurement.

A single commit was chosen over six (decision 3) because the option lists are mechanical once the helpers exist and six CI cycles buy less than the risk of a half-finished batch on the branch. The per-group tests in Task 6 exist to compensate: one commit, six focused tests, so a red run still names the guilty group.