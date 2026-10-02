# Containers Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give pyegui the egui 0.31.1 containers — `Window`, `SidePanel`, `TopBottomPanel`, `Modal`, `Popup` — by making Python, not `run_native`, own frame composition.

**Architecture:** `run_native` stops opening `CentralPanel` itself and instead calls Python inside `egui::Context::run` with only a `Context`, matching egui's own `examples/custom_keypad/src/main.rs`. Python draws panels in order and calls `central_panel` last. Each container function takes the `Context` explicitly, shows the egui container, and pushes the inner `Ui` onto the existing global `UI_STACK` for the duration of the Python callback — reusing `run_nested_update_func_lossy`, which already implements push/call/pop.

**Tech Stack:** Rust, PyO3 0.24 (abi3), egui/eframe pinned to `=0.31.1`, GitHub Actions as the only compiler.

**Spec:** `docs/superpowers/specs/2026-10-02-containers-design.md`

## Global Constraints

- egui, eframe and egui_extras stay pinned to exactly `=0.31.1`. `check.yml` asserts it; do not touch `Cargo.toml` pins.
- **CI is the only compiler.** There is no local Rust toolchain and none will be installed. Every verdict is a `check` run on `fork`.
- egui source is authoritative. Before writing any binding, read the 0.31.1 signature from `crates/egui/src/containers/*.rs`; do not infer from newer docs or from memory.
- No migration shim, no deprecation period for `central_panel`. One note in the README.
- This is a **breaking change shipping as 0.6.0**. Bump `version` in `Cargo.toml`.
- Every new exported name goes in `tests/expected_exports.py`. The export gate is bidirectional: a name exported but undeclared fails CI, and a name declared but unexported fails CI.
- Container functions take `ctx: &Context` as their first positional argument, matching the crate's existing explicit-state discipline (`checkbox(data, "check me")`). No new global state and no thread-locals.
- Keep `Window` first: it exercises the whole mechanism alone.

## Review Focus

Five failure modes the spec implies. Each has a test in the task named below. Most likely to bite first.

1. **A container drawn in the wrong order still draws.** A `Ui` resolved from the wrong stack slot renders content in the wrong place with no error. Every layout assertion must check *position*, not just "not blank". (Tasks 4, 6)
2. **Python never draws a panel.** An unmigrated `update_func` renders a blank window with no diagnostic — the single worst outcome of this change. (Task 2)
3. **An unknown builder option silently does nothing.** `window(..., resizble=True)` must raise, not quietly produce a fixed window. (Task 3)
4. **`Window::open` borrow lifetime.** `.open(&mut bool.value)` must satisfy egui's `&'open mut bool` for the whole `.show()` call, or the borrow checker rejects it. (Task 4)
5. **2-tuples vs. `Vec2`.** `default_size=(400.0, 300.0)` must work now, and must switch cleanly when a `Vec2` class lands in §4. (Task 3)

---

### Task 1: `run_native` yields frame composition to Python

**Files:**
- Modify: `src/lib.rs` — `impl eframe::App for PyeguiApp`, the `fn update` body

**Interfaces:**
- Consumes: nothing from other tasks.
- Produces: `update_func(ctx)` is called inside `egui::Context::run` with **no** `CentralPanel` opened by pyegui. Task 2 adds `central_panel`; Tasks 3–5 depend on this ordering.

- [ ] **Step 1: Read the current `fn update` and confirm the shape**

Read `src/lib.rs`, find `fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame)`. Confirm it currently wraps the Python call in `egui::CentralPanel::default().show(ctx, |ui| { ... })` and that it pushes `&raw mut *ui` onto `UI.as_mut()` before calling `self.update_func.call1((ctx_r,))`.

- [ ] **Step 2: Replace the body so Python owns the frame**

The new body keeps the per-frame `Context` clone and the existing error display, but drops `CentralPanel` and the UI-stack push:

```rust
fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    let ctx_r = Context(ctx.clone());

    unsafe {
        egui::Context::run(ctx.clone(), |_ctx| {
            Python::with_gil(|py| {
                if let Err(err) = self.update_func.call1((ctx_r,)) {
                    err.display(py);
                }
            });
        });
    }
}
```

`Context::run` is how egui's own `__run_test_ctx` gives a body bare context access. If the borrow checker rejects `ctx.clone()` into the closure, use `egui::Context::run(ctx.clone(), ..)` with the outer `ctx` shadowed — the closure needs no `Context` argument since Python receives `ctx_r`.

**Do not** remove `CentralPanel` from anywhere else. `debug.py`'s `Group` and `pages.py`'s `Layout` keep working because they go through `UI_STACK`, which this task leaves untouched.

- [ ] **Step 3: Confirm nothing else in the crate opened CentralPanel**

Run: `cd /home/chetanamrao/Documents/projects/py_ui/pyegui && git grep -n 'CentralPanel' -- src/lib.rs`
Expected: exactly one occurrence, inside `fn update`. Two or more means a second site needs removing.

- [ ] **Step 4: Push and read the `check` verdict**

```bash
git add src/lib.rs
git commit -m "run_native no longer opens CentralPanel itself"
git push fork feature/egui-0.31-coverage
gh run list --repo ChetanKnowIT/pyegui --branch feature/egui-0.31-coverage \
  --json databaseId,name,conclusion -q '.[] | "\(.databaseId) \(.name) \(.conclusion)"'
