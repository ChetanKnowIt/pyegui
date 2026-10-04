# Builder options on existing widgets (§6) — design

**Date:** 2026-10-04
**Status:** awaiting review
**Scope:** §6 of `TODO.md` — the ~97 builder options pyegui cannot reach today.
Adds keyword options to `Slider`, `DragValue`, `TextEdit`, `DatePickerButton`,
`run_native`'s viewport, and `eframe::NativeOptions`. Not a new widget, not a
new container, no change to any existing exported name.

## Problem

egui's widgets are builders. `Slider` alone carries 23 options, `TextEdit` 18,
`DragValue` 13, `DatePickerButton` 12, `run_native`'s viewport 20 more than
pyegui exposes, `NativeOptions` 11. pyegui reaches almost none of them, so an
app that needs a slider with a suffix and a step cannot have one.

Three options are already reachable, and they are the precedent this design
follows rather than replaces:

| Where | Options today | Mechanism |
| --- | --- | --- |
| `text_edit_singleline` / `_multiline` (+ `_response`) | `hint_text`, `editable` | `**kwargs` → `opt_*` helpers |
| `image` (+ `_response`) | `max_width`, `max_height` | `**kwargs` → `opt_*` helpers |
| `run_native` | `inner_width`, `inner_height`, `min_*`, `max_*`, `fullscreen`, `maximized`, `resizable`, `transparent`, `icon_path` | `**kwargs` |
| all containers (`frame`, `collapsing`, `scroll_area_*`, `window`, …) | dozens | `**options` → `opt_*` helpers |

Ten `opt_*` helpers exist and are the established convention: `opt_bool`,
`opt_f32`, `opt_vec2`, `opt_color32`, `opt_corner_radius`, `opt_margin`,
`opt_order`, `opt_range`, `opt_scroll_bar_visibility`, `opt_stroke`. Each reads
one key from a `Bound<PyDict>` and returns `PyResult<Option<T>>`.

So the machinery is proven and the naming convention is settled. What is missing
is coverage.

**The benchmark work that landed the same day is directly relevant.** Run
37196277892 measured the kwargs path: **one `hint_text` option costs ~0.17 µs
of pyegui overhead per widget** (0.449 µs with the option, 0.275 µs without,
against a `label`'s 0.183 µs), and the `text_edit_*` ratio sits *below*
`label`'s. That is the cost this design accepts knowingly, and it is why the
option count per widget is a design question rather than a "add everything"
question. `docs/performance.rst` is the permanent home for that number.

## Decisions

Settled with the human partner on 2026-10-04. Do not re-open without new
information.

1. **Named parameters for the common options, `**kwargs` for the long tail.**
   `Slider` gets explicit parameters for what an app actually reaches for;
   the remaining options arrive through the `opt_*` machinery that already
   ships. Chosen because a 26-parameter signature is harder to read, not
   easier, and because the stated goal was that the API stays "easy to pick up
   when reading the documentation". A reader seeing `slider_float(v, 0, 100,
   suffix="ms")` learns something; a reader seeing 23 defaulted parameters
   learns that the count is large.

2. **An unknown option name is an error.** A misspelled `suffix` must not
   silently do nothing. Chosen over silent-ignore because pyegui cannot know
   what a future egui release will accept, and a silent no-op is
   indistinguishable from correct behaviour until the app is wrong.

3. **All six target groups ship in one commit, verified by one `check` run.**
   Not one commit per widget. Chosen for throughput: the option lists are
   mechanical once the helpers exist, and six CI cycles (~18 min) buy less
   than the risk that a half-finished batch sits on the branch. The cost is a
   coarser revert granularity, accepted knowingly.

4. **`TODO.md` §7 and the handoff doc keep their existing run figures.** Not
   reconciled against run 37196277892 in this plan. `docs/performance.rst`
   carries the newer measurement with its own provenance, and both older
   documents name the run they cite, so a reader sees two runs rather than a
   contradiction. Revisit when a full-input (3 counts × 5 trials) run is
   dispatched deliberately.

## The compatibility constraint that shapes everything

**Every option name pyegui accepts today must keep working, spelled exactly as
it is spelled.** These are public API, in released versions:

- `text_edit_*`: `hint_text`, `editable`
- `image`: `max_width`, `max_height`
- `run_native`: `inner_width`, `inner_height`, `min_inner_width`,
  `min_inner_height`, `max_inner_width`, `max_inner_height`, `fullscreen`,
  `maximized`, `resizable`, `transparent`, `icon_path`

Which of them are actually exercised by the repository's own apps is a narrower
set, and the distinction matters for testing. Verified by grepping `examples/`
and `guides/`:

| Name | Used by |
| --- | --- |
| `hint_text` | `examples/layout_and_sizing.py`, `examples/gallery.py`, `examples/widget_gallery.py`, `guides/copytext.py` |
| `max_width`, `max_height` | `examples/widget_gallery.py`, `examples/gallery.py` |
| `inner_width`, `inner_height` | `examples/gallery.py`, `examples/widget_gallery.py` |
| `resizable` | `examples/gallery.py` |
| `editable`, `fullscreen`, `maximized`, `transparent`, `icon_path`, all four `min_*`/`max_*` sizes | **not used anywhere in the repo** |

So the `examples` job covers six of the fourteen incidentally. The other eight —
including every `min_*`/`max_*` size name and `icon_path` — are public API with
**no coverage at all**, and a rename of any of them would ship green. That is
the argument for the explicit preservation test in "Testing" below rather than
relying on the examples to catch it.

A rename (`inner_width` → `width`) is a silent breaking change dressed as a
tidiness improvement, because a `TypeError` on an unexpected keyword argument
looks identical to a typo. So egui's own names are the names: `step_by` not
`step`, `fixed_decimals` not `decimals`, `drag_value_speed` not `speed`.
`Slider`'s existing positional `text` argument keeps its position — it is
already third positional in `slider_float(value, min, max, text)`.

## Design

### Two mechanisms, one per decision

**Named parameters** for the high-traffic options, declared in the
`#[pyo3(signature = ...)]` attribute:

