"""The same app as bench/hello_egui.rs, in Python.

Paired widget-for-widget with the Rust version so the ergonomics comparison
rests on identical work. Honest in both directions: this is shorter largely
because the binding takes a callable where egui takes a generic `impl FnOnce`,
not because it does less.

Every widget here is egui's own, and every Response is `egui::Response` -- there
is no second set of concepts to learn. That is the actual ergonomics claim, and
the line count is secondary to it.
"""

import pyegui
from pyegui import *

slider_value = Float(0.5)
checkbox_value = Bool(False)
name = Str("world")
clicks = Int(0)


def contents():
    heading("hello, pyegui")

    # egui's Response, not a binding's redefinition of it.
    if button_clicked("click me"):
        clicks.value += 1
    label(f"clicked {clicks.value} times")

    r = checkbox_response(checkbox_value, "enabled")
    label(f"checkbox changed: {r.changed}")

    s = slider_float_response(slider_value, 0.0, 1.0, "amount")
    label(f"dragged: {s.dragged}")

    t = text_edit_singleline_response(name)
    label(f"text changed: {t.changed}")

    scroll_area_vertical(lambda: [label(f"row {i}") for i in range(50)])

    if button_clicked("quit"):
        ctx.close()


def run(ctx):
    # Explicit, and drawn last -- the counterpart to
    # CentralPanel::default().show(ctx, ...).
    central_panel(ctx, contents)


if __name__ == "__main__":
    pyegui.run_native("hello", run)