gh run watch <check-run-id> --repo ChetanKnowIT/pyegui --exit-status
```

Expected: `check` success. Clippy 0 diagnostics. This task changes no exports, so the export gate must still pass untouched — a failure here means something else in the crate depended on the panel.

**Nothing renders yet after this task.** Every app is broken until Task 2. That is expected and is why the migration is sequenced after the verdict is green.

- [ ] **Step 5: Commit**

Already committed in Step 4. No separate commit.

---

### Task 2: `central_panel` and the no-panel error

**Files:**
- Modify: `src/lib.rs` — new `pyfunction`, `#[pymodule]` registration
- Modify: `tests/expected_exports.py` — `FUNCTIONS`

**Interfaces:**
- Consumes: Task 1's frame composition. This is the function that makes apps draw again.
- Produces: `central_panel(ctx: &Context, contents: Bound<PyAny>) -> PyResult<()>`. Tasks 3–5 mirror this exact signature shape for their containers.

- [ ] **Step 1: Add `central_panel` to the declared exports**

In `tests/expected_exports.py`, add `"central_panel"` to the `FUNCTIONS` set (the plain set, **not** `RESPONSE_VARIANTS` — it returns nothing).

- [ ] **Step 2: Push and confirm the gate fails for the right reason**

```bash
git add tests/expected_exports.py
git commit -m "Declare central_panel in the export gate"
git push fork feature/egui-0.31-coverage
```

Expected: `check` **fails** with `declared but not exported: ['central_panel']`. A pass here means the gate is not actually reading your edit — stop and check the file was staged.

- [ ] **Step 3: Implement `central_panel` in `src/lib.rs`**

Place it near the other container functions. Signature and body:

```rust
/// Draws the central panel. Must be called last in update_func, after any
/// side panels, top/bottom panels or modals.
///
/// egui requires CentralPanel to be added after every other top-level
/// panel; see the panel docs in egui 0.31.1. Python owns frame composition
/// here, exactly as an egui app does.
///
/// Example::
///
///     def update_func(ctx):
///         side_panel_left(ctx, "nav", nav_contents)
///         central_panel(ctx, main_contents)
#[pyfunction]
unsafe fn central_panel(ctx: &Context, contents: Bound<'_, PyAny>) -> PyResult<()> {
    let ctx = &ctx.0;
    egui::CentralPanel::default().show(ctx, |ui| {
        run_nested_update_func_lossy(ui, contents.clone())
    });
    Ok(())
}
```

Use `run_nested_update_func_lossy`, not `run_nested_update_func`: egui types this callback as `impl FnOnce(&mut Ui)` returning `()`, so a `PyResult` return type does not fit. This is the same reason `on_hover_ui` uses the lossy variant.

`CentralPanel::show` returns `InnerResponse<R>`; ignore it rather than returning the inner value.

