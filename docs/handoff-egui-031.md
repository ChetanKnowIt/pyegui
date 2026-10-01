# Handoff: egui 0.31.1 coverage branch

State as of commit `e0beb36` on `feature/egui-0.31-coverage`. Written so the
next session can resume without re-deriving anything.

## Where things stand

Branch `feature/egui-0.31-coverage` is cut from `main` (8b11fd4) and pushed
to the `fork` remote. Latest `check` run on `e0beb36`: **green**.

Five commits ahead of `main`:

| Commit | What |
| --- | --- |
| `dc4ba9c` | Pin egui to `=0.31.1`, add the README roadmap, add `check.yml` |
| `fcef561` | Regenerate `Cargo.lock` for the explicit `egui` dependency |
| `196d90a` | Document the CI-only build setup; make clippy advisory |
| `96e00e0` | Fix the pin-verification whitespace comparison |
| `e0beb36` | Rewrite `TODO.md` as a prioritised coverage roadmap |

Working tree is clean except untracked `uv.lock`, which is a stray uv
artifact from earlier exploration and is deliberately not committed.

## Non-obvious constraints

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

**Clippy is advisory, not a gate.** The pre-existing `lib.rs` carries ~19
lints, so `check.yml` runs clippy without `-D warnings` and posts the
diagnostics to the job summary instead of failing. It is currently a "no new
lints" regression gate. Promote to `-D warnings` when the baseline is clean.

## Verification loop

```bash
git push fork feature/egui-0.31-coverage
gh run list --repo ChetanKnowIT/pyegui --branch feature/egui-0.31-coverage
gh run watch <run-id> --repo ChetanKnowIT/pyegui --exit-status
```

`check.yml` runs: `cargo check --locked --all-targets` → advisory clippy →
assert `cargo tree -p {egui,eframe,egui_extras}` all resolve to exactly
`0.31.1` → `maturin build` + `pip install` → Python import asserting a
required-exports set. That last step exists because a `#[pyfunction]` that
is not registered in the `#[pymodule]` block compiles cleanly and only fails
at import.

**Every new public name must be added to the export assertion in
`check.yml` in the same change.** Otherwise CI stays green while the Python
API silently drifts from the Rust API.

Roughly 3 minutes per run. `make build`, `make build-manylinux`,
`make develop` and `make doc` all cannot run locally.

## What was built, in short

- `Cargo.toml` pins `eframe`, `egui` and `egui_extras` to `=0.31.1` with an
  explicit `egui` dependency, so the pin is real rather than transitive.
- `README.rst` gained a **Roadmap** section: what pyegui exposes today (51
  functions, 10 classes), what egui has that it does not, and the ten
  concrete blockers for the 0.36 upgrade. Plus a short **Contributing**
  section pointing at the CI setup.
- `TODO.md` was rewritten from a flat alphabetical checklist into seven
  prioritized sections (Response, widgets, containers, layout, Context,
  builder options, egui_extras). Its appendix classifies all 174 egui 0.31.1
  `Ui` methods so none are unaccounted for.
- `docs/development.rst` is new and wired into the Sphinx toctree.

## The open decision

Four candidate batches were scoped. Nothing has been implemented yet beyond
infrastructure.

```
A  Response core           1 new class, ~20 getters, 0 changes   inert alone
B  *_response variants    ~19 new functions, 0 changes          needs A
C  non-RGB color pickers   ~9 new names, 2 classes (RGBA, HSVA)  independent
D  simple missing widgets  6 new functions                      independent
   selectable_label, radio, drag_angle, drag_angle_tau,
   close_menu, colored_label  (colored_label needs Color32+RichText)
```

Dependency shape: A → B unlocks the rest of TODO §1. C and D are independent
of both.

Suggested sequencing: A+B as one coherent change (A alone verifies little),
then C and D in parallel. Three CI runs total.

Three questions were left unanswered and are worth settling before writing
Rust:

1. **Nesting.** Add `button_response()` alongside `button_clicked()`
   (doubles the widget names), or change existing functions to return a
   falsy `Response` object (cleaner long term, breaks `if button_clicked()`
   for anyone relying on the bool)? Recommendation: additive suffix, decide
   now before the name count doubles.
2. **Drag-and-drop payloads.** `egui`'s `dnd_set_drag_payload` takes
   `Arc<dyn Any + Send + Sync>`, which does not map cleanly onto Python
   objects. Recommendation: ship A+B without DnD and treat payload bridging
   as its own design problem, rather than burying it in "Response core".
3. **Batch 1 scope.** A+B, or start narrow with D minus `colored_label`
   (five trivial functions, no new classes) to calibrate the loop.

## House rules for this repo

- Do not add local build steps. If it cannot run in CI, it does not run.
- Do not trust build output that did not come from a workflow run.
- Keep the lockfile authoritative; CI builds with `--locked`.
- Commit only once `check` is green.
- Update `TODO.md` and the README roadmap in the same change as the code.