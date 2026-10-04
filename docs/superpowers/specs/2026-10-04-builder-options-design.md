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
   indistinguishable from correct behaviour until the app is wrong. Implemented
   with a consumed-key tracker, never by mutating the caller's dict — see "The
   unknown-name check".

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

## The option inventory is pinned to egui 0.31.1

egui evolves, so a spec that says "Slider's 23 options" is a claim about a
release, not about `Slider`. The inventory this plan implements is derived from
**egui 0.31.1 exactly** (`Cargo.toml` pins `=0.31.1`; `check.yml` asserts every
`egui`/`eframe`/`egui_extras` in the tree resolves to 0.31.1 and fails the build
otherwise). Two consequences:

- Every option list in this document must be read against that tag's sources,
  the same discipline `TODO.md` already follows for its `Ui`-method census.
- If the pin moves, the inventory is re-derived rather than assumed. The counts
  in `TODO.md` §6 are the ones to re-check first.

The per-group lists in "Per-group notes" below are that inventory. They are
claims about 0.31.1 and should be verified against it during implementation,
not trusted because they appear in a document.

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

### PyO3 materialisation, verified rather than assumed

The proposed signature mixes named parameters with `**options`, and how PyO3
materialises the trailing keywords matters enough to check before writing code.
Verified against this repository's resolved versions: **pyo3 0.24.1** (`Cargo.toml`
requests `0.24.0`, the lockfile resolves 0.24.1) and **egui 0.31.1** (pinned `=0.31.1`).

Two facts from the code settle it:

- All 25 option-consuming pyfunctions already declare the dict as
  `Option<&Bound<'_, PyDict>>`, and `run_native` — the one function that mixes
  named parameters with trailing keywords today — receives `**kwargs` that way.
  So the mixed form is **already in use and already compiles**; §6 copies a
  working shape rather than introducing one.
- `Option<&Bound<'_, PyDict>>` is `None` when the caller passes no keywords.
  Every `opt_*` call site must therefore keep handling `None`, which it already
  does.

The one thing to confirm during implementation is that a named parameter
shadowing a dict key behaves as expected — e.g. `slider_float(v, 0, 1,
suffix="ms", **{"suffix": "s"})` must raise `TypeError` for the duplicate
keyword rather than silently picking one. That is a Python-level guarantee
rather than a PyO3 one, but it should be pinned by a test because §6 introduces
the first signature where a name can arrive twice.

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

Decision 2 needs a mechanism no existing `opt_*` call provides, and both drafts
of this spec got it wrong before the code was read.

Today `opt_bool` is:

```rust
unsafe fn opt_bool(opts: &Bound<'_, PyDict>, name: &str) -> PyResult<Option<bool>> {
    match opts.get_item(name)? {
        Some(value) => Ok(Some(value.extract()?)),
        None => Ok(None),
    }
}
```

It reads the key and leaves it in the dict, so "what remains after reading the
known keys" does not exist. Checking for leftovers needs a record of which keys
were consumed.

**Rejected: `remove_item`.** An earlier draft had each `opt_*` helper remove the
key it consumes, so the leftovers would be exactly the unrecognised names. That
was rejected on two independent grounds, the second of which was found by
reading the signatures:

1. **It is a backwards-incompatible mutation of a caller-owned object.** A
   caller who builds an options dict and passes it to two widgets currently has
   both calls see every key; after the change the first call consumes them.
   Nothing in this repository does that — every call site uses a literal dict —
   but it is user-visible, and introducing a silent behaviour change to released
   API purely to simplify a validation check is the wrong trade.
2. **It does not compile without a sweeping signature change.** All 25
   option-consuming pyfunctions take `Option<&Bound<'_, PyDict>>` — a *shared*
   reference. `remove_item` requires `&mut PyDict`. Mutating would mean changing
   the parameter type in all 25 signatures and importing `PyDictMut`, which is
   not currently imported. The approach was never a small change; it was a large
   one wearing a small one's clothes.

**Adopted: a consumed-key tracker.** Zero compatibility risk, at the cost of one
extra argument at each call site:

```rust
/// The set of option names a widget has consumed.
///
/// The unknown-option check needs to know which keys were read, because
/// `opt_*` helpers leave the dict untouched — deliberately, so that a caller
/// may reuse one dict across several widgets. Rather than mutate the caller's
/// dictionary, each read records the name here and the leftovers are computed
/// at the end.
type OptNames = std::collections::HashSet<String>;

/// Read an optional bool from `opts`, recording `name` as consumed.
unsafe fn opt_bool(
    opts: &Bound<'_, PyDict>,
    name: &str,
    used: &mut OptNames,
) -> PyResult<Option<bool>> {
    used.insert(name.to_string());
    match opts.get_item(name)? {
        Some(value) => Ok(Some(value.extract()?)),
        None => Ok(None),
    }
}
```

and at the end of the widget:

```rust
reject_unknown_options(options, &used, "slider_float")?;
```