- [ ] **Step 4: Register it in `#[pymodule]`**

In `fn pyegui`, add `m.add_function(wrap_pyfunction!(central_panel, m)?)?;` alongside the other `wrap_pyfunction!` calls.

- [ ] **Step 5: Push and read the verdict**

Expected: `check` success with 122 exported names.

- [ ] **Step 6: Migrate `examples/hello_world.py` as the first real proof**

Rewrite it so the widgets are drawn inside the panel callback:

```python
from pyegui import *

def main_contents():
  heading("Hello, World!")

def update_func(ctx):
  central_panel(ctx, main_contents)

if __name__ == "__main__":
  run_native("Hello World App", update_func)
```

The split into `main_contents` is deliberate: widgets resolve their `Ui` from `UI_STACK`, which only has an entry while the panel callback runs. Drawing them directly in `update_func` would hit `UI_STACK is empty`.

- [ ] **Step 7: Verify the migration renders, via the screenshot workflow**

```bash
gh workflow run screenshot.yml --repo ChetanKnowIT/pyegui \
  --ref feature/egui-0.31-coverage
```

This will fail its blank-frame check until Task 6 adds a `containers` page — that failure is expected at this point. Confirm instead that `hello_world`-style content appears in the pages that do render, by downloading the artifact and opening a PNG. If every page is blank, Task 1's ordering is wrong and must be fixed before continuing.

- [ ] **Step 8: Commit**

```bash
git add src/lib.rs examples/hello_world.py
git commit -m "Add central_panel and migrate hello_world to it"
git push fork feature/egui-0.31-coverage
```

---

### Task 3: Shared option parsing, with a loud unknown-option error

**Files:**
- Modify: `src/lib.rs` — new private helper, used by Tasks 4–5

**Interfaces:**
- Consumes: nothing.
- Produces:
  ```rust
  unsafe fn take_option<'py>(
      opts: &Bound<'py, PyDict>,
      name: &str,
  ) -> Option<Bound<'py, PyAny>>
  ```
  plus helpers the container functions call:
  - `unsafe fn opt_bool(opts: &Bound<PyDict>, name: &str) -> PyResult<Option<bool>>`
  - `unsafe fn opt_vec2(opts: &Bound<PyDict>, name: &str) -> PyResult<Option<egui::Vec2>>` — reads a 2-tuple `(x, y)` of floats
  - `unsafe fn opt_id(opts: &Bound<PyDict>, name: &str) -> PyResult<Option<egui::Id>>`

  Unknown names are checked once by `validate_options(opts, KNOWN)`, which raises `PyValueError` listing every unrecognised key. Callers must not silently ignore an unknown key.

- [ ] **Step 1: Write the helper**

`validate_options` takes a `&[&str]` of every key the container accepts and raises on anything else:

```rust
unsafe fn validate_options(opts: &Bound<'_, PyDict>, known: &[&str]) -> PyResult<()> {
    let mut unknown: Vec<String> = Vec::new();
    for key in opts.keys().iter() {
        let name: String = key.extract()?;
        if !known.contains(&name.as_str()) {
            unknown.push(name);
        }
    }
    if !unknown.is_empty() {
        return Err(PyValueError::new_err(format!(
            "unknown option(s): {}. Known: {}",
            unknown.join(", "),
            known.join(", ")
        )));
    }
    Ok(())
}
```

Raise on **every** unknown key, not the first — one run should reveal every typo.

`opt_vec2` reads a 2-sequence and returns `egui::Vec2::new(x, y)`, raising `PyValueError` if it is not length 2 or holds non-floats.

- [ ] **Step 2: Prove it raises, before any container uses it**

There is no local Python test harness for the Rust side — CI is the only place this compiles. So cover the helper from Task 4's gallery page rather than inventing a unit-test file. Record in Task 4's steps that the unknown-option error is exercised there.

- [ ] **Step 3: Commit**

```bash
git add src/lib.rs
git commit -m "Add container option parsing that rejects unknown keys"
```

Do not push yet — Task 4 needs these helpers and a push costs a run. Push at the end of Task 4.

---

### Task 4: `window`, driven by a `Bool`

