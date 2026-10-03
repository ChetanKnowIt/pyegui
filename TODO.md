# pyegui TODO

Feature coverage of the pinned egui release, and the order in which the gaps
get closed.

**Target: egui 0.31.1** (pinned exactly via `=0.31.1` in `Cargo.toml` and
asserted by `check.yml`). Every `egui::…` name below is checked against the
0.31.1 sources, not against a newer release.

Legend: `[x]` shipped · `[~]` partial (exists, but egui's builder options are
not reachable) · `[ ]` not implemented

## Current state

Version 0.5.1, built on egui 0.31.1. See `CHANGELOG.md`.

174 names exported (`src/lib.rs`): 18 classes, 156 functions and 46
`*_response` variants. The response variants return a `Response`; they exist
alongside the original boolean helpers, which are unchanged. egui 0.31.1
exposes 174 inherent methods on `Ui` and 61 on `Response`.

`tests/expected_exports.py` is the authoritative list and the CI export gate;
it is generated from, and asserted against, the `#[pymodule]` block.

Validation is CI-only — see `docs/development.rst`. Nothing is compiled
locally, so each batch is pushed to `fork` and the `check` run is the verdict.
The `examples` job additionally runs every app in the repository and every
runnable README snippet, and `release.yml` will only publish a GitHub Release
once both jobs pass on the tagged commit.

---

## 1. Response: interaction state — **highest value, do first**

egui returns a `Response` from every widget. pyegui used to collapse it to a
bool (`button_clicked`) or discard it (`heading`), so interaction state was
unreachable.

Shipped: a `Response` pyclass wrapping `egui::Response`, and a `*_response`
variant for every widget that already had a binding. The original functions
are now thin wrappers over them, so `if button_clicked("x"):` is unchanged.

- [x] `Response` pyclass wrapping `egui::Response`
- [x] `hovered`, `changed`, `clicked`, `clicked_by`, `double_clicked`,
      `triple_clicked`, `secondary_clicked`, `middle_clicked`, `long_touched`
- [x] `drag_started`, `dragged`, `drag_stopped`, `drag_delta`, `drag_motion`,
      and the `_by(PointerButton)` variants
- [x] `has_focus`, `gained_focus`, `lost_focus`, `request_focus`,
      `surrender_focus`
- [x] `on_hover_text`, `on_hover_ui`, `show_tooltip_text`, `show_tooltip_ui`,
      `on_disabled_hover_text`
- [x] `context_menu`, `clicked_elsewhere`, `enabled`, `highlighted`,
      `contains_pointer`, `interact_pointer_pos`, `hover_pos`,
      `is_pointer_button_down_on`, `mark_changed`, `is_tooltip_open`,
      `context_menu_opened`
- [x] `*_response` variants for existing widgets (46 of them)
- [ ] `interact`, `interact_opt`, `highlight`, `widget_info`, `union`,
      `scroll_to_me`, `output_event`, `labelled_by` — builder-style methods
      that consume or replace the `Response`
- [ ] `dnd_set_drag_payload`, `dnd_hover_payload`, `dnd_release_payload` —
      deferred: `Arc<dyn Any + Send + Sync>` has no clean Python mapping
- [ ] Drag-and-drop containers: `dnd_drag_source`, `dnd_drop_zone`

Two names in earlier revisions of this file do not exist in egui 0.31.1 and
were dropped rather than shipped as stubs:

  - `total_drag_delta`. `Response` has `drag_delta` (this frame) and
    `drag_motion`; there is no total.
  - `drag_released` / `drag_released_by`. Deprecated in 0.31.1 in favour of
    `drag_stopped` / `drag_stopped_by`, which are the ones exposed.

## 2. Missing widgets

