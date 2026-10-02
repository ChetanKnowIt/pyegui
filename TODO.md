# pyegui TODO

Feature coverage of the pinned egui release, and the order in which the gaps
get closed.

**Target: egui 0.31.1** (pinned exactly via `=0.31.1` in `Cargo.toml` and
asserted by `check.yml`). Every `egui::…` name below is checked against the
0.31.1 sources, not against a newer release.

Legend: `[x]` shipped · `[~]` partial (exists, but egui's builder options are
not reachable) · `[ ]` not implemented

## Current state

90 functions and 13 classes exported (`src/lib.rs`), 103 names in total. The
34 `*_response` variants are the ones that return a `Response`; they exist
alongside the original boolean helpers, which are unchanged. egui 0.31.1
exposes 174 inherent methods on `Ui` and 45 on `Response`.

Validation is CI-only — see `docs/development.rst`. Nothing is compiled
locally, so each batch is pushed to `fork` and the `check` run is the verdict.

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
- [x] `drag_started`, `dragged`, `drag_stopped`, `drag_delta`,
      `drag_released`, `drag_motion`, and the `_by(PointerButton)` variants
- [x] `has_focus`, `gained_focus`, `lost_focus`, `request_focus`,
      `surrender_focus`
- [x] `on_hover_text`, `on_hover_ui`, `show_tooltip_text`, `show_tooltip_ui`,
      `on_disabled_hover_text`
- [x] `context_menu`, `clicked_elsewhere`, `enabled`, `highlighted`,
      `contains_pointer`, `interact_pointer_pos`, `hover_pos`,
      `is_pointer_button_down_on`, `mark_changed`, `is_tooltip_open`,
      `context_menu_opened`
- [x] `*_response` variants for existing widgets (34 of them)
- [ ] `interact`, `interact_opt`, `highlight`, `widget_info`, `union`,
      `scroll_to_me`, `output_event`, `labelled_by` — builder-style methods
      that consume or replace the `Response`
- [ ] `dnd_set_drag_payload`, `dnd_hover_payload`, `dnd_release_payload` —
      deferred: `Arc<dyn Any + Send + Sync>` has no clean Python mapping
- [ ] Drag-and-drop containers: `dnd_drag_source`, `dnd_drop_zone`

Note: `total_drag_delta` is listed in earlier revisions of this file but does
not exist in egui 0.31.1 — `Response` has `drag_delta` and `drag_motion`, and
egui asserts `Send + Sync` on `Context` rather than exposing a total.

## 2. Missing widgets

- [x] `selectable_label` (selection without a companion value)
- [x] `radio` (selectable without a companion value)
- [x] `drag_angle`, `drag_angle_tau`
- [ ] `colored_label`
- [~] `color_edit_button_rgb` — only RGB. Missing `hsva`, `srgb`, `srgba`,
      `rgba_premultiplied`, `rgba_unmultiplied`, `srgba_premultiplied`,
      `srgba_unmultiplied`
- [~] `combo_box` — hand-rolled, not egui's `ComboBox`. Missing `width`,
      `wrap`, `icon`, `popup_style`, `from_id_salt`
- [~] `image` — missing `tint`, `size`, `fit_to_exact_size`, `rotate`, `uv`,
      `corner_radius`, `sense`, `alt_text`
- [~] `image_and_text_clicked` — missing `Button` options: `selected`,
      `min_size`, `sense`, `corner_radius`, `shortcut_text`
- [~] `progress` — missing text/format and `animate`
- [x] `close_menu`

## 3. Missing containers

egui 0.31.1 ships all of these; pyegui reaches only `CentralPanel` and
`Frame::group`.

- [ ] `Window` — secondary windows (39 builder options available)
- [ ] `SidePanel` (`left`/`right`)
- [ ] `TopBottomPanel` (`top`/`bottom`)
- [ ] `Area`
- [ ] `Popup`
- [ ] `Modal`
- [ ] `Resize`
- [ ] `Scene`
- [ ] `MenuBar` / submenus
- [~] `CollapsingHeader` — only the `collapsing` helper exists. Missing
      `default_open`, `show_background`, `icon`, `open` toggling,
      `CollapsingState` access
- [~] `Frame` — only `Frame::group`. Missing `fill`, `stroke`,
      `corner_radius`, `inner_margin`, and the presets (`popup`, `menu`,
      `window`, `canvas`, `central_panel`, `side_top_panel`)
- [~] `ScrollArea` — only vertical/horizontal. Missing `both`, `max_width`,
      `max_height`, `min_scrolled_width`, `min_scrolled_height`,
      `scroll_bar_visibility`, `id_source`

## 4. Layout, sizing and geometry

- [ ] `columns`, `columns_const`, `end_row`, `set_row_height` — no
      multi-column layout exists today
- [ ] Sizing: `set_width`, `set_height`, `set_min_width`, `set_max_width`,
      `set_min_height`, `set_max_height`, `set_min_size`, `set_max_size`,
      `set_width_range`, `set_height_range`, `shrink_width_to_current`,
      `shrink_height_to_current`
- [ ] Measurement queries: `available_size`, `available_width`,
      `available_height`, `available_rect_before_wrap`, `cursor`, `min_rect`,
      `max_rect`, `min_size`, `pixels_per_point`, `next_widget_position`
- [ ] `with_layout`, `wrap_mode`, `wrap_text`
- [ ] `push_id`, `auto_id_with`, `unique_id`, `make_persistent_id`,
      `next_auto_id`, `skip_ahead_auto_ids`, `id`
- [ ] `scope_builder`, `new_child`, `child_ui`, `UiBuilder` support
- [ ] `interact`, `interact_opt`
- [ ] `painter` access for custom painting (shapes, text layout)

## 5. Context API

`Context` exposes 8 methods; egui 0.31.1 has 151.

- [ ] Input state: `input`, `is_pointer_over_area`, `wants_keyboard_input`,
      `wants_pointer_input`, `is_using_pointer`, `pointer_hover_pos`,
      `pointer_latest_pos`, `pointer_interact_pos`, `multi_touch`
- [ ] `request_repaint` and the `_after` / `_of` variants
- [ ] `memory`, `memory_mut`
- [ ] Styling: `style`, `style_mut`, `set_style`, `visuals`, `visuals_mut`,
      `spacing`, `spacing_mut`, `set_visuals`
- [ ] Animations: `animate_bool_with_time`, `animate_value_with_time`,
      `clear_animations`
- [ ] Viewport: `viewport`, `send_viewport_cmd`, `embed_viewports`,
      `set_embed_viewports`, `screen_rect`
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

---

## Appendices

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