**Files:**
- Modify: `src/lib.rs` — new `pyfunction`, registration
- Modify: `tests/expected_exports.py` — `FUNCTIONS`
- Modify: `examples/gallery.py` — new `containers` page

**Interfaces:**
- Consumes: `run_nested_update_func_lossy` (existing), `opt_bool`/`opt_vec2` (Task 3).
- Produces:
  ```rust
  #[pyo3(signature = (ctx, title, id, contents, open=None, **options))]
  unsafe fn window(
      ctx: &Context,
      title: &str,
      id: &str,
      contents: Bound<'_, PyAny>,
      open: Option<&mut Bool>,
      options: Option<&Bound<'_, PyDict>>,
  ) -> PyResult<bool>
  ```
  Returns `true` when the window was visible this frame; `false` when collapsed or closed.

- [ ] **Step 1: Declare `window` and push to confirm the gate fails**

Add `"window"` to `FUNCTIONS` in `tests/expected_exports.py`. Push. Expect `check` to fail with `declared but not exported: ['window']`.

- [ ] **Step 2: Verify the egui signature before writing the binding**

Read `crates/egui/src/containers/window.rs` at tag 0.31.1 and confirm:

```rust
pub fn show<R>(self, ctx: &Context, add_contents: impl FnOnce(&mut Ui) -> R)
    -> Option<InnerResponse<Option<R>>>
pub fn open(mut self, open: &'open mut bool) -> Self
pub fn id(mut self, id: Id) -> Self
```

The `Option` return is why `window` returns `bool`. The `'open` lifetime on `open` is the borrow risk in Review Focus.

- [ ] **Step 3: Implement `window`**

```rust
#[pyfunction]
#[pyo3(signature = (ctx, title, id, contents, open=None, **options))]
unsafe fn window(
    ctx: &Context,
    title: &str,
    id: &str,
    contents: Bound<'_, PyAny>,
    open: Option<&mut Bool>,
    options: Option<&Bound<'_, PyDict>>,
) -> PyResult<bool> {
    let ctx = &ctx.0;
    let mut builder = egui::Window::new(title).id(egui::Id::new(id));

    if let Some(opts) = options {
        apply_window_options(&mut builder, opts)?;
    }

    let shown = if let Some(open) = open {
        builder
            .open(&mut open.value)
            .show(ctx, |ui| run_nested_update_func_lossy(ui, contents.clone()))
            .is_some()
    } else {
        builder
            .show(ctx, |ui| run_nested_update_func_lossy(ui, contents.clone()))
            .is_some()
    };

    Ok(shown)
}
```

Borrow `&mut open.value` — `Bool.value` is an owned field, so the borrow lives exactly as long as the `.show()` call and satisfies `'open`. Do **not** try to hold the borrow across the `if let`; the two branches above keep it scoped to each `.show()`.

- [ ] **Step 4: Implement `apply_window_options`**

Takes `&mut egui::Window` and the dict. Validate first:

```rust
const WINDOW_OPTIONS: &[&str] = &[
    "default_size", "default_pos", "min_size", "max_size",
    "min_width", "min_height", "max_width", "max_height",
    "default_width", "default_height", "anchor",
    "fixed_pos", "fixed_size",
    "resizable", "collapsible", "title_bar", "movable",
    "scroll", "vscroll", "hscroll", "auto_sized",
    "enabled", "fade_in", "fade_out", "interactable",
    "order", "constrain",
];
```

Then apply, each guarded by its `opt_*` returning `Some`:

| option | egui call |
|---|---|
| `default_size`, `min_size`, `max_size`, `fixed_size` | `.default_size` / `.min_size` / `.max_size` / `.fixed_size` with `Vec2` |
| `default_pos`, `fixed_pos` | `.default_pos` / `.fixed_pos` with `Pos2` |
| `min_width`, `max_width`, `default_width` | `.min_width` / `.max_width` / `.default_width` with `f32` |
| `min_height`, `max_height`, `default_height` | same, `f32` |
| `resizable` | `.resizable(bool)` |
| `collapsible`, `title_bar`, `movable`, `scroll`, `vscroll`, `hscroll`, `auto_sized`, `enabled`, `fade_in`, `fade_out`, `interactable`, `constrain` | `.name(bool)` |
| `order` | `.order(f32)` |
| `anchor` | `.anchor(Align2)` — leave unimplemented this round; omit it from the table and from `WINDOW_OPTIONS` rather than guessing the `Align2` conversion. Note its absence in the spec. |

