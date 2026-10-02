# Containers for pyegui — design

**Date:** 2026-10-02
**Status:** awaiting review
**Scope:** §3 of `TODO.md` — the containers pyegui cannot reach today

## Problem

pyegui reaches only `CentralPanel` and `Frame::group`. egui 0.31.1 also ships
`Window`, `SidePanel`, `TopBottomPanel`, `Area`, `Popup`, `Modal`, `Resize`,
`Scene`, `MenuBar` and submenus. Without them a pyegui app is a single
window of widgets: there is no second window, no side panel, no modal, no
menu bar. This is the largest functional gap in the crate.

Containers are also the one gap that cannot be closed by adding
straightforward one-line bindings, because they are not `Ui`-shaped. Every
existing pyegui function resolves its target through a global stack of raw
`Ui` pointers:

```rust
static mut UI: Vec<*mut egui::Ui> = ...   // UI_STACK

unsafe fn heading(text: &str) -> PyResult<()> {
    let ui = current_ui(&UI)?;   // pops the stack top
    ...
}
```

This works because every egui container that pyegui already exposes
(`horizontal`, `collapsing`, `Group`, `ScrollArea`) hands the closure a
`&mut Ui` that stays alive for the closure's duration, so the pushed pointer
is valid while Python runs inside it.

`Window`, `SidePanel`, `TopBottomPanel`, `Modal` and `Popup` are built on
`egui::Context`, not `Ui`, and their `.show()` takes the context explicitly:

```rust
pub fn show<R>(self, ctx: &Context, add_contents: impl FnOnce(&mut Ui) -> R)
    -> Option<InnerResponse<Option<R>>>          // Window
pub fn show<R>(self, ctx: &Context, add_contents: impl FnOnce(&mut Ui) -> R)
    -> InnerResponse<R>                          // Panel
pub fn show<T>(self, ctx: &Context, content: impl FnOnce(&mut Ui) -> T)
    -> ModalResponse<T>                          // Modal
```

So two things must change: a container function needs access to the
`Context`, and it must push a `Ui` onto the stack for the duration of the
Python callback.

## What already exists (verified in `src/lib.rs`)

Three findings remove most of the anticipated difficulty.

**1. The `Context` pyclass owns a real handle.** It is not a borrowed
pointer:

```rust
#[pyclass]
struct Context(egui::Context);      // by value, not a reference
```

**2. The `Context` is already constructed and passed into `update_func`
every frame.** In `PyeguiApp::update`:

```rust
fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    let ctx_r = Context(ctx.clone());
    egui::CentralPanel::default().show(ctx, |ui| {
        ...
        self.update_func.call1((ctx_r,))   // Python already receives ctx
    });
}
```

`egui::Context` is a cheap clonable handle (refcounted internally), so
`Context(ctx.clone())` per frame is the existing, working mechanism. **No
new global state is required.** Container functions take the context as an
explicit first argument from Python, mirroring how widgets take an explicit
state object:

```python
def update_func(ctx):
    window(ctx, "Settings", "win_id", lambda: heading("inside the window"))
```

This is the same explicit-argument discipline the crate already uses
(`checkbox(data, "check me")`, `slider_float(value, min, max)`). It is
honest about where the context comes from and needs no thread-local
plumbing.

**3. `Bool { value: bool }` is an owned field, so `&mut self.value` yields a
valid `&'open mut bool`.** egui's `Window::open` is the awkward one:

```rust
pub fn open(mut self, open: &'open mut bool) -> Self
```

The `'open` lifetime is satisfied by borrowing a `Bool`'s `value` field for
the duration of the `.show()` call, which is exactly as long as the borrow
needs to live. This is safe and needs no unsafe pointer work. `Bool` is
already the crate's convention for "value egui mutates" (`checkbox`,
`radio_value`, `selectable_value` all take one).

## The blocking constraint: panel ordering forces a `run_native` change

egui's `panel.rs` module docs are unambiguous:

> The order in which you add panels matter! The first panel you add will
> always be the outermost, and the last you add will always be the innermost.
> **⚠ Always add any `CentralPanel` last.**

`SidePanel::show` takes `ctx: &Context` and claims screen area.
`CentralPanel::show` takes `&mut Ui` — it is a *child* of an existing `Ui`,
not a peer. So a side panel cannot be added after `CentralPanel`; it has to
come first, and `CentralPanel` fills whatever is left.

