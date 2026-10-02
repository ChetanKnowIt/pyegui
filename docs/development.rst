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

   # Filter on name == "check": regen-lockfile fires on the same push and
   # its run id sorts adjacent, so "take the latest run" watches the
   # wrong job.
   gh run list --repo ChetanKnowIT/pyegui --branch feature/<name> \
     --json databaseId,name,headSha -q '.[] | select(.name=="check")'
   gh run watch <run-id> --repo ChetanKnowIT/pyegui --exit-status
   gh run view <run-id> --repo ChetanKnowIT/pyegui --log-failed

regen-lockfile.yml -- lockfile updates without a local cargo
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Triggered on pushes to ``feature/**`` and via ``workflow_dispatch``.

The lockfile has to change whenever a dependency is added, removed or
re-pinned, and ``cargo update`` needs network access plus a toolchain.
This workflow runs it in CI and uploads the result as a
``Cargo.lock`` artifact:

.. code:: bash

   gh run download <run-id> --repo ChetanKnowIt/pyegui --dir ./lockout
   cp ./lockout/lockfile/Cargo.lock ./Cargo.lock
   git add Cargo.lock && git commit

It is a one-shot tool, not permanent infrastructure. Once the current
dependency work settles it should be deleted; reintroduce it if a future
re-pin needs it.

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