`scroll` exists as a real 0.31.1 method but is marked deprecated in favour of `hscroll`/`vscroll`; verify in the source and, if deprecated, drop it from `WINDOW_OPTIONS` too rather than shipping a call that raises a clippy warning.

- [ ] **Step 5: Register `window` in `#[pymodule]`**

- [ ] **Step 6: Add a `containers` page to `examples/gallery.py`**

Add a page function to the `PAGES` list. It must exercise, in this order:

1. a `window` whose `open` is a module-level `Bool(True)` — set it `False` from a button inside the window so open/close is visible across frames
2. a second `window` with `default_size=(400.0, 300.0)` and `resizable=True`, proving the 2-tuple path
3. a window nested inside `collapsing`, and a `collapsing` inside a window

Widgets must be drawn inside the callbacks, never directly in the page function, because `UI_STACK` only has an entry while a callback runs.

Register the page so `gallery.py list` includes it — CI enumerates pages from that output.

- [ ] **Step 7: Push and read the verdict**

```bash
git add src/lib.rs tests/expected_exports.py examples/gallery.py
git commit -m "Add window with builder options and a Bool-driven open state"
git push fork feature/egui-0.31-coverage
```

Expect `check` success, 123 exported names, clippy 0.

- [ ] **Step 8: Verify the render, and check position not just non-emptiness**

```bash
gh run watch <screenshot-run-id> --repo ChetanKnowIT/pyegui --exit-status
gh run download <screenshot-run-id> --repo ChetanKnowIT/pyegui \
  --name gallery-screenshots --dir /tmp/containers
```

Open `/tmp/containers/containers.png`. Confirm: the window has a title bar with the title text; the `Bool`-driven window actually closes when its button is pressed; the resizable window is visibly the requested size. This is Review Focus #1 — a window drawn in the wrong place still renders, so position is the assertion, not non-blankness.

- [ ] **Step 9: Verify the `open` borrow survives across frames**

Review Focus #4: the `&'open mut bool` lifetime must hold for the whole
`.show()` call. Compile-time verified by Task 4 Step 7 passing, but the
runtime behaviour is separate — add to the gallery page a window whose
`Bool` starts `True`, with a button inside that sets it `False`, plus one
that sets it back to `True`. Then dispatch the workflow twice and confirm
from the two rendered PNGs that the window is present in one and absent in
the other. A `Bool` that never takes effect looks identical to a window
that was never wired up.

- [ ] **Step 10: Exercise the unknown-option error**

Temporarily add a bad option to the gallery page, e.g. `window(ctx, "t", "i", contents, resizble=True)`. Push, and confirm `check` fails with `unknown option(s): resizble` rather than the run passing silently. Revert and push again. Two pushes, and this is the second — batch it with any other pending edit.

---

### Task 5: Panels, `Modal`, `Popup`

**Files:**
- Modify: `src/lib.rs` — six new `pyfunction`s, registration
- Modify: `tests/expected_exports.py` — `FUNCTIONS`
- Modify: `examples/gallery.py` — extend the `containers` page

**Interfaces:**
- Consumes: `opt_bool`, `opt_vec2`, `validate_options` (Task 3).
- Produces, all `(ctx, id, contents, **options)` unless noted:

```rust
unsafe fn side_panel_left(ctx: &Context, id: &str, contents: Bound<PyAny>, options: ...) -> PyResult<()>
unsafe fn side_panel_right(ctx: &Context, id: &str, contents: Bound<PyAny>, options: ...) -> PyResult<()>
unsafe fn top_panel(ctx: &Context, id: &str, contents: Bound<PyAny>, options: ...) -> PyResult<()>
unsafe fn bottom_panel(ctx: &Context, id: &str, contents: Bound<PyAny>, options: ...) -> PyResult<()>
unsafe fn modal(ctx: &Context, id: &str, contents: Bound<PyAny>, options: ...) -> PyResult<bool>
unsafe fn popup(ctx: &Context, contents: Bound<PyAny>, options: ...) -> PyResult<()>
unsafe fn popup_menu(ctx: &Context, contents: Bound<PyAny>, options: ...) -> PyResult<()>
```