`run_native` currently does the opposite:

```rust
fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    let ctx_r = Context(ctx.clone());
    egui::CentralPanel::default().show(ctx, |ui| {      // <-- CentralPanel FIRST
        self.update_func.call1((ctx_r,))                // <-- Python runs inside it
    });
}
```

Anything a panel function drew from inside `update_func` would be added
*after* `CentralPanel`, which is the order egui explicitly warns against.

egui's own test harness shows the intended shape. `__run_test_ctx` calls the
user's body with **only** a `Context` — no panel:

```rust
pub fn __run_test_ctx(mut run_ui: impl FnMut(&Context)) {
    let ctx = Context::default();
    ctx.run(Default::default(), |ctx| { run_ui(ctx); });
}
```

and egui's documented `SidePanel` example puts the panel before
`CentralPanel` in the same `ctx.run` body.

### The change

`PyeguiApp::update` stops opening `CentralPanel` itself. Python is
responsible for the frame:

```rust
fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    let ctx_r = Context(ctx.clone());
    unsafe {
        egui::Context::run(ctx.clone(), |ctx| {
            // Python draws panels, then central_panel(ctx, contents) last.
            Python::with_gil(|py| {
                if let Err(err) = self.update_func.call1((ctx_r,)) { err.display(py); }
            });
        });
    }
}
```

A new `central_panel(ctx, contents)` pyfunction — mirroring the other
containers, and the explicit last step:

```python
def update_func(ctx):
    side_panel_left(ctx, "nav", nav_contents)      # panels first
    modal(ctx, "confirm", confirm_contents)
    central_panel(ctx, main_contents)              # must be last
```

### Who this breaks (measured, 2026-10-02)

Every pyegui app in the wild is an `update_func` that calls widgets and
never draws the central panel, because `run_native` has always done it. So
this is not a small blast radius:

| Group | Count | Shape |
|---|---|---|
| Tracked `.py` apps | 15 | `run_native(...)` + `update_func`, zero container calls |
| README snippets | 12 | every "Getting started" and usage example |
| In-repo examples | 6 | `hello_world`, `pages`, `python-ide`, `widget_gallery`, `fileviwer`, `gallery` |
| In-repo guides | 8 | `guides/*.py`, one per feature |
| `debug.py` at root | 1 | uses `Group`, so it exercises a container today |
| Published on PyPI | 0.5.0 | 48 downloads last month, 11 GitHub stars, 4 forks |

I checked every tracked `.py` for container calls and found **none**. So
every existing app — ours and anyone's — takes the breakage path if
`central_panel` is not defaulted on.

Three distinct shapes have to keep working:

1. **Plain widget code** — `hello_world.py`, `guides/*.py`. Draws only
   widgets, never a container.
2. **Nested Ui containers** — `debug.py` uses `Group()`; `pages.py` uses
   `Layout(Horizontal)`. These already push onto `UI_STACK`, and they must
   keep resolving their `Ui` after `CentralPanel` moves out of `run_native`.
   This is the case most likely to reveal a bug, because it depends on the
   stack rather than on the frame.
3. **The gallery** — `examples/gallery.py` drives eight pages through
   `run_page`, and CI depends on it. Its preflight already exits when
   pyegui is stale, so it must also work when pyegui is current and
   `central_panel` defaults on.

### Decision: explicit clean break

`central_panel` is **not** defaulted on. Python draws every panel,
`CentralPanel` last, exactly as an egui app does.

This is settled by the parent library's own examples, read at 0.31.1.
`examples/custom_keypad/src/main.rs` draws a `Window` and a sub-widget
with no `CentralPanel` at all:

```rust
fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    egui::Window::new("Custom Keypad").show(ctx, |ui| { ... });
    self.keypad.show(ctx);
}
```

and `examples/popups/src/main.rs` and `examples/serial_windows/src/main.rs`
each open `CentralPanel` themselves, first in `update`. In Rust
`CentralPanel` is an ordinary container the app draws; `update` gets only a
`&Context`; nothing is implicit.

So a default-on `CentralPanel` would preserve a pyegui-specific convenience
that does not exist in egui, and would be the very thing that makes the
ordering rule unenforceable — pyegui cannot tell whether Python meant to
call it last or simply forgot. Being a binding is only worth anything if the
two APIs teach the same mental model, so explicit wins over convenient.