- [x] `selectable_label` (selection without a companion value)
- [x] `radio` (selectable without a companion value)
- [x] `drag_angle`, `drag_angle_tau`
- [ ] `colored_label`
- [x] `color_edit_button_rgb` — and now every other colour space: `hsva`,
      `srgb`, `srgba`, `rgba_premultiplied`, `rgba_unmultiplied`,
      `srgba_premultiplied`, `srgba_unmultiplied`. Backed by the `RGBA`,
      `HSVA`, `Color32` and `SRGB` classes
- [~] `combo_box` — hand-rolled, not egui's `ComboBox`. Missing `width`,
      `wrap`, `icon`, `popup_style`, `from_id_salt`
- [~] `image` — missing `tint`, `size`, `fit_to_exact_size`, `rotate`, `uv`,
      `corner_radius`, `sense`, `alt_text`
- [~] `image_and_text_clicked` — missing `Button` options: `selected`,
      `min_size`, `sense`, `corner_radius`, `shortcut_text`
- [~] `progress` — missing text/format and `animate`
- [x] `close_menu`

## 3. Missing containers

egui 0.31.1 ships all of these.

Shipped: `CentralPanel` (`central_panel`) and `Window` (`window`, with a
`Bool`-driven `open` and egui's own builder options as keyword arguments).

**Two names in this section do not exist in egui 0.31.1 and were dropped rather
than shipped as stubs.** Verified against the sources at tag 0.31.1, not a
newer release:

  - `Popup`. There is no `egui::Popup` type in 0.31.1. `containers/popup.rs` at
    that tag defines only free functions -- `show_tooltip_text`,
    `was_tooltip_open_last_frame`, `seconds_since_last_tooltip`,
    `next_tooltip_id`, `tooltip_id` -- and `lib.rs` exports no `Popup`. The
    capability is reachable under its real names: `Response.on_hover_ui` and
    `Response.context_menu` ship, and `show_tooltip_text` is reachable
    through them.
  - `MenuBar` / submenus *as a container*. egui 0.31.1 has no `MenuBar` type;
    `MenuBar` landed in a later release. 0.31.1's menu surface is
    `Ui::menu_button`, `Ui::menu_image_button`, `Ui::menu_image_text_button`
    and `Ui::close_menu`, which open a popup menu at the widget's position.
    Submenus are the same call nested -- a `menu_button` inside a
    `menu_button` is a submenu. Those four ship; a wrapping `MenuBar`
    container has nothing to wrap until the pin moves.

The rest of this section is real work against real 0.31.1 APIs.

- [x] `Window` — secondary windows
- [x] `CentralPanel` — `central_panel(ctx, contents, **options)`
- [x] `SidePanel` (`left`/`right`) — `side_panel_left`, `side_panel_right`
- [x] `TopBottomPanel` (`top`/`bottom`) — `top_panel`, `bottom_panel`
- [x] `Area` — `area(ctx, id, contents, **options)`
- [x] `Modal` — `modal(ctx, id, contents, **options)`
- [x] `Resize` — `resize(contents, **options)`. Not a top-level container:
      egui's `Resize::show` takes a `&mut Ui`.
- [x] `Scene` — `scene(contents, view, **options)`, plus the `Rect` class it
      reads and writes. `zoom_range`, `max_inner_size`.
- [ ] `CollapsingHeader` — only the `collapsing` helper exists. Missing
      `default_open`, `show_background`, `icon`, `open` toggling,
      `CollapsingState` access
- [~] `Frame` — only `Frame::group`. Missing `fill`, `stroke`,
      `corner_radius`, `inner_margin`, and the presets (`popup`, `menu`,
      `window`, `canvas`, `central_panel`, `side_top_panel`)
- [~] `ScrollArea` — only vertical/horizontal. Missing `both`, `max_width`,
      `max_height`, `min_scrolled_width`, `min_scrolled_height`,
      `scroll_bar_visibility`, `id_source`
- [ ] `menu_button` / `menu_image_button` / `menu_image_text_button`

## 4. Layout, sizing and geometry

egui 0.31.1 exposes 174 public methods on `Ui`. These are the layout, sizing and
measurement ones pyegui did not reach.

Shipped: the twelve sizing calls, multi-column layout, the measurement queries,
and `push_id`.

- [x] `columns(num_columns, contents)` — one callable per column, each run in
      its own `Ui`. egui hands its callback a slice of `Ui`s; a Python callable
      cannot receive that, so a list of callables is the shape used here
- [x] `end_row`, `set_row_height`
- [x] Sizing: `set_width`, `set_height`, `set_min_width`, `set_max_width`,
      `set_min_height`, `set_max_height`, `set_min_size`, `set_max_size`,
      `set_width_range`, `set_height_range`, `shrink_width_to_current`,
      `shrink_height_to_current`
- [x] Measurement queries: `available_size`, `available_width`,
      `available_height`, `available_size_before_wrap`,
      `available_rect_before_wrap`, `cursor`, `min_rect`, `max_rect`,
      `min_size`, `pixels_per_point`, `next_widget_position`,
      `is_rect_visible`
- [x] `push_id` — gives a subtree its own id space, which is what makes a list
      of repeated widgets work at all
- [ ] `with_layout`, `wrap_mode`, `wrap_text`. `with_layout` takes a whole
      `egui::Layout`; the existing `Layout`/`LayoutType` classes are a
      different, narrower thing and do not cover it
- [ ] `unique_id`, `make_persistent_id`, `next_auto_id`, `auto_id_with`,
      `skip_ahead_auto_ids`, `id` — these return egui's `Id`, which has no
      Python equivalent and no useful Python-facing operation
- [ ] `scope_builder`, `new_child`, `child_ui`, `UiBuilder` support
- [ ] `interact`, `interact_opt` — take a `Rect`, an `Id` and a `Sense`
- [ ] `painter` access for custom painting (shapes, text layout)

The measurement functions return plain tuples for points and sizes, and a
`Rect` for rectangles. They are in egui points, not pixels — multiply by
`pixels_per_point()`.

## 5. Context API

`Context` exposes 10 methods; egui 0.31.1 has 148 public ones. Unchanged by
the coverage work so far — this is the next large gap after containers.

- [ ] Input state: `input`, `is_pointer_over_area`, `wants_keyboard_input`,
      `wants_pointer_input`, `is_using_pointer`, `pointer_hover_pos`,
      `pointer_latest_pos`, `pointer_interact_pos`, `multi_touch`
- [~] `Context.request_repaint()` ships, but not the `_after` / `_of` /
      `_after_secs` variants. `Context.close()` also ships, which is the one
      viewport command currently reachable.
- [ ] `memory`, `memory_mut`
- [ ] Styling: `style`, `style_mut`, `set_style`, `visuals`, `visuals_mut`,
      `spacing`, `spacing_mut`, `set_visuals`
- [ ] Animations: `animate_bool_with_time`, `animate_value_with_time`,
      `clear_animations`
- [~] Viewport: `Context.close()` ships (`send_viewport_cmd(Close)`), but
      `viewport`, the general `send_viewport_cmd`, `embed_viewports`,
      `set_embed_viewports` and `screen_rect` are unreachable
- [ ] Zoom and scale: `set_zoom_factor`, `zoom_factor`,
      `set_pixels_per_point`, `pixels_per_point`
- [ ] Textures and loaders: `load_texture`, `try_load_texture`,
      `try_load_bytes`, `try_load_image`, `forget_image`,
      `forget_all_images`
- [ ] Fonts beyond `set_font`: `fonts`, `fonts_mut`, `add_font`
- [ ] `set_cursor_icon`, `copy_image`
- [ ] Debug: `debug_on_hover`, `set_debug_on_hover`, `debug_text`

## 6. Builder options on existing widgets

Currently only `hint_text` (text edits), `max_width`/`max_height` (image) and
a positional label reach the builders.

- [ ] `Slider`: `logarithmic`, `step_by`, `binary`, `hexadecimal`, `octal`,
      `prefix`, `suffix`, `custom_formatter`, `custom_parser`, `vertical`,
      `clamping`, `text_color`, `handle_shape`, `fixed_decimals`,
      `min_decimals`, `max_decimals`, `show_value`, `trailing_fill`,
      `drag_value_speed`, `update_while_editing`, `largest_finite`,
      `smallest_positive`
- [ ] `DragValue`: `prefix`, `suffix`, `custom_formatter`, `custom_parser`,
      `binary`, `hexadecimal`, `octal`, `fixed_decimals`, `min_decimals`,
      `max_decimals`, `update_while_editing`, `clamp_existing_to_range`
- [ ] `TextEdit`: `password`, `desired_width`, `desired_rows`, `char_limit`,
      `lock_focus`, `font`, `interactive`, `cursor_at_end`,
      `background_color`, `margin`, `horizontal_align`, `vertical_align`,
      `clip_text`, `frame`, `return_key`, `load_state`, `store_state`
- [ ] `DatePickerButton`: `format`, `start_end_years`, `reverse_years`,
      `show_icon`, `combo_boxes`, `arrows`, `calendar`, `calendar_week`,
      `highlight_weekends`, `year_scroll_to`, `id_salt`
- [ ] `run_native` viewport kwargs — currently size, fullscreen, maximized,
      resizable, transparent, `icon_path`. Missing: `position`, `decorations`,
      `title`, `always_on_top`, `visible`, `app_id`, `monitor`, `window_level`,
      `taskbar`, `window_type`, `minimize_button`, `maximize_button`,
      `close_button`, `titlebar_shown`, `movable_by_background`,
      `mouse_passthrough`, `clamp_size_to_monitor_size`, `has_shadow`
- [ ] `eframe::NativeOptions`: renderer choice, `multisampling`,
      `depth_buffer`, `stencil_buffer`, `dithering`, `centered`,
      `persist_window`, `persistence_path`, `glow_options`, `wgpu_options`,
      `run_and_return`
- [ ] Persistence: `App::save` / `Storage` is unreachable, so no state is
      saved between runs

## 7. egui_extras

- [ ] `Table` / `TableBuilder` / `TableRow` / `Column` — data grids
- [ ] `StripBuilder`
- [ ] `Sizing` / `Size`
- [ ] `code_view_ui` with syntax highlighting — needs the `syntect` feature,
      which is not currently enabled
- [~] Image loaders: `all_loaders` is on, so png, jpeg, svg, gif and webp
      all decode. Missing: `svg_text` (selectable SVG text), `RetainedImage`

## 7. Benchmarks: what does the binding actually cost

**Status: measured at three widget counts (run 37093530513) and in the README.
The result is ~1.3x per widget, not "almost no overhead" — the README says so
plainly.**

Measured, 61 frames, both sides, all three counts in one run on one
GitHub-hosted runner (so the rows are comparable with each other):

| widgets/frame | pyegui us | egui us | ratio | extra us |
| ---: | ---: | ---: | ---: | ---: |
| 50 | 0.633 | 0.477 | 1.33x | 0.156 |
| 500 | 0.507 | 0.391 | 1.30x | 0.116 |
| 2000 | 0.516 | 0.389 | 1.33x | 0.127 |

`import pyegui` is 7.7 ms.

**The ratio does not scale with widget count, which is the finding.** The
expectation was that a fixed per-call toll amortises over more of egui's own
work, so the ratio should fall as widgets rise. It does not: 1.33x, 1.30x,
1.33x, which is inside run-to-run noise. A 2000-widget frame is 1.03 ms
through pyegui against 0.78 ms in Rust, or about 6% of egui's 16.7 ms budget
at 60 fps, so the binding is not what a profiler would point at. The point
where it would matter is the tens of thousands of widgets, not the thousands
measured here.

This supersedes the earlier single-count run (37061272948), which reported
1.84x at 500 widgets. That figure is not wrong as a measurement of that run;
it was quoted as though it described the binding, and it does not. Two
changes moved it: the widget count is now an input rather than a constant, and
the combine step refuses to compute a ratio unless both sides report the same
widget count and frame count — the two halves had drifted apart before, and a
ratio between different workloads is indistinguishable from a real one.

Remaining, all of them about making the measurement more trustworthy rather than
about finding a better number:

- [x] Re-run at a second widget count. Done at 50/500/2000, in one run so the
      ratios are comparable with each other. The count is `WIDGETS` on both
      sides, so the workflow no longer measures 500 whatever it is asked for.
- [x] Record the runner class, with the note that the ratios are the portable
      part and the absolute microseconds are not
- [ ] Measure a heavier widget (image, text edit, drag) — `label` is the
      cheapest path through the binding and may flatter it
- [ ] Establish the run-to-run spread. The three counts above differ by up to
      25% in absolute per-widget cost on one runner, so the flat ratio rests on
      a single sample per count; a second run of the same counts would say
      whether 1.30x and 1.33x are one number or two

The question worth answering is not "is Python fast" but "what does the binding
add on top of egui". Rendering speed is egui's and identical either way, so a
number describing it says nothing about pyegui. Measuring only pyegui would be
worse than measuring nothing, since it puts a figure next to the word
"overhead" without saying what it is overhead relative to.

- [x] `bench/src/main.rs` — egui baseline: N labels/frame, 61 frames, timed
      around building the frame
- [x] `bench/bench.py` — the same work through pyegui
- [x] `.github/workflows/benchmark.yml` — builds both, writes
      `bench/results/combined.json`, prints the comparison to the run summary
- [x] Both exclude eframe's compositing from the measurement
- [x] `min` reported alongside the mean: `min` is the cost of the call itself,
      the mean folds in whatever else the runner was doing
- [ ] Record results on the runner class used, since a GitHub runner is shared
      and slower than a laptop — the ratios are the portable part, the
      absolutes are not
- [x] `bench/hello_egui.rs` + `bench/hello_pyegui.py` — the same app in each
      language, for the ergonomics claim that timings cannot support
- [x] Both benchmark halves request repaints and draw a panel, so they measure
      the same work. They had drifted: the Rust side repainted, the Python side
      did not, and the labels sat outside any panel on the Python side. Both
      bugs were found by running the benchmark, not by reading it

Deliberately **not** claimed:

- [ ] **Time to launch.** Launch here is dominated by `dlopen` of an 8MB
      extension plus eframe's window creation, neither of which the binding
      meaningfully changes. A "time to launch" figure would mostly measure the
      window system. `import pyegui` is reported on its own instead, which is
      the part a binding is actually responsible for.
- [ ] **First frame folded into the steady-state average.** The first frame
      includes shader compilation, texture upload and font rasterisation, none
      of which recur. It is reported as a separate line.
- [ ] **A single blended "performance" score.** Overhead varies with widget
      count; a ratio measured at 500 labels/frame does not transfer to 50 or
      5,000.

**Ergonomics is the stronger claim and is not yet measured.** The API point is
that a complete egui app is expressible in Python without losing interaction
state, and that reading `Response` does not mean learning a second framework.
That is a claim about the examples, not about timing, and it should be
demonstrated by a side-by-side of one example in each language rather than
asserted. Same workload, same widgets, same number of lines — count them.

---

## Appendices

### Shipped on the 0.31.1 coverage branch

Everything in the 0.5.0 appendix below, plus:

Interaction state: the `Response` class (40 methods) and 45
`*_response` variants covering every widget that has a binding. The boolean
helpers are unchanged and now delegate to them.

Widgets added: `selectable_label`, `radio`, `drag_angle`, `drag_angle_tau`,
`close_menu`.

Colours: all seven non-RGB pickers — `color_edit_button_hsva`, `_srgb`,
`_srgba`, `_rgba_unmultiplied`, `_rgba_premultiplied`,
`_srgba_unmultiplied`, `_srgba_premultiplied` — plus the `RGBA`, `HSVA`,
`Color32` and `SRGB` classes.

### Shipped in 0.5.0

Text: `heading`, `label`, `monospace`, `small`, `strong`, `weak`, `code`,
`code_editor` · Input: `text_edit_singleline`, `text_edit_multiline` ·
Buttons/links: `button_clicked`, `small_button_clicked`, `link_clicked`,
`hyperlink`, `hyperlink_to`, `image_and_text_clicked` ·
Selection: `checkbox`, `toggle_value`, `radio_value`, `selectable_value`,
`combo_box` · Numbers: `slider_float`, `slider_int`, `drag_float`,
`drag_int`, `progress` · Colour/dates: `color_edit_button_rgb`,
`date_picker_button` · Images: `image` · Layout: `horizontal`,
`horizontal_centered`, `horizontal_top`, `horizontal_wrapped`, `vertical`,
`vertical_centered`, `vertical_centered_justified`, `centered_and_justified`,
`indent` · Scopes: `collapsing`, `group`, `scope`, `scroll_area_vertical`,
`scroll_area_horizontal`, `Layout`/`LayoutType`, `Group` · Ui state:
`disable`, `add_enabled`, `set_invisible`, `set_opacity`, `add_space`,
`separator` · App: `run_native`, `Context` · State holders: `Str`, `Bool`,
`Int`, `Float`, `RGB`, `Date`

### egui 0.31.1 `Ui` methods, complete reference

Derived from `egui-0.31.1/src/ui.rs`. Every method is classified by the
sections above or by the exclusions below, so nothing falls through the
cracks.

**Not planned:** allocation internals (`allocate_*`, `expand_to_include_*`),
paint/layer internals (`painter`, `painter_at`, `layer_id`, `opacity`,
`multiply_opacity`, `reset_style`, `set_clip_rect`, `shrink_clip_rect`,
`rect_contains_pointer`, `ui_contains_pointer`, `is_rect_visible`,
`is_sizing_pass`, `debug_paint_cursor`), bookkeeping
(`skip_ahead_auto_ids`, `make_persistent_id`, `text_style_height`,
`text_valign`), and deprecated 0.30-era names (`add_enabled_ui`,
`close_menu`, `set_sizing_pass`, `wrap_text`, `with_layer_id`,
`interact_bg`, `interact_with_hovered`, `child_ui_with_id_source`,
`allocate_new_ui`, `allocate_ui_at_rect`, `child_ui`, `data`, `data_mut`,
`input`, `input_mut`, `output`, `output_mut`, `memory`, `memory_mut`,
`fonts`, `new`, `put`, `with_visual_transform`).

**Deferred to §4/§5 as listed:** `add`, `add_sized`, `add_visible`, `auto_id_with`,
`available_*`, `ctx`, `cursor`, `id`, `interact*`, `is_enabled`, `is_visible`,
`layout`, `max_rect`, `min_rect`, `min_size`, `next_auto_id`,
`next_widget_position`, `pixels_per_point`, `push_id`, `scope_builder`,
`scope_dyn`, `new_child`, `scroll_*`, `set_height*`, `set_max_*`,
`set_min_*`, `set_row_height`, `set_style`, `set_width*`, `shrink_*`,
`spacing*`, `stack`, `style*`, `unique_id`, `visuals*`, `with_layout`,
`wrap_mode`, `columns`, `columns_const`, `end_row`.

### Upstream egui 0.36 upgrade

Deliberately **out of scope** on this branch. The pin stays at 0.31.1; see the
"Upgrade path to egui 0.36" section of `README.rst` for the ten blockers and
why the bump is a project rather than a version edit.