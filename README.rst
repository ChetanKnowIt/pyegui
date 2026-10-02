pyegui
======

**pyegui** is a native extension for Python that provides bindings for
Rust immediate mode GUI library
`egui <https://github.com/emilk/egui>`__.

Example
-------

.. code:: python

   from pyegui import *

   name = Str("Van")
   age = Int(24)

   def update_func(ctx):
     heading("My egui Application")
     text_edit_singleline(name, hint_text="Your name")
     slider_int(age, 0, 150, "age")

     if button_clicked("Increment"):
       age.value += 1

     heading(f"Hello '{name.value}', age {age.value}")
     image("file://image.png", max_width=350, max_height=250)

   run_native("My pyegui Application", update_func)

|example 1| |example 2|

Features
--------

**pyegui** tries to be as close as possible to the original egui API,
but with the focus on simplicity and usability. Callbacks were removed
where possible to accomplish more smooth experience in Python.

- Light and Dark themes(defaults to the system's)
- Built-in latin and cyrillic alphabets. You can load any font you want
  with ``ctx.set_font`` function
- Images(png, jpeg, svg, gif, webp, and anything the ``image`` crate
  decodes)
- Date picker
- RGB color picker
- Text fields, radio buttons, buttons, code, progress bar etc.
- Scroll areas, collapsing sections, groups and scopes
- No dependencies which destroy you project when you distribute it. Just
  pure giant Rust binary

The API surface is **121 names** today: 17 classes, 63 functions and 41
``*_response`` variants. See `Roadmap`_ for what egui can do that pyegui
cannot do (yet). The authoritative list is `tests/expected_exports.py
<https://github.com/ChetanKnowIT/pyegui/blob/main/tests/expected_exports.py>`__,
which CI asserts against the built module.

Full list of implemented features is available
`here <https://github.com/GachiLord/pyegui/blob/main/TODO.md>`__

Roadmap
-------

Upstream egui versions
~~~~~~~~~~~~~~~~~~~~~~

pyegui wraps egui through Rust crates, so its feature set is bounded by
the egui release it is pinned to.

+--------------+------------+--------------+-----------------------+
| pyegui       | egui       | eframe       | Notes                 |
+==============+============+==============+=======================+
| 0.5.0        | 0.31.1     | 0.31.1       | current release       |
+--------------+------------+--------------+-----------------------+
| —            | 0.32.x     | 0.32.x       | not adopted yet       |
+--------------+------------+--------------+-----------------------+
| —            | 0.33.x     | 0.33.x       | not adopted yet       |
+--------------+------------+--------------+-----------------------+
| —            | 0.34.x     | 0.34.x       | not adopted yet       |
+--------------+------------+--------------+-----------------------+
| —            | 0.35.x     | 0.35.x       | not adopted yet       |
+--------------+------------+--------------+-----------------------+
| planned      | 0.36.2     | 0.36.2       | current egui release  |
+--------------+------------+--------------+-----------------------+

pyegui is five egui minor releases behind. egui 0.36 requires Rust 1.95
(0.31 required 1.81), so the upgrade also means a newer toolchain.

Available now
~~~~~~~~~~~~~

- Text: ``heading``, ``label``, ``monospace``, ``small``, ``strong``,
  ``weak``, ``code``, ``code_editor``
- Input: ``text_edit_singleline``, ``text_edit_multiline``
- Buttons and links: ``button_clicked``, ``small_button_clicked``,
  ``link_clicked``, ``hyperlink``, ``hyperlink_to``,
  ``image_and_text_clicked``
- Selection: ``checkbox``, ``toggle_value``, ``radio_value``, ``radio``,
  ``selectable_value``, ``selectable_label``, ``combo_box``
- Numbers: ``slider_float``, ``slider_int``, ``drag_float``, ``drag_int``,
  ``drag_angle``, ``drag_angle_tau``, ``progress``
- Colour: ``color_edit_button_rgb``, ``_srgb``, ``_srgba``, ``_hsva``,
  ``_rgba_unmultiplied``, ``_rgba_premultiplied``,
  ``_srgba_unmultiplied``, ``_srgba_premultiplied`` — every colour space
  egui 0.31.1 offers
- Dates: ``date_picker_button``
- Images: ``image`` (``max_width`` / ``max_height``)
- Layout: ``horizontal``, ``horizontal_centered``, ``horizontal_top``,
  ``horizontal_wrapped``, ``vertical``, ``vertical_centered``,
  ``vertical_centered_justified``, ``centered_and_justified``, ``indent``
- Scopes: ``collapsing``, ``group``, ``scope``, ``scroll_area_vertical``,
  ``scroll_area_horizontal``, ``Layout`` / ``LayoutType``, ``Group``
- Ui state: ``disable``, ``add_enabled``, ``set_invisible``,
  ``set_opacity``, ``add_space``, ``separator``, ``close_menu``
- App: ``run_native`` with viewport kwargs, ``Context`` (theme, fonts,
  ``open_url``, ``copy_text``)
- State holders: ``Str``, ``Bool``, ``Int``, ``Float``, ``Date``
- Colours: ``RGB``, ``RGBA``, ``HSVA``, ``Color32``, ``SRGB``

Interaction state
~~~~~~~~~~~~~~~~~

egui returns a ``Response`` from every widget, carrying hover, click, drag,
focus and rect information. pyegui reaches it through the ``*_response``
variants:

.. code-block:: python

   from pyegui import *

   name = Str("")
   enabled = Bool(True)

   def update_func(ctx):
       response = button_response("save")
       if response.clicked:
           print("saved")
       if response.hovered:
           response.on_hover_text("ctrl+s saves the file")

       if text_edit_singleline_response(name, hint_text="name").changed:
           print("name is now", name.value)

       if checkbox_response(enabled, "enabled").changed:
           print("toggled to", enabled.value)

The existing boolean helpers (``button_clicked`` and friends) are unchanged,
so existing code keeps working. ``Response`` also carries ``clicked_by``,
``drag_delta``, ``has_focus``, ``request_focus``, ``context_menu``,
``on_hover_ui`` and the rest of the 0.31.1 surface. A ``Response`` describes
one frame — read it in the same frame the widget was shown.

Not available yet — planned
~~~~~~~~~~~~~~~~~~~~~~~~~~~

Ordered roughly by value per unit of work. "egui" names the upstream
API this would wrap.

**Missing containers — the largest remaining gap**

- ``Window`` — no secondary windows (39 builder options upstream)
- ``SidePanel`` / ``TopBottomPanel`` — only ``CentralPanel`` is ever used
- ``Area``, ``Popup``, ``Modal``, ``Resize``, ``Scene``
- ``MenuBar`` / ``menu_button`` / submenus — no menus at all
- ``Frame`` with custom fill / stroke / corner radius, and the presets
  (``popup``, ``menu``, ``window``, ``canvas``, ``central_panel``,
  ``side_top_panel``) — only ``Frame::group`` is reachable
- ``CollapsingHeader`` builder options: ``default_open``,
  ``show_background``, ``icon``, open-state toggling
- ``ScrollArea``: ``both``, ``max_width`` / ``max_height``,
  ``min_scrolled_width`` / ``min_scrolled_height``,
  ``scroll_bar_visibility``, ``id_source``

**Missing layout and sizing**

- ``Grid``, ``columns`` / ``columns_const``, ``end_row``,
  ``set_row_height`` — no multi-column layout at all today
- Sizing: ``set_width``, ``set_height``, ``set_min_size``, ``set_max_size``,
  ``set_width_range``, ``shrink_width_to_current`` …
- Measurement: ``available_size``, ``available_width``, ``cursor``,
  ``min_rect``, ``max_rect``, ``pixels_per_point``,
  ``next_widget_position``
- ``with_layout``, ``wrap_mode``, ``push_id``, ``unique_id``
- ``UiBuilder`` / ``scope_builder`` / ``new_child``
- ``Painter`` access — no custom painting, shapes or text layout

**Missing Context API**

egui's ``Context`` exposes 151 methods; pyegui reaches 8.

- Input state: ``input``, ``is_pointer_over_area``,
  ``wants_keyboard_input``, ``wants_pointer_input``,
  ``pointer_hover_pos``, modifiers — you cannot read the keyboard or mouse
  directly today
- ``request_repaint`` (and the ``_after`` / ``_of`` variants)
- ``memory`` / ``memory_mut``
- ``style``, ``visuals``, ``spacing``, ``set_style``, ``set_visuals`` —
  no way to customise widget appearance
- Animations: ``animate_bool_with_time``, ``animate_value_with_time``
- ``viewport`` commands, ``set_zoom_factor``, ``set_pixels_per_point``
- ``load_texture`` / ``try_load_bytes`` / custom image loaders
- ``set_cursor_icon``, debug hooks

**Missing widgets**

- ``colored_label`` — needs ``Color32`` and ``RichText`` wrappers
- ``ComboBox`` as a widget (only pyegui's hand-rolled ``combo_box``
  exists), including ``width``, ``wrap``, ``icon``, ``popup_style``,
  ``from_id_salt``
- ``Table`` / ``TableBuilder`` from ``egui_extras`` — no data grids
- ``StripBuilder`` and ``Sizing`` from ``egui_extras``
- Syntax-highlighted code view (the ``egui_extras`` ``syntect`` feature
  is not enabled)
- ``svg_text`` (selectable SVG source), ``RetainedImage``
- Drag and drop: ``dnd_drag_source`` / ``dnd_drop_zone``. The payload
  methods on ``Response`` are also deferred: ``dnd_set_drag_payload``
  takes ``Arc<dyn Any + Send + Sync>``, which has no clean Python
  mapping.

**Missing builder options on existing widgets**

- ``Slider``: ``logarithmic``, ``step_by``, ``binary`` / ``hexadecimal`` /
  ``octal``, ``prefix`` / ``suffix``, ``custom_formatter`` /
  ``custom_parser``, ``vertical``, ``clamping``, ``text_color``,
  ``handle_shape``, ``fixed_decimals``, ``show_value``, ``trailing_fill``
- ``DragValue``: ``prefix`` / ``suffix``, formatters, ``binary``,
  ``fixed_decimals``, ``clamp_existing_to_range``
- ``TextEdit``: ``password``, ``desired_width`` / ``desired_rows``,
  ``char_limit``, ``lock_focus``, ``font``, ``interactive``,
  ``cursor_at_end``, ``background_color``, ``margin``,
  ``horizontal_align`` / ``vertical_align``, ``frame``, ``return_key``
- ``Button``: ``selected``, ``min_size``, ``atoms``, ``shortcut_text``,
  ``wrap``
- ``Image``: ``tint``, ``size``, ``fit_to_exact_size``, ``rotate``, ``uv``,
  ``corner_radius``, ``sense``, ``alt_text``
- ``ProgressBar``: text, ``animate``
- ``DatePickerButton``: ``format``, ``start_end_years``, ``show_icon``,
  ``combo_boxes``, ``calendar_week``, ``highlight_weekends``
- ``run_native`` viewport kwargs: only size, fullscreen, maximized,
  resizable, transparent and ``icon_path`` are forwarded; position,
  decorations, window level, app id, monitor and always-on-top are not
- ``eframe`` ``NativeOptions``: renderer choice, ``multisampling``,
  ``depth_buffer``, ``persistence_path``, ``dithering``, ``centered``,
  and ``App::save`` (state persistence) are all unreachable

Upgrade path to egui 0.36
~~~~~~~~~~~~~~~~~~~~~~~~~~

These are the concrete blockers found while comparing ``src/lib.rs``
against egui 0.36.2. They are why the version bump is a project, not a
one-line ``Cargo.toml`` edit.

1. **MSRV**: egui/eframe 0.36 require Rust 1.95 (0.31 needed 1.81).
2. **``eframe::App`` trait split (0.34)**: ``App::update`` was replaced by
   ``fn ui(&mut self, ui: &mut Ui, frame: &mut Frame)`` plus ``fn logic``.
   ``PyeguiApp`` implements ``update``, so it must be rewritten around the
   ``&mut Ui`` that eframe now hands it.
3. **``Context::run`` → ``Context::run_ui`` (0.34)** and ``Ui: Deref<
   Target = Context>``. The global ``UI`` pointer stack in ``lib.rs`` can
   stay, but it needs to key off the passed-in ``Ui`` rather than a
   stashed pointer.
4. **Atoms (0.32)**: ``Button``, ``Checkbox``, ``RadioButton`` and
   ``selectable_value`` take ``impl IntoAtoms``. String-based calls still
   compile, but any wrapper meant to accept image+text needs porting.
5. **Popup rewrite (0.32)**: ``Popup``, ``PopupAnchor``,
   ``PopupCloseBehavior``. Existing popup-ish code (``combo_box``,
   ``date_picker_button``) needs rechecking.
6. **Date type change**: ``egui_extras`` 0.36 datepicker uses
   ``jiff::civil::Date``; 0.31 used ``chrono::NaiveDate``. The pyegui
   ``Date`` class wraps ``NaiveDate``, so it must move to ``jiff`` (or
   keep ``chrono`` and convert).
7. **Font rendering (0.34)**: ``ab_glyph`` → ``skrifa`` + ``vello_cpu``,
   plus a font-variations API. ``Context.set_font`` keeps working but
   gains options (families, variations).
8. **Colour spaces**: the full ``color_edit_button_*`` family is expected
   rather than RGB-only.
9. **MSRV-adjacent dependency bumps**: pyo3 0.24 is fine, but
   ``image``, ``log`` and friends move with the egui release train.
10. **New upstream capabilities worth exposing once unblocked**:
    ``egui::Plugin`` (0.33), ``Ui`` classes via ``UiBuilder`` (0.35),
    the inspection protocol and ``egui_mcp`` (0.35), ``BoxedWidget``
    (0.36), and window-chrome theme syncing (0.36).

Deliberately not planned
~~~~~~~~~~~~~~~~~~~~~~~~

- Rendering backends other than the eframe default. pyegui does not
  expose renderer selection.
- Custom Rust-side widgets defined by the user. pyegui targets a pure
  Python surface.
- Web/wasm targets, Android, and mobile input tuning.

Contributing
------------

pyegui is a Rust extension module, and this repository is developed
**without a local Rust toolchain** -- no ``rustup``, no ``cargo``, no
``docker``, and no ``maturin`` in the venv. GitHub Actions is therefore the
only place pyegui is ever compiled, and every build claim in a pull request
must come from an actual workflow run rather than a local invocation.

The full setup, including the per-push ``check`` gate, the lockfile
regeneration workflow, and which ``Makefile`` targets cannot run here, is
documented in ``docs/development.rst`` (published as "Development and CI"
in the documentation site).

In short:

- ``check.yml`` runs on every push to ``main`` and ``feature/**``: ``cargo
  check --locked``, advisory ``cargo clippy``, a hard assertion that egui
  resolves to exactly 0.31.1, then ``maturin develop`` plus a Python import
  that asserts every expected export exists.
- Push work to the ``fork`` remote; ``origin`` is read-only.
- Commit only once ``check`` is green.

.. code:: bash

   git push fork feature/<name>
   gh run list --repo ChetanKnowIT/pyegui --branch feature/<name>

Install
-------

Prebuilt binaries are provided for Linux, Windows and macOS. On other platforms
pip will build wheel for your OS. In this case you'll need Rust compiler
and `maturin <https://github.com/PyO3/maturin>`__

Install from pypi:

.. code:: bash

   pip install pyegui

Install from source:

.. code:: bash

   git clone https://github.com/gachilord/pyegui
   pip install <path to pyegui>

Usage
-----

This is how you write a "hello world" app.

.. code:: python

   from pyegui import *

   def update_func(ctx):
     # draw UI here
     heading("Hello, World!")

   if __name__ == "__main__":
     run_native("Example app", update_func)

You can find more examples in the `documentation <https://gachilord.github.io/pyegui>`__.

Update functions
~~~~~~~~~~~~~~~~

**pyegui** has a notion of update functions which the library calls to
draw your UI.

.. code:: python

   def update_func():
     # you can place here any widget
     heading("I'm a heading")
     # some widgets are interactive
     if button_clicked("I'm a clickable button"):
       # you can update state from here or show another widget
       print("Clicked")

The top level update function has the Context object that controls
global aspects of your app(e.g fonts and theme).

.. code:: python

   def update_func(ctx):
     ctx.set_light_theme()
     heading("Using light theme even if system's is dark")

Update functions may be nested. Such functions create a new UI scope
that can have different styles and behaviour.

.. code:: python

   def update_func(ctx):
     # define update_func
     def nested():
       label("I'm a label inside nested update function")
       label("New label")
       disable() # this function will disable all further widgets in the scope
       if button_clicked("You can't click me"):
         print("Unreachable")
     # all the widgets inside 'nested' will be centered vertically 
     horizontal_centered(nested)
     # this widget won't be disabled though it goes after 'disable()'
     if button_clicked("You can click me"):
       print("Clicked")

Containers
~~~~~~~~~~~~~~~~

Containers is a syntactic sugar for code that needs update functions.
Function calls are replaced by Python's ``with`` statement.

The code that centers widgets vertically:

.. code:: python

   def update_func(ctx):

     def nested():
       label("I'm a label inside nested update function")
       label("New label")

     horizontal_centered(nested)

Can be written without callbacks:

.. code:: python

   def update_func(ctx):

     with Layout(LayoutType.HorizontalCentered):
       label("I'm a label inside nested update function")
       label("New label")

Variables
~~~~~~~~~

Many widgets require access to a state via a reference, which can't be
done for integers, floats and strings in Python. That's why such helper
classes as Str, Bool, Int and Float exist.

They are essentially the following:

.. code:: python

   # Example for bool type
   class Bool:
     value = False

These classes can be used to draw UI or to store user input. You have to
create them outside of update functions.

.. code:: python

   data = Bool(False)

   def update_func():
     heading(f"Value of the data is {data.value}")
     # button will be shown only if the checkbox is checked 
     if data.value and button_clicked("set to False"):
       # hiding the button
       data.value = False
     checkbox(data, "Check me")

.. |example 1| image:: https://github.com/GachiLord/pyegui/raw/main/example1.jpeg
.. |example 2| image:: https://github.com/GachiLord/pyegui/raw/main/example2.jpeg