```rust
#[pyo3(signature = (value, min, max, text, suffix=None, prefix=None, step_by=None, logarithmic=None, **options))]
unsafe fn slider_float_response(
    value: &mut Float,
    min: f32,
    max: f32,
    text: &str,
    suffix: Option<&str>,
    prefix: Option<&str>,
    step_by: Option<f32>,
    logarithmic: Option<bool>,
    options: Option<&Bound<'_, PyDict>>,
) -> PyResult<PyeguiResponse> { … }
```

pyo3 fills the named parameters from the call and passes the remainder as
`**options`. This is the same split `run_native` already uses for its viewport,
so it is not a new pattern.

**`opt_*` helpers** for the tail, unchanged in shape:

```rust
if let Some(v) = opt_bool(options, "binary")? { s = s.binary(v); }
if let Some(v) = opt_f32(options, "drag_value_speed")? { s = s.drag_value_speed(v); }
```

Helpers needed beyond the existing ten, all following the same
`PyResult<Option<T>>` shape: `opt_string`, `opt_usize`, `opt_i32`, `opt_f64`,
`opt_text_style`, and an `opt_handle_shape` for `Slider::handle_shape` (an
egui enum with no Python equivalent yet — see "Enums" below).

### The unknown-name check

Decision 2 needs a mechanism no existing `opt_*` call provides, and the spec's
first draft got the mechanism wrong. Corrected after reading the code.

Today `opt_bool` is:

```rust
unsafe fn opt_bool(opts: &Bound<'_, PyDict>, name: &str) -> PyResult<Option<bool>> {
    match opts.get_item(name)? {
        Some(value) => Ok(Some(value.extract()?)),
        None => Ok(None),
    }
}
```

It **reads** the key and leaves it in the dict — `opts.get_item`, not
`opts.remove_item`. So "what remains after reading the known keys" does not
exist: every key is still there afterwards. Checking for leftovers therefore
needs a *list of the names the widget knows*, which is a second list to maintain
and drifts the moment an option is added.

The alternative is to make the helpers consume the key:

```rust
/// Read an optional bool from `opts`, REMOVING the key so that whatever is
/// left after a widget has read everything it knows is exactly the set of
/// names it did not recognise.
unsafe fn opt_bool(opts: &Bound<'_, PyDict>, name: &str) -> PyResult<Option<bool>> {
    match opts.remove_item(name)? {
        Some(value) => Ok(Some(value.extract()?)),
        None => Ok(None),
    }
}
```

Then the check needs no known-names list at all:

```rust
/// Fail on any option key the widget did not consume.
///
/// Decided 2026-10-04: a misspelled option must be an error, not a silent
/// no-op. Every `opt_*` read removes the key it consumes, so whatever remains
/// in `opts` is either a typo or an option this binding does not expose.
/// Either way the caller is wrong, and silently doing nothing is the worst
/// possible answer.
unsafe fn reject_unknown_options(opts: &Bound<'_, PyDict>, widget: &str) -> PyResult<()> {
    if opts.is_empty() {
        return Ok(());
    }
    let mut names: Vec<String> = opts
        .keys()
        .iter()
        .map(|k| k.extract::<String>())
        .collect::<PyResult<_>>()?;
    names.sort();
    Err(PyValueError::new_err(format!(
        "{widget} got unknown option(s): {}. See the documentation for the \
         options this widget accepts.",
        names.join(", ")
    )))
}
```

It must run **last** in every function that uses it, after all `opt_*` reads —
including reads inside helper functions that option handling delegates to.

Two consequences worth stating because they are easy to get wrong:

- **`get_item` → `remove_item` changes the signature's behaviour for callers
  that reuse one dict across calls.** Today a caller may pass the same options
  dict to two widgets and have both see every key; after this change the first
  call consumes them. That is correct for the stated use (a literal dict at the
  call site) but is a real change for anyone building a dict programmatically
  and reusing it. It is a behaviour change to documented-as-stable `text_edit_*`
  and `image`, and the release notes must say so.
- **A helper called twice for the same name now returns `None` the second
  time.** No current helper is, but it means the consume-a-key discipline is
  part of each helper's contract, not just an implementation detail.

### Enums with no Python equivalent

Four of the target options are egui enums or types pyegui has no class for:
`Slider`'s `handle_shape`, `TextEdit`'s `horizontal_align` / `vertical_align`,
`DatePickerButton`'s `calendar` / `arrows`. Per the established pattern from
the handoff's settled decisions — tuples for geometry, a class only where egui's
shape has no tuple equivalent — these take **strings** matching egui's variant
names (`"Verts"`, `"Horizontal"`, `"Arrow"`, `"Grid"`). A string that matches no
variant is an error naming the accepted values.

Building four enum classes instead would be consistent with `Color32` and `Rect`,
which exist as classes because their shape is genuinely not a tuple. These are
closed string sets, so a class would be a wrapper around one string with no
behaviour. Revisit if a future option needs associated data.

### Per-group notes

| Group | Named | `opt_*` / notes |
| --- | --- | --- |
| `Slider` / `SliderInt` | `suffix`, `prefix`, `step_by`, `logarithmic`, `clamping`, `binary` | `custom_formatter`/`custom_parser` are Rust closures — **not reachable**, see below |
| `DragValue` | `suffix`, `prefix`, `speed`, `clamping` | `custom_formatter`/`custom_parser` not reachable |
| `TextEdit` | `desired_width`, `desired_rows`, `char_limit`, `password`, `interactive`, `clip_text` | `font` needs `TextStyle`; `margin` reuses `opt_margin` |
| `DatePickerButton` | `format`, `show_icon`, `highlight_weekends`, `id_salt` | `calendar`/`arrows` take strings |
| `run_native` viewport | `title`, `position`, `visible`, `always_on_top`, `decorations`, `app_id` | 14 more via `opt_*`; **all 11 existing names preserved verbatim** |
| `NativeOptions` | `persistence_path`, `persist_window`, `centered`, `multisampling` | `glow_options`/`wgpu_options` need their own structs — defer |

### Options that are genuinely not reachable