The cost is real but small and concentrated: 48 PyPI downloads last month,
11 stars, 4 forks, and 15 tracked `.py` files all owned here. This is a
breaking change and ships as **0.6.0**, which is what the major bump is
for.

**No migration shim.** No deprecation period, no auto-close fallback. One
note in the README, because at this adoption level a shim would cost more
in complexity and confusion than it saves.

### Failing loudly, never blank

If `update_func` returns having drawn no panel, pyegui raises a `RuntimeError`
naming the fix, rather than rendering an empty window:

    no top-level panel was drawn this frame -- call
    central_panel(ctx, contents) last in update_func

A loud error at startup is recoverable; a silently blank window after an
upgrade is not. This inverts the argument I made earlier for default-on:
loudness is better served by failing than by silently guessing what the user
meant.

## Design

### Function shape

Each container is one function that builds the egui container, shows it, and
pushes the inner `Ui` for the callback — the same push/pop discipline
`run_nested_update_func` already implements:

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
) -> PyResult<()> {
    let ctx = &ctx.0;
    let mut builder = egui::Window::new(title).id(egui::Id::new(id));
    // ... apply options from the dict ...

    let result = if let Some(open) = open {
        builder = builder.open(&mut open.value);
        // Window::show returns Option<...>; a collapsed window yields None.
        builder.show(ctx, |ui| run_nested_update_func_lossy(ui, contents.clone())).is_some()
    } else {
        builder.show(ctx, |ui| run_nested_update_func_lossy(ui, contents.clone())).is_some()
    };
    Ok(())
}
```

The `Option` return of `Window::show` is collapsed to a `bool` at the
boundary — `None` means the window was collapsed or fully closed, which is
the single most useful thing to tell Python. This mirrors how the crate
already flattens egui's richer returns.

`Panel::show` and `Modal::show` return non-`Option` values, so those return
`None`; they are not useful from Python. `ModalResponse::should_close` is
returned as a `bool`.

### Functions to ship

Ordered so each step is independently verifiable, and `Window` lands first
because it exercises the whole mechanism.

| Function | egui call | Returns |
|---|---|---|
| `central_panel(ctx, contents)` | `CentralPanel::show` | `None` — must be called last |
| `window(ctx, title, id, contents, open=None, **opts)` | `Window::show` | `bool` (visible) |
| `side_panel_left(ctx, id, contents, **opts)` | `Panel::left().show` | `None` |
| `side_panel_right(ctx, id, contents, **opts)` | `Panel::right().show` | `None` |
| `top_panel(ctx, id, contents, **opts)` | `Panel::top().show` | `None` |
| `bottom_panel(ctx, id, contents, **opts)` | `Panel::bottom().show` | `None` |
| `modal(ctx, id, contents, **opts)` | `Modal::show` | `bool` (should_close) |
| `popup(ctx, contents, **opts)` | `Popup::from_context` | `None` |
| `popup_menu(ctx, contents)` | `Popup::menu` | `None` |

`Area`, `Resize`, `Scene` and `MenuBar` are deliberately **out of scope for
this batch**. `MenuBar` in particular needs submenu support and an
`on_bar` closure, which is a second mechanism; `Area` and `Resize` are
lower value than the eight above. They stay as TODO items.

### Builder options

Options are passed as a Python kwargs dict and applied to the egui builder
in Rust. Unknown option names raise `PyValueError` naming the offender —
a typo must not silently do nothing, which is the same lesson as the export
gate catching a silently-asserting-nothing manifest.

Ship a curated subset per container, not all 39 `Window` options. The
`Window` set to ship:

- Geometry: `default_size`, `default_pos`, `min_size`, `max_size`,
  `min_width`, `min_height`, `max_width`, `max_height`, `default_width`,
  `default_height`, `anchor`, `fixed_pos`, `fixed_size`
- Behaviour: `resizable`, `collapsible`, `title_bar`, `movable`,
  `scroll`, `vscroll`, `hscroll`, `auto_sized`, `enabled`, `fade_in`,
  `fade_out`, `interactable`, `order`, `constrain`

Numeric options take egui types that have no Python equivalent yet
(`Vec2`, `Pos2`, `Vec2b`, `Align2`). Rather than invent a position/size
type in this batch, these accept 2-tuples and are converted in Rust. A
`Vec2` pyclass belongs in §4 with the rest of the geometry work; if that
lands first, these should switch to it.

Panels take `default_width` / `default_height` / `resizable` /
`show_separator` as appropriate.

### Nesting and the stack

Container callbacks push onto the existing `UI_STACK`, so a container nests
inside a widget (`collapsing`, `Group`) and vice versa with no special
casing. The pointer is popped in `run_nested_update_func`'s existing
`match ui_stack.pop()` arm.

One invariant to preserve: the push happens inside egui's closure, so the
`Ui` is alive for exactly the callback's duration. Containers do **not**
retain the `Ui` after the callback returns, matching every other container
pyegui already exposes.

### Error handling

- Called outside `update_func` → the existing `UI_CALL_OUTSIDE_UPDATE_FUNC`
  error, unchanged
- Unknown builder option → `PyValueError` naming it
- Python exception inside `contents` → displayed via `err.display(py)`,
  as `run_nested_update_func` already does; the container still closes
  cleanly because the pop is unconditional
- `Window::show` returning `None` (collapsed) → `False`, not an error

## Testing

CI is the only place anything compiles (no local Rust toolchain), so the
verdict is always a `check` run.

1. **Export gate.** Add the eight names to `tests/expected_exports.py`. The
   bidirectional check then proves they are exported *and* that nothing else
   appeared.
2. **Render smoke test.** Add a `containers` page to
   `examples/gallery.py` drawing a `Window`, a left `SidePanel`, a
   `bottom_panel` and a `MenuBar`-less modal. `screenshot.yml` renders it
   and its blank-frame check fails if the widgets do not paint. This is the
   only test that would catch a container that builds but never draws — the
   failure mode that matters most here.
3. **Manual smoke script.** A script showing open/close via a `Bool`, a modal
   whose `should_close` returns `True`, and a container nested inside
   `collapsing`. Folded into `examples/gallery.py` as the `containers`
   page rather than a new file — one gallery, not two, so CI keeps a single
   thing to render.

Visual inspection of the rendered page before declaring the batch done.

## Risks

- **`Window::show`'s `Option` return** — the `.is_some()` collapse loses the
  inner `Response`. Accepted: `Response` from a container is not more useful
  than its visibility, and `_response` variants for containers can follow if
  wanted.
- **`'open` lifetime on `Window::open`** — verified sound against `Bool`'s
  owned field, but it is the most subtle part and the first thing to
  revisit if the borrow checker complains.
