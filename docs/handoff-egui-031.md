## Current state

Branch `feature/egui-0.31-coverage`, 14 commits ahead of `main` (`8b11fd4`).
Latest `check` run: **green**, 121 exported names (17 classes, 63 functions,
41 response variants), clippy at 0 diagnostics.

Shipped since the roadmap was written:

| Commit | What |
| --- | --- |
| `0c6407e` | Batch D minus `colored_label`: `selectable_label`, `radio`, `drag_angle`, `drag_angle_tau`, `close_menu`. Export assertion widened to every name. |
| `dea277f` | Batch A+B: `Response` class + 34 `*_response` variants. |
| `babcd77` | Fix three 0.31.1 API mistakes in A+B. |
| `4d69116` | Batch C: seven non-RGB colour pickers + `RGBA`, `HSVA`, `Color32`, `SRGB`. Export gate moved to `tests/expected_exports.py` and gained the reverse check. |
| `7aca0fe` | Exclude the module's own name from the reverse check. |

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

**`regen-lockfile.yml` fires on every push to `feature/**`.** That is
deliberate for now: it uploads a `Cargo.lock` artifact after `cargo update`,
which is how lockfile changes get made without a local cargo. Delete the
workflow once dependency work settles.

**Clippy is advisory, not a gate.** `check.yml` runs clippy without
`-D warnings` and posts the diagnostics to the job summary instead of
failing, so it is a "no new lints" regression gate. The baseline is now **0
diagnostics** (it used to be ~19), so promoting it to `-- -D warnings` is
worth doing — it would make CI enforce the clean state rather than merely
report it.

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
- Tooltips and context menus need their own callback wrapper: egui types
  them `impl FnOnce(&mut Ui)`, returning `()`, but `run_nested_update_func`
  returns `PyResult<()>`. Those call sites use `run_nested_update_func_lossy`,
  which drops the error. The Python exception has already been displayed.
- `on_hover_text` / `on_hover_ui` consume the `Response` in egui. They take
  `&mut self` here and reassign from `self.inner.clone()...`, because
  `egui::Response` has no `Default` and cannot be moved out of a `&mut`.

## Verification loop

```bash
git push fork feature/egui-0.31-coverage
gh run list --repo ChetanKnowIT/pyegui --branch feature/egui-0.31-coverage \
  --json databaseId,name,headSha -q '.[] | select(.name=="check")'
gh run watch <run-id> --repo ChetanKnowIT/pyegui --exit-status
```

Filter on `name == "check"`: `regen-lockfile` fires on the same push and its
id sorts adjacent, so grabbing the latest run id watches the wrong job.

`check.yml` runs: `cargo check --locked --all-targets` → advisory clippy →
assert `cargo tree -p {egui,eframe,egui_extras}` all resolve to exactly
`0.31.1` → `maturin build` + `pip install` → the export gate.

**Every new public name must be added to `tests/expected_exports.py` in the
same change.** Otherwise the gate fails, which is the point.

Roughly 3 minutes per run. `make build`, `make build-manylinux`,
`make develop` and `make doc` all cannot run locally.

## Settled decisions

These three were open at `e0beb36` and were decided on 2026-10-02. Do not
re-open them without new information.

1. **Nesting: additive `_response` suffix.** New `button_response()` etc. sit
   alongside `button_clicked()`; nothing existing changes signature. Chosen
   over returning a falsy `Response` from the bool helpers, which would break
   `if button_clicked():` for current users.
2. **Drag-and-drop payloads are deferred.** Ship the `Response` class and the
   `_*_response` variants without DnD. `dnd_set_drag_payload` takes
   `Arc<dyn Any + Send + Sync>`, which does not map onto Python objects; that
   bridging is its own design problem. TODO §1 keeps the DnD items unchecked.
3. **Batch order, as executed.** D minus `colored_label` shipped first (five
   trivial functions, no new classes) to calibrate the loop; then A+B as one
   change, since A alone verifies little; then C. All three are done.
   `colored_label` is the only one outstanding, and it needs `RichText`.

**Next up, in order:** `colored_label` (needs a `RichText` wrapper) →
containers (§3: `Window`, `SidePanel`, `TopBottomPanel`, `Popup`, `Modal`,
`MenuBar` — the largest remaining gap) → the `Context` API (§5) → builder
options (§6).

## House rules for this repo

- Do not add local build steps. If it cannot run in CI, it does not run.
- Do not trust build output that did not come from a workflow run.
- Keep the lockfile authoritative; CI builds with `--locked`.
- Commit only once `check` is green.
- Update `TODO.md` and the README roadmap in the same change as the code.