```rust
/// Fail on any option key the widget did not consume.
///
/// Decided 2026-10-04: a misspelled option must be an error, not a silent
/// no-op. `used` is every name the option path read, so any key in `opts`
/// outside it is either a typo or an option this binding does not expose.
/// Either way the caller is wrong, and silently doing nothing is the worst
/// possible answer.
///
/// The contract this relies on: EVERY key supplied to a widget must be consumed
/// exactly once by that widget's own option-processing path, or the call fails.
/// Delegated helpers must therefore thread `used` through to their own `opt_*`
/// reads rather than taking a dict and reading behind the caller's back.
unsafe fn reject_unknown_options(
    opts: &Bound<'_, PyDict>,
    used: &OptNames,
    widget: &str,
) -> PyResult<()> {
    let unknown: Vec<String> = opts
        .keys()
        .iter()
        .filter_map(|k| k.extract::<String>().ok())
        .filter(|k| !used.contains(k))
        .collect();
    if unknown.is_empty() {
        return Ok(());
    }
    let mut unknown = unknown;
    unknown.sort();
    Err(PyValueError::new_err(format!(
        "{widget} got unknown option(s): {}. See the documentation for the \
         options this widget accepts.",
        unknown.join(", ")
    )))
}
```

`used.insert` happens unconditionally, whether or not the key is present, so a
declared-but-absent option is still "known" — which is correct, since the point
is to distinguish *names the widget does not implement* from *values the caller
did not supply*.

Two properties worth stating because they are what make this the right shape:

- **The caller's dict is never mutated.** Existing behaviour is preserved
  exactly, including reuse across widgets. That was the whole objection to
  `remove_item`.
- **A helper called twice for the same name is harmless**, unlike the
  `remove_item` version where the second call would silently return `None`.

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

**Enum strings are case-exact and must be tested both ways.** The failure mode is
a caller typing `"verts"` or `"horizontal"` and getting either an obscure error
or, worse, a silent default. So for every enum-valued option:

- The accepted set is documented as the exact egui variant names, spelled as
  egui spells them (`"Verts"`, not `"verts"`).
- The error message for an unrecognised value **lists the accepted values**.
  `"unknown handle_shape 'circle'; expected one of Verts, Circle"` is
  self-correcting; `"invalid handle_shape"` is not.
- A test asserts each accepted spelling succeeds and each near-miss
  (`"verts"`, `"VERTS"`, `""`) is rejected. `"circle"` — a real variant of
  `Slider`'s handle shape in egui — must be accepted, which is exactly the kind
  of thing a hand-written list gets wrong.

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
- **New: a preservation test that checks BEHAVIOUR, not resolution.** Asserting
  that `run_native(inner_width=800)` merely *resolves* is weak — it passes even
  if the value is accepted and then dropped before reaching `ViewportBuilder`.
  What must be asserted is that the option reaches egui: that the window comes
  up at the requested inner width, that `fullscreen=True` fullscreens, that
  `Slider`'s `suffix` appears in the rendered text. Where an option's effect is
  observable from Python, assert the observation. The repo's `examples/` and
  `guides/` are the natural home for this, since the examples job already runs
  them under Xvfb and can screenshot.
- **New: a many-options benchmark.** `docs/performance.rst` currently measures a
  single `hint_text` option (~0.17 µs). §6 adds up to 23 options on one widget,
  and the cost of an option lookup is paid once per *declared* option, not once
  per *supplied* one — so 23 declared options on a call that passes none is the
  worst case and is not what the existing number measures. Add a
  `slider_many_options` scenario pairing a bare `Slider` against one passing
  every available option, both against their Rust twins, and let the same
  warn-not-fail gate check the page. Without it the design's cost claim rests on
  a one-option extrapolation.
- **New: per-group tests, because one commit does not mean one test.** A single
  commit can still fail in a way that does not name the guilty group. Each of
  the six groups gets at least one test that exercises a representative option
  end to end, so a failure points at a group rather than at §6. This is
  independent of the commit-count decision: one commit, six focused tests.

Because §6 ships as one commit, a regression in any one group surfaces as one
red run naming one function. That is the accepted cost of decision 3.

## Risks

**The tracker threads a `&mut OptNames` through every `opt_*` call site.** Ten
helpers gain a parameter, and every existing container that reads options must
either thread it or opt out of the unknown-name check. A helper that reads
behind the caller's back — taking only the dict — is the failure mode, and it
produces a *false rejection*: an option the widget does support gets reported as
unknown because nobody recorded it. That is the more likely mistake, and it is
caught by the existing container examples.

**Delegated option paths must thread `used` consistently.** `window`,
`collapsing_response` and the rest read options inside helper functions. The
contract above ("every key must be consumed exactly once by that widget's own
option-processing path") is what makes this checkable, and a widget that accepts
an option through a delegate but checks in the outer function will reject valid
input. Per-group tests exist to catch exactly this.

**Does the unknown-name check apply to the existing containers too?** This spec
adds it to the six §6 targets. Applying it to the ~19 existing option-consuming
functions is a separate, larger change that would surface unknown names in code
that ships today. Leaving them unvalidated is inconsistent; changing them in this
plan risks breaking working apps on names pyegui happens not to implement. Not
doing it here — recorded in `TODO.md` as follow-up.

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