`modal` returns `bool` — `ModalResponse::should_close()`. The panels and `popup` return nothing; their `InnerResponse` carries nothing Python needs.

- [ ] **Step 1: Verify all six signatures at 0.31.1 first**

Read `containers/panel.rs`, `containers/modal.rs`, `containers/popup.rs` and confirm before writing:

```rust
pub fn left(id: impl Into<Id>) -> Self          // SidePanel
pub fn right(id: impl Into<Id>) -> Self
pub fn top(id: impl Into<Id>) -> Self           // TopBottomPanel
pub fn bottom(id: impl Into<Id>) -> Self
pub fn show<R>(self, ctx: &Context, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R>
pub fn show<T>(self, ctx: &Context, content: impl FnOnce(&mut Ui) -> T) -> ModalResponse<T>  // Modal
```

For `popup`, confirm whether `Popup::from_context` is the 0.31.1 name and what it returns. If it takes a `Response` rather than a `Context`, that changes the signature — report it rather than guessing.

- [ ] **Step 2: Declare all six in `FUNCTIONS`, push, confirm the gate fails**

Expect `declared but not exported: [...]` listing all six.

- [ ] **Step 3: Implement the four panels**

Each is the same shape. `side_panel_left` in full:

```rust
#[pyfunction]
#[pyo3(signature = (ctx, id, contents, **options))]
unsafe fn side_panel_left(
    ctx: &Context,
    id: &str,
    contents: Bound<'_, PyAny>,
    options: Option<&Bound<'_, PyDict>>,
) -> PyResult<()> {
    let ctx = &ctx.0;
    let mut builder = egui::SidePanel::left(egui::Id::new(id));

    if let Some(opts) = options {
        apply_panel_options(&mut builder, opts)?;
    }

    builder.show(ctx, |ui| run_nested_update_func_lossy(ui, contents.clone()));
    Ok(())
}
```

The other three differ only in the constructor: `egui::SidePanel::right`, `egui::TopBottomPanel::top`, `egui::TopBottomPanel::bottom`. `apply_panel_options` accepts `default_width`, `default_height`, `resizable`, `show_separator`, `exact_width`, `exact_height` — verify each exists in 0.31.1 `panel.rs` and drop any that does not.

- [ ] **Step 4: Implement `modal`**

```rust
let response = builder.show(ctx, |ui| run_nested_update_func_lossy(ui, contents.clone()));
Ok(response.should_close())
```

`should_close` takes `&self`, so call it on the returned value directly.

- [ ] **Step 5: Implement `popup`**

Follow whatever Step 1 established. If `Popup::from_context` needs a `Response` rather than a `Context`, take the response as an extra argument instead of dropping the function, and note the deviation from the spec.

- [ ] **Step 5b: Implement `popup_menu`**

Same shape as `popup`, calling `Popup::menu(ctx, ...)`. Confirm in `popup.rs`
that `menu` exists at 0.31.1 and what it returns; if it is not public in
0.31.1, report that instead of shipping a stub, and drop it from this task
and leave `popup_menu` as a TODO item.

- [ ] **Step 6: Register all seven in `#[pymodule]`**

- [ ] **Step 7: Extend the gallery `containers` page**

Add, in this order — the ordering is the point, so keep it visible in the page:

```python
side_panel_left(ctx, "nav", nav_contents)   # first
bottom_panel(ctx, "status", status_contents) # then
central_panel(ctx, main_contents)            # last
```

Add a `modal` gated on a `Bool` toggle inside the central panel, so the modal can be opened and closed. Extend the panel callbacks with `default_width` on the side panel to prove the option path.

- [ ] **Step 8: Push and read both verdicts**

```bash
git add src/lib.rs tests/expected_exports.py examples/gallery.py
git commit -m "Add side, top-bottom, modal and popup containers"
git push fork feature/egui-0.31-coverage
```

