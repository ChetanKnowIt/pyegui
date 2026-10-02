Development and CI
===================

.. contents::
   :local:
   :depth: 2

Why there is no local build
---------------------------

pyegui is a Rust extension module (pyo3 + cdylib). Compiling it requires a
Rust toolchain matching egui's MSRV, plus a C toolchain for the native
dependencies that ``eframe`` pulls in.

This repository is developed **without a local Rust toolchain**. There is
deliberately no ``rustup``/``cargo`` on the developer machine, and the
``Makefile`` targets that shell out to ``docker`` or ``.venv/bin/maturin``
cannot run locally either.

That constraint is the reason CI is structured the way it is: **GitHub
Actions is the only place pyegui is ever compiled.** Every claim about
whether the crate builds comes from a workflow run, never from a local
invocation. If you cannot run it in CI, you cannot run it.

What this means in practice
~~~~~~~~~~~~~~~~~~~~~~~~~~~

- Do not trust ``cargo check`` output that did not come from a CI run.
- Do not add build steps that only a local toolchain could satisfy.
- Keep the lockfile authoritative; CI builds with ``--locked``.
- Report build status with a run link, not with a description.

Remotes
~~~~~~~

The upstream remote is read-only for the current account, so all CI-backed
work happens on a fork:

===================  ==================================================
Remote               URL
===================  ==================================================
``origin``           ``https://github.com/snappercayt/pyegui.git``
                     (read-only; upstream project)
``fork``             ``https://github.com/ChetanKnowIt/pyegui.git``
                     (push target; runs Actions)
===================  ==================================================

.. code:: bash

   git remote -v
   git push fork feature/<name>

Workflows
---------