- **`Context::run` nesting.** The spec wraps Python in
  `egui::Context::run` so panels can be ordered freely. egui's
  `__run_test_ctx` does exactly this, but whether it is correct to nest it
  inside `eframe::App::update` (which is already called from within a
  `Context::run`) is the single assumption most likely to need correcting
  first. If nesting misbehaves, the fallback is to open a bare parent `Ui`
  via `ctx.run` without a panel, which preserves the same ordering freedom.
- **15 tracked `.py` apps and 12 README snippets stop working** until
  migrated to call `central_panel`. This is intended, but it means the
  branch is not usable until the migration lands. Sequenced deliberately:
  green `check` first, then migrate, then a final verification run that
  renders the migrated examples.

## Explicitly out of scope

`Area`, `Resize`, `Scene`, `MenuBar` and submenus; `CollapsingState`;
`Vec2`/`Pos2`/`Align2` pyclasses (§4); the `Window` builder options not
listed above; persistence.

## Success criteria

- The eight functions are exported and covered by the export gate
- A container drawn inside `collapsing`, and `collapsing` inside a
  container, both render — and land in the right place, not merely
  non-blank, since a wrong `Ui` still draws something
- `screenshot.yml` renders the new `containers` page and it is visually
  confirmed to show the containers
- A `Bool` drives `Window` open/close correctly across frames
- An `update_func` that draws no panel raises the named `RuntimeError`
  rather than rendering blank
- **All 15 tracked `.py` apps and 12 README snippets are migrated** to call
  `central_panel` and are verified to still render. The migration happens
  after the CI verdict is green, per the agreed sequencing.
- Existing behaviour is otherwise unchanged — the pre-existing export names
  and the boolean helpers still pass the gate

## Sequencing

Per the agreed plan: land the code change and get a green `check` verdict
**first**, then migrate the examples, guides and README. The verdict comes
from the Rust side, so migrating Python before it is known to compile would
be fixing code against a guess.