Expect `check` success with 130 exported names (123 + 7), clippy 0, and `screenshot` success.

- [ ] **Step 9: Verify layout, which is the whole risk in this task**

Open the rendered `containers.png`. Confirm the side panel is on the **left** edge, the bottom panel is on the **bottom** edge, and the central panel fills the remaining area — not merely that something drew. A panel added in the wrong order renders overlapping the centre and still counts as "not blank", which is exactly the failure Review Focus #1 names.

---

### Task 6: Migrate the remaining apps, and the README note

**Files:**
- Modify: `debug.py`, `examples/pages.py`, `examples/python-ide.py`, `examples/widget_gallery.py`, `examples/fileviwer.py`, `examples/gallery.py`
- Modify: `guides/centerhor.py`, `guides/centerver.py`, `guides/copytext.py`, `guides/copytextprog.py`, `guides/fonts.py`, `guides/log.py`, `guides/openurl.py`, `guides/themes.py`
- Modify: `README.rst` — one breaking-change note; `docs/development.rst`, `docs/gallery.rst` if they show frame shape
- Modify: `TODO.md` — tick the §3 items that shipped

**Interfaces:**
- Consumes: `central_panel` (Task 2) and the panels (Task 5).
- Produces: a branch where every app runs. No new exports.

- [ ] **Step 1: Migrate each app mechanically**

For every app, extract the body of `update_func` into a `main_contents()` function and wrap:

```python
def update_func(ctx):
  central_panel(ctx, main_contents)
```

Exceptions needing judgement:
- `examples/gallery.py` — each page draws its own widgets; wrap each page body in `central_panel(ctx, page_fn)` inside `run_page`, and make sure `main()` still passes `--out` through
- `examples/fileviwer.py` — inspect it first; if it already nests, keep the existing nesting and only add the outer `central_panel`
- `examples/widget_gallery.py` — the largest example; migrate it last

Never move widgets out of their existing callbacks. Only the top level of `update_func` gains the `central_panel` wrapper.

- [ ] **Step 2: Verify no app is left unwrapped**

```bash
cd /home/chetanamrao/Documents/projects/py_ui/pyegui
for f in $(git grep -l 'run_native' -- '*.py'); do
  grep -q 'central_panel' "$f" || echo "NOT MIGRATED: $f"
done
```

Expected: no output. Every app that runs an app must mention `central_panel`.

- [ ] **Step 3: Add the README note**

One short subsection near the existing usage docs, stating plainly: from 0.6.0 `update_func` no longer draws a central panel implicitly; call `central_panel(ctx, contents)` last; panels go before it. Show the two-line before/after so a reader can fix their app at a glance. No deprecation language — there is no shim.

- [ ] **Step 4: Bump the version**

`version = "0.6.0"` in `Cargo.toml`. This is a breaking change and a minor-version bump under a 0.x scheme is the honest signal. Do not touch the `=0.31.1` egui pins.

- [ ] **Step 5: Update `TODO.md`**

Tick the shipped §3 entries: `Window`, `SidePanel`, `TopBottomPanel`, `Modal`, `Popup`, and `CollapsingHeader`/`Frame`/`ScrollArea` only if this batch actually completed them. Leave `Area`, `Resize`, `Scene`, `MenuBar` unticked — they are still out of scope.

Update the "Current state" export count to the final number CI reports.

- [ ] **Step 6: Push and read both verdicts**

```bash
git add -A
git commit -m "Migrate the examples and guides to the explicit frame API"
git push fork feature/egui-0.31-coverage
```

Expect `check` success and `screenshot` success.

- [ ] **Step 7: Verify the migrated examples still render**

The screenshot job only renders `examples/gallery.py`, so it does **not** cover the migrated guides. Spot-check by dispatching a run, or by confirming the gallery's own pages — which are the most complex migrated app — render correctly. If a migrated guide cannot be verified visually, say so rather than claiming it works.

- [ ] **Step 8: Commit and confirm the tree is clean**

```bash
git status --short   # expected: empty
git log --oneline -8
```