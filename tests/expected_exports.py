"""Every public name pyegui is expected to export.

This is the CI export gate. `check.yml` builds the extension module, installs
the wheel and runs this file against it, so a `#[pyfunction]` that compiles but
is missing its `#[pymodule]` registration fails the build instead of silently
shrinking the Python API.

When you add a public name to `src/lib.rs`, add it here in the same change.
The lists are sorted; `verify_exports.py` checks that against the module.

Groups, in order:

* classes -- `#[pyclass]` types, reachable as `pyegui.Name`
* functions -- module-level `#[pyfunction]`s that do not return a `Response`
* response_variants -- the `*_response` functions returning a `Response`
"""

CLASSES = [
    "Bool",
    "Color32",
    "Context",
    "Date",
    "Float",
    "Group",
    "HSVA",
    "Int",
    "Layout",
    "LayoutType",
    "PointerButton",
    "RGB",
    "RGBA",
    "Response",
    "SRGB",
    "Scope",
    "Str",
]

FUNCTIONS = [
    "add_enabled",
    "add_space",
    "button_clicked",
    "centered_and_justified",
    "area",
    "bottom_panel",
    "modal",
    "resize",
    "central_panel",
    "side_panel_left",
    "side_panel_right",
    "top_panel",
    "checkbox",
    "close_menu",
    "code",
    "code_editor",
    "collapsing",
    "color_edit_button_hsva",
    "color_edit_button_rgb",
    "color_edit_button_rgba_premultiplied",
    "color_edit_button_rgba_unmultiplied",
    "color_edit_button_srgb",
    "color_edit_button_srgba",
    "color_edit_button_srgba_premultiplied",
    "color_edit_button_srgba_unmultiplied",
    "combo_box",
    "date_picker_button",
    "disable",
    "drag_angle",
    "drag_angle_tau",
    "drag_float",
    "drag_int",
    "group",
    "heading",
    "horizontal",
    "horizontal_centered",
    "horizontal_top",
    "horizontal_wrapped",
    "hyperlink",
    "hyperlink_to",
    "image",
    "image_and_text_clicked",
    "indent",
    "label",
    "link_clicked",
    "monospace",
    "progress",
    "radio",
    "radio_value",
    "run_native",
    "scope",
    "scroll_area_horizontal",
    "scroll_area_vertical",
    "selectable_label",
    "selectable_value",
    "separator",
    "set_invisible",
    "set_opacity",
    "slider_float",
    "slider_int",
    "small",
    "small_button_clicked",
    "spinner",
    "strong",
    "text_edit_multiline",
    "text_edit_singleline",
    "toggle_value",
    "vertical",
    "vertical_centered",
    "vertical_centered_justified",
    "weak",
    "window",
]

RESPONSE_VARIANTS = [
    "button_response",
    "checkbox_response",
    "code_editor_response",
    "code_response",
    "color_edit_button_hsva_response",
    "color_edit_button_rgb_response",
    "color_edit_button_rgba_premultiplied_response",
    "color_edit_button_rgba_unmultiplied_response",
    "color_edit_button_srgb_response",
    "color_edit_button_srgba_premultiplied_response",
    "color_edit_button_srgba_response",
    "color_edit_button_srgba_unmultiplied_response",
    "date_picker_button_response",
    "drag_angle_response",
    "drag_angle_tau_response",
    "drag_float_response",
    "drag_int_response",
    "heading_response",
    "hyperlink_response",
    "hyperlink_to_response",
    "image_and_text_response",
    "image_response",
    "label_response",
    "link_response",
    "monospace_response",
    "progress_response",
    "radio_response",
    "radio_value_response",
    "selectable_label_response",
    "selectable_value_response",
    "separator_response",
    "slider_float_response",
    "slider_int_response",
    "small_button_response",
    "small_response",
    "spinner_response",
    "strong_response",
    "text_edit_multiline_response",
    "text_edit_singleline_response",
    "toggle_value_response",
    "weak_response",
]

# `run_native` is the entrypoint and is listed on its own in check.yml rather
# than here, so that a failure to import the module at all is distinguishable
# from a missing widget.
REQUIRED = set(CLASSES) | set(FUNCTIONS) | set(RESPONSE_VARIANTS) | {"run_native"}

# Methods on the exported classes are NOT listed here: only module-level names
# show up in `dir(pyegui)`. For example `Context.close` exists but is not a
# module-level export, so it must not be added to the lists above. The class
# itself is declared; its methods are checked by being called from
# examples/gallery.py, which the screenshot workflow renders.

# Names that appear on a module object without being declared API. Kept small
# and explicit rather than pattern-matched, so that adding a real widget never
# gets silently excused by an over-broad rule.
_INTERPRETER_NAMES = {
    # pyo3 names the submodule after the crate and binds it on the module.
    "pyegui",
    # CPython module attributes and the usual frozen-importlib helpers.
    "copyreg",
    "sys",
    "TESTING",
    "ISOLATED",
    "load",
}


def check(module) -> None:
    """Raise AssertionError listing every name `module` fails to export."""
    missing = sorted(name for name in REQUIRED if not hasattr(module, name))
    assert not missing, f"missing exports: {missing}"


def check_no_unexpected(module) -> None:
    """Raise AssertionError if `module` exports something not listed here.

    This is the other half of the gate. Without it, a name could be removed
    from this file and from the module together and CI would stay green while
    the API quietly lost a widget.
    """
    exported = {name for name in dir(module) if not name.startswith("_")}

    # A module object also carries names that are not part of the pyegui API:
    # pyo3 binds the submodule's own name on it, and CPython attaches a few of
    # its own. Subtract those before comparing. The set is deliberately
    # explicit rather than pattern-matched, so that adding a real widget can
    # never be silently excused by an over-broad rule.
    exported -= _INTERPRETER_NAMES

    undeclared = sorted(exported - REQUIRED)
    assert not undeclared, (
        f"exported but not declared in expected_exports.py: {undeclared}"
    )