check.yml -- the per-push gate
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Triggered on every push to ``main`` and ``feature/**``, and on every pull
request. This is the fast feedback loop (~2 min, Linux, Python 3.11) that
replaces local compilation.

Steps, in order:

1. ``actions/checkout@v4`` and ``actions/setup-python@v5`` at 3.11 (the
   floor from ``requires-python``).
2. ``dtolnay/rust-toolchain@stable`` installs Rust, with
   ``Swatinem/rust-cache@v2`` caching the registry and ``target/``.
3. ``cargo check --locked --all-targets`` -- **the real gate.** If the Rust
   side does not compile, nothing else matters.
4. ``cargo clippy --locked --all-targets`` -- currently **advisory**. It runs
   without ``-D warnings`` and posts a summary instead of failing the job,
   making it a "no new lints" regression gate. The baseline is now 0
   diagnostics (it used to be roughly 19), so promoting it to
   ``-- -D warnings`` is the obvious next step.
5. ``Verify pinned egui version`` -- runs ``cargo tree -p <crate>`` for
   ``egui``, ``eframe`` **and** ``egui_extras``, and fails if any does not
   resolve to exactly ``0.31.1``. This is what makes the ``=0.31.1`` pins
   in ``Cargo.toml`` enforceable rather than aspirational.
6. ``Build wheel and verify exports`` -- installs ``maturin``, runs
   ``maturin build --release --locked``, pip-installs the wheel, then runs
   ``tests/expected_exports.py`` against the imported module. This catches
   missing ``#[pymodule]`` registrations that ``cargo check`` alone will
   not: adding a ``#[pyfunction]`` without registering it compiles cleanly
   and fails only at import time. It builds a wheel rather than using
   ``maturin develop`` because CI has no activated virtualenv.

The export gate lives in ``tests/expected_exports.py``, not inline in the
workflow. It asserts in **both** directions:

- ``check()`` -- every declared name is exported by the module.
- ``check_no_unexpected()`` -- every name the module exports is declared.
  Without this, a widget could be deleted from the module and the list
  together and CI would stay green while the API quietly shrank.

Every new public function or class must be added to that file in the same
change. It is a real Python file rather than a literal embedded in the YAML
so that it can be linted, diffed and executed directly.

Concurrency is set per-ref with ``cancel-in-progress``, so rapid pushes to
a feature branch do not queue up redundant builds.

.. code:: bash

   # Only `check` fires on every push. `screenshot` is path-filtered and
   # `regen-lockfile` is workflow_dispatch-only, so this lists one run.
   gh run list --repo ChetanKnowIT/pyegui --branch feature/<name> \
     --json databaseId,name,conclusion -q '.[] | "\(.databaseId) \(.name) \(.conclusion)"'
   gh run watch <run-id> --repo ChetanKnowIT/pyegui --exit-status
   gh run view <run-id> --repo ChetanKnowIT/pyegui --log-failed

regen-lockfile.yml -- lockfile updates without a local cargo
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Triggered via ``workflow_dispatch`` only.

The lockfile has to change whenever a dependency is added, removed or
re-pinned, and ``cargo update`` needs network access plus a toolchain.
This workflow runs it in CI and uploads the result as a
``Cargo.lock`` artifact:

.. code:: bash

   gh run download <run-id> --repo ChetanKnowIt/pyegui --dir ./lockout
   cp ./lockout/lockfile/Cargo.lock ./Cargo.lock
   git add Cargo.lock && git commit

It is manual rather than automatic because it was firing on every push and
producing an unchanged lockfile each time. It is also not permanent: once the
current dependency work settles it should be deleted; reintroduce it if a
future re-pin needs it.

screenshot.yml -- gallery renders and README images
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Renders every page of ``examples/gallery.py`` and uploads one PNG per page.
The images are checked into ``docs/_static/`` and rendered on the
``docs/gallery.rst`` page. They deliberately do *not* go in the README: a
figure directive cannot satisfy both renderers, because GitHub resolves a
README's relative paths against the repository root while Sphinx resolves
them relative to ``docs/`` (index.rst does ``include ../README.rst``).
Referencing them from a page inside ``docs/`` makes the two agree, and keeps
each image next to the repo's other documentation images.

This is also a render smoke test. The export gate proves the module imports
and exposes the right names; this proves the widgets actually draw. A widget
that compiles, imports, and paints nothing fails here, which the export gate
cannot catch.

Triggered when ``examples/gallery.py``, ``src/lib.rs`` or this workflow file
changes, and manually via ``workflow_dispatch``. It is path-filtered because
the images are checked in: regenerating them for an unrelated docs change is
five minutes of runner for identical output.

Two things about running a GUI in CI are worth knowing if this ever breaks:

- eframe builds with the ``glow`` backend, which needs a real OpenGL context.
  A bare Xvfb opens a window and renders nothing, so Mesa's software
  rasteriser (``libgl1-mesa-dri``) is what actually draws the pixels.
- winit ``dlopen()``s ``libxkbcommon-x11.so`` and panics if it is absent, so
  the X client libraries have to be installed explicitly. An incomplete runner
  image fails at window creation, not at import.

Screenshots are taken from inside the gallery, after the final frame settles
and immediately before the window closes. Capturing the X root from the
workflow afterwards photographs an empty desktop instead.

CI.yml -- release pipeline (upstream, untouched)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

The pre-existing maturin-generated ``CI.yml`` builds the full wheel matrix
(Linux glibc and musl, Windows, macOS) plus the sdist, publishes to PyPI on
tag pushes, and deploys the Sphinx docs to GitHub Pages.

It is left exactly as upstream generated it. It is **not** the development
gate: it only runs on tags, pull requests and manual dispatch, and building
the whole wheel matrix is far too slow to iterate against. Reach for
``check.yml`` while working, and ``CI.yml`` when cutting a release.

Local-only targets
------------------

The ``Makefile`` targets remain but are not usable in this environment:

=========================  ==================================================
Target                     Why it does not run here
=========================  ==================================================
``make build``             Needs ``.venv/bin/maturin`` and a
                           ``x86_64-pc-windows-gnu`` cross toolchain.
``make build-manylinux``   Needs ``docker``.
``make develop``           Needs ``.venv/bin/maturin`` (absent from the venv).
``make doc``               Builds Sphinx docs against an installed ``pyegui``;
                           run in CI via ``build_docs`` instead.
=========================  ==================================================

Adding a feature
----------------

1. Create or use a ``feature/*`` branch.
2. Implement the change in ``src/lib.rs``, following the existing pattern:
   ``#[pyfunction] unsafe fn ...`` taking the current ``Ui`` from the UI
   stack, plus a matching ``m.add_function(wrap_pyfunction!(...))`` in the
   ``#[pymodule]`` block.
3. Add the new name to ``tests/expected_exports.py``.
4. Push to ``fork`` and read the actual ``check`` run result.
5. Update ``TODO.md`` and the README roadmap.
6. Commit only once ``check`` is green.

If the change adds or alters a widget, add it to
``examples/gallery.py`` so ``screenshot.yml`` renders it, download the
artifact, and commit the refreshed PNGs under ``docs/_static/``.

The examples job
----------------

``check.yml`` proves the Rust side compiles and that the module exposes the
right names. It never looks at Python syntax, and it never runs an app, so
an example can be broken while every check is green.

``.github/workflows/examples.yml`` closes that gap. It builds the wheel,
installs it, and runs every app under Xvfb/Mesa:

- every app in ``examples/`` and ``guides/`` that calls ``run_native``, plus
  ``debug.py``, discovered by ``tests/smoke.py --list`` rather than listed by
  hand, so a new example is covered without editing the job
- each one under a wall-clock bound, with ``ctx.request_repaint()`` and a
  capture before ``ctx.close()``, so a run cannot hang
- each capture checked for drawn pixels, by counting distinct colours

Apps run to completion individually, and every failure is reported before the
step fails. One broken example does not hide the state of the others.

``gallery.py`` is excluded: it takes arguments and closes its own windows, so
the screenshot job drives it instead.

A capture that cannot be read fails the run rather than passing quietly. If
ImageMagick is ever dropped from the job, the job says so.

What this job caught
~~~~~~~~~~~~~~~~~~~~~

It exists because an example rotted unnoticed. When it first ran, it found:

- ``examples/pages.py`` did not parse -- damage from a bulk migration of mine
  that two green ``check`` runs had passed straight over
- ``guides/fonts.py`` had never been runnable from a fresh clone; it read a
  ``.ttf`` that was never tracked in the repository

Neither was visible to ``check.yml``.
