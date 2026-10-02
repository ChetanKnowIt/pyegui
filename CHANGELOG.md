# Changelog

All notable changes to pyegui are recorded here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project uses [semantic versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.1]

Released 2026-10-02. Built on egui 0.31.1.

```python
def update_func(ctx):
    central_panel(ctx, main_contents)
```

That is the whole breaking change: `run_native` used to open a `CentralPanel`
for you, and now it does not.

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

egui draws nothing until the frame asks for a container, so an app composes
its own. Draw side, top and bottom panels first and `central_panel` last, which
then takes the space that is left over.

- `central_panel(ctx, contents, **options)` — the central panel, mirroring
  `egui::CentralPanel`.
- `window(ctx, title, id, contents, **options)` — `egui::Window`, returning
  whether it is open, with a `Bool` for the `open` argument and egui's own
  builder options mapped to keyword arguments.
- `side_panel_left` / `side_panel_right` — `egui::SidePanel::left` / `::right`,
  with `resizable`, `show_separator_line`, `default_width`, `min_width`,
  `max_width` and `width_range`.
- `top_panel` / `bottom_panel` — `egui::TopBottomPanel::top` / `::bottom`, with
  the equivalent height options.
- Unknown container options raise `ValueError` naming every valid option,
  rather than being ignored.

`modal`, `popup`, `popup_menu`, `area`, `resize` and `scene` are not in this
release. See `TODO.md` for the remaining API surface.

**egui 0.31.1 widget coverage**

- `selectable_label`, `radio`, `drag_angle`, `drag_angle_tau`, `close_menu`.

**Response**

- `Response` class, and `*_response` variants for 41 widgets for callers who
  need interaction state. The existing boolean helpers are unchanged, so
  existing code keeps working.

**Colour**

- `RGBA`, `SRGB` and `HSVA` classes, alongside the existing `RGB`.
- Colour pickers in the spaces egui 0.31.1 offers, each with a `_response`
  variant: `color_edit_button_rgb`, `color_edit_button_rgba_premultiplied`,
  `color_edit_button_rgba_unmultiplied`, `color_edit_button_srgb`,
  `color_edit_button_hsva`, `color_edit_button_srgba_premultiplied`,
  `color_edit_button_srgba_unmultiplied`.

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