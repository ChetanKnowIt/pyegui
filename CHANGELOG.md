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

**Frames, scroll areas, collapsing headers and menus**

egui 0.31.1 has no menu-bar container and no `Popup` type. Its menu surface is
`menu_button` and its two image variants plus `close_menu`, and a
`menu_button` inside a `menu_button` is a submenu. Both the capability and the
popup surface are shipped here under those real names.

- `frame(contents, **options)` — `egui::Frame`, built from scratch, with
  `fill`, `stroke`, `corner_radius` (and egui's alias `rounding`),
  `inner_margin`, `outer_margin` and `multiply_with_opacity`.
- `frame_group`, `frame_popup`, `frame_menu`, `frame_window`, `frame_canvas`,
  `frame_dark_canvas`, `frame_central_panel`, `frame_side_top_panel` — egui's
  eight presets. Each is an associated function in Rust, so each is a
  constructor here; all read the current style, so they look right in any
  theme.
- `scroll_area_vertical`, `scroll_area_horizontal` and the new
  `scroll_area_both`, all now accepting `max_width`, `max_height`,
  `min_scrolled_width`, `min_scrolled_height`, `scroll_bar_visibility`,
  `id_source`, `id_salt`, `auto_shrink`, `animated`, `drag_to_scroll`,
  `stick_to_right` and `stick_to_bottom`.
- `collapsing_response(heading, update_fun, open=None, **options)` — the
  header's `Response`, with `default_open`, `enabled`, `show_background`,
  `id_salt`, `id_source` and an optional `Bool` for `open`. `collapsing` remains
  and now passes its options through.
- `menu_button(text, contents)`, `menu_image_button(source, contents)` and
  `menu_image_text_button(source, text, contents)` — egui's popup menu. Images
  are URIs, as for `image`.

Margins and corner radii accept a single number or a 2- or 4-sequence;
colours accept either a `Color32` or an `(r, g, b, a)` tuple.

See `examples/menus_and_popups.py`.

**Scene**

`scene(contents, view, **options)` — a pan-and-zoom canvas, and the last of
egui 0.31.1's containers. The user drags to pan and scrolls to zoom; the
visible region is a `Rect` that egui reads and writes as they interact, so it
has to be the same object every frame — pass the same one, not a fresh one.

```python
view = Rect.zero()

def contents():
    heading("inside the scene")
    label("drag to pan, scroll to zoom")

def main():
    scene(contents, view, zoom_range=(0.1, 4.0))
```

`Rect` is a new class mirroring `egui::Rect`, which is two corners rather than
a position and a size: `Rect(min, size)`, `Rect.from_corners(min, max)` and
`Rect.zero()`, with `min`, `max`, `size`, `width`, `height`, `center`,
`is_finite()` and a `repr`. `Rect.zero()` means "no view yet", which is what
egui resets from — it fits the contents on the first frame.

`zoom_range` defaults to egui's `(0.0, 1.0)`, which allows zooming out
arbitrarily but not in past 1:1. Pass something like `(0.0, float("inf"))` to
allow zooming in; text goes blurry past 1:1 (egui issue 4813).

See `examples/scene_canvas.py`.

**Layout, sizing and measurement**

- `columns(num_columns, contents)` — multi-column layout, mirroring
  `Ui::columns`. egui hands its callback a slice of `Ui`s, one per column; a
  Python callable cannot receive a slice, so this takes a **list of callables**,
  one per column, each run in its own `Ui`. That is what makes columns work:
  each column is its own `Ui` with its own cursor.
  `end_row()` closes a row early and `set_row_height()` gives a row a uniform
  height.
- Sizing: `set_width`, `set_height`, `set_min_width`, `set_max_width`,
  `set_min_height`, `set_max_height`, `set_min_size`, `set_max_size`,
  `set_width_range`, `set_height_range`, `shrink_width_to_current`,
  `shrink_height_to_current`. Each takes one value and returns nothing, because
  that is egui's shape — they set the size of the current `Ui`.
- Measurement: `available_size`, `available_width`, `available_height`,
  `available_size_before_wrap`, `available_rect_before_wrap`, `cursor`,
  `min_rect`, `max_rect`, `min_size`, `pixels_per_point`,
  `next_widget_position`, `is_rect_visible`. Points and sizes come back as
  2-tuples, rectangles as a `Rect`. All are in egui **points**, not pixels.
- `push_id(salt, contents)` — egui derives widget ids from position in the `Ui`
  tree, so identical widgets in a loop share one id and their state leaks
  between iterations. `push_id` gives a subtree its own id space, which is what
  makes a list of repeated inputs work at all.

```python
values = [Str("") for _ in range(3)]

for i, value in enumerate(values):
    push_id(i, lambda i=i, value=value: text_edit_singleline(value))
```

See `examples/layout_and_sizing.py`.

`modal` returns `True` when the backdrop is clicked, which is the signal to
dismiss it -- egui's `ModalResponse` has no `should_close`, and its
`is_top_modal` field answers "am I the topmost modal", which a lone modal
satisfies every frame. `resize` is not a top-level container: egui's
`Resize::show` takes a `&mut Ui`, so it must be called from inside another one.

A benchmark comparing pyegui with plain egui ships in `bench/`, run by the manual `benchmark` workflow. It measures both sides at 50, 500 and 2000 widgets per frame over 5 trials in a single run, so the ratios are comparable with each other. On the reference run a widget call through pyegui costs about 0.12 us more than calling egui from Rust — a median 1.33x, 1.33x and 1.32x, flat across that range — which is 0.97 ms for a 2000-widget frame against egui's 16.7 ms budget at 60 fps. Every one of the 15 trials fell between 1.31x and 1.34x. The widget count is an input, not a constant: the workflow previously exported `WIDGETS` and then ignored it, measuring 500 whatever it was asked for. `bench/hello_egui.rs` and `bench/hello_pyegui.py` are the same app in each language, for the ergonomics claim that timings cannot support.

`popup`, `popup_menu` and `scene` are not in this release. See `TODO.md`.

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