Four of `Slider`'s 23 and two of `DragValue`'s 13 are Rust closures:
`custom_formatter` and `custom_parser`. egui types them
`impl Fn(&f64) -> String` and `impl Fn(&str) -> Result<f64, _>`. A Python
callable cannot be handed to a Rust closure without a trampoline and a
lifetime strategy, which is its own design problem — the same reasoning that
deferred drag-and-drop payloads in §1 (`Arc<dyn Any + Send + Sync>` has no clean
Python mapping).

These ship as **explicitly not implemented**, recorded in `TODO.md` with the
evidence, exactly as `total_drag_delta` and `Popup` were handled. Shipping a
stub that silently ignores them would be worse than absence.

`NativeOptions`' `glow_options` and `wgpu_options` are deferred for the same
reason in a different guise: they are large nested option structs, and half of
each is irrelevant depending on the chosen renderer. `TODO.md` records them
open.

## Testing

No local Rust toolchain — every claim comes from a workflow run, per the repo's
standing constraint.

- **`check`** — `cargo check --locked --all-targets`, advisory clippy, the
  egui-version assertion, and the export gate. A signature change alters no
  exported *name*, so `tests/expected_exports.py` needs no new entries — but
  that must be verified, not assumed.
- **`examples`** — runs every app and every runnable README snippet, which is
  what actually proves the preserved names still work. The `guides/` and
  `examples/` trees use `hint_text`, `max_width`, `max_height` and the
  `run_native` viewport names; if a rename slipped through, those fail.
- **New: an unknown-option test.** Nothing in CI currently proves decision 2.
  Add a snippet to `tests/doc_snippets.py` (or a case in the examples job)
  asserting that `slider_float(v, 0, 1, sufix="x")` raises and names the
  widget. Without it, decision 2 is an intention rather than a behaviour.
- **New: a preservation test.** Assert the eleven `run_native` names and the
  three widget-level names still resolve. The examples cover this incidentally;
  an explicit test makes it deliberate.

Because §6 ships as one commit, a regression in any one group surfaces as one
red run naming one function. That is the accepted cost of decision 3.

## Risks

**The unknown-name check requires changing all ten `opt_*` helpers.** Making
them `remove_item` instead of `get_item` is a change to code that already works,
used by every container. If a helper is called twice for the same name, or a
widget's `reject_unknown_options` runs before its last read, it breaks.
Mitigation: `check` and `examples` both exercise the containers, so a mistake is
loud.

**It is also a behaviour change to released API, not an internal tidy-up.** A
caller who builds an options dict and passes it to two widgets currently has
both calls see every key; after this change the first consumes them. Nothing in
the repo does this — every call site uses a literal dict — but it is user-visible
and belongs in the release notes.

**Enum-as-string is a precedent with a cost.** Four options get strings today.
If a future option needs associated data, this decision has to be revisited for
that option, and the two spellings will coexist. Accepted: a class per enum is
a class per enum to maintain and test, for a closed set of names.

**One commit means one revert granularity.** Accepted per decision 3.

**`Slider`'s positional `text` argument.** Adding named parameters after it
must not disturb its position — `slider_float(v, 0, 100, "Gain")` is existing
usage. pyo3 keeps positional order, so this holds, but it is worth an explicit
test since it is the one thing a signature change here can break silently.

## Out of scope

- No new exported names, so `tests/expected_exports.py` is unchanged.
- No new widget classes; `Slider`/`DragValue` reuse the existing `Float`/`Int`
  holders.
- `with_layout` and the whole `egui::Layout` — that is §4's remaining item and
  needs a different decision.
- Drag-and-drop payloads — §1, deferred on the same closure-mapping grounds.
- `App::save` / `Storage` is in scope for a later batch in this plan; it is
  listed here so the option work and the persistence work are not confused.

## Follow-up this plan does NOT do

`TODO.md` §7 and `docs/handoff-egui-031.md` continue to cite run 37106337662
(1.33x). This plan adds no benchmark run, so those figures stay accurate as
history. When a full-input run is next dispatched deliberately, both documents
should be reconciled in that change, and `docs/performance.rst` will warn if the
page drifts from the new snapshot.