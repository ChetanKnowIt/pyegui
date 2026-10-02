# Changelog

All notable changes to pyegui are recorded here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project uses [semantic versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.1]

Unreleased. Built on egui 0.31.1.

### Breaking

**Frame composition is now explicit.** `run_native` no longer opens a
`CentralPanel` around your `update_func`. egui draws nothing until the frame is
asked for a container, so an app now says which panels it wants and in what
order:

```python
from pyegui import *

def main_contents():
    heading("Hello, World!")

def update_func(ctx):
    central_panel(ctx, main_contents)

if __name__ == "__main__":
    run_native("Example app", update_func)
```

This matches egui 0.31.1 and eframe directly, rather than pyegui choosing a
panel for you. The two `central_panel` / `window` entry points take a
`Context`, which is the same value `update_func` already receives.

There is no compatibility shim and no deprecation path. Existing apps keep
working once they wrap their drawing code in `central_panel`; nothing else about
them changes.

Panel ordering follows egui's rule: draw side, top and bottom panels first, and
`central_panel` last so it takes the remaining space.

### Added

**Containers**

- `central_panel(ctx, contents, **options)` — the central panel, mirroring
  `egui::CentralPanel`.
- `window(ctx, title, contents, **options)` — `egui::Window`, returning whether
  it is open, with a `Bool` for the `open` argument and egui's own builder
  options mapped to keyword arguments.
- Unknown container options raise `ValueError` naming every valid option,
  rather than being ignored.

Side, top and bottom panels, `modal`, `popup` and `popup_menu` are not in this
release. See `TODO.md` for the remaining API surface.

**egui 0.31.1 widget coverage**

- `selectable_label`, `radio`, `drag_angle`, `drag_angle_tau`, `close_menu`.

**Response**

- `Response` class with its 28 accessors, and 34 `*_response` variants of
  existing widgets for callers who need interaction state.

**Colour**

- `RGBA`, `Hsva`, `Hsl`, `Oklch` classes.
- Seven non-RGB colour pickers with response forms: `color_edit_hsva`,
  `color_edit_hsva_response`, `color_edit_hsl`, `color_edit_hsl_response`,
  `color_edit_oklch`, `color_edit_oklch_response`, `color_edit_srgba`,
  `color_edit_srgba_response`.

**Context**

- `Context.close()` — ends the frame loop deterministically, so headless tests
  and screenshots need no timeout.
- `Context.request_repaint()` — ask egui for another frame, for animations and
  capture.

### Changed

- Every example, guide and the README snippets use the explicit frame API.
- `docs/gallery.rst` and the README carry screenshots of all eight gallery
  pages, rendered in CI.
- `guides/fonts.py` reports which font path it looked for instead of raising
  `FileNotFoundError`. It now honours `FONT_PATH` from the environment; the
  font it originally named was never tracked in this repository, so the example
  could never run from a fresh clone.

### Fixed

- The gallery fails with an actionable message when built against a stale
  pyegui, rather than a `NameError` partway through rendering.

### Internal

- `check` compiles with Clippy and verifies that `egui`, `eframe` and
  `egui_extras` all resolve to exactly 0.31.1.
- A new `examples` job runs every app in the repository under Xvfb, requires
  each to open a window, draw, and exit, and runs every runnable README
  snippet. A capture is checked for drawn pixels rather than file size, so a
  sparse window is not mistaken for a blank one.
- A `release` workflow publishes a GitHub Release only after `check` and
  `examples` both pass on the tagged commit.