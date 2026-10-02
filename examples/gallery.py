"""Widget gallery, one section per invocation.

Run a single page:

    python examples/gallery.py text
    python examples/gallery.py colours

Run them all (this is what CI does):

    python examples/gallery.py all

Each page opens a window, draws itself, then closes via ``ctx.close()``. That
makes it deterministic under ``xvfb-run``, where there is no user to click
anything: the frame count is controlled from Python rather than by a human.

The state holders are module-level on purpose. They have to outlive a single
``update_func`` call, because egui runs that function once per frame and the
gallery draws across many frames.
"""

import sys

from pyegui import *

# ---------------------------------------------------------------- state
# Kept at module scope so values persist across frames.

name = Str("pyegui")
bio = Str("a Python binding for egui")
enabled = Bool(True)
notifications = Bool(False)

choice = Int(0)
COLOURS = ["red", "green", "blue"]

volume = Float(0.5)
count = Int(3)
angle = Float(0.0)

toggle_a = Bool(True)
toggle_b = Bool(False)

colour_rgb = RGB(0.9, 0.3, 0.3)
colour_rgba = RGBA(0.2, 0.6, 0.9, 0.8)
colour_hsva = HSVA(0.55, 0.7, 0.9, 1.0)
colour_srgb = SRGB(240, 130, 60)
colour_srgba = Color32(240, 130, 60, 200)

import datetime

date = Date(datetime.datetime(2026, 10, 2))

click_count = Int(0)
last_action = Str("nothing yet")

PAGES = []


def page(name):
    """Register a page so `gallery.py all` can find it."""

    def wrap(fn):
        PAGES.append((name, fn))
        return fn

    return wrap


# ---------------------------------------------------------------- pages


@page("text")
def page_text(ctx):
    heading("Text")
    separator()
    add_space(8)

    label("plain label")
    heading("heading")
    strong("strong")
    weak("weak")
    monospace("monospace")
    small("small")
    code("code()")

    add_space(8)
    text_edit_singleline(name, hint_text="your name")
    text_edit_multiline(bio, hint_text="a short bio")
    code_editor(bio)

    add_space(8)
    hyperlink("https://github.com/emilk/egui")
    hyperlink_to("pyegui on GitHub", "https://github.com/GachiLord/pyegui")
    link_clicked("I'm a fake link")


@page("buttons")
def page_buttons(ctx):
    heading("Buttons and links")
    separator()
    add_space(8)

    button_clicked("button")
    small_button_clicked("small button")
    image_and_text_clicked(
        "https://github.githubassets.com/favicons/favicon.svg", "with an icon"
    )

    add_space(8)
    if button_clicked("count a click"):
        click_count.value += 1
    label(f"clicked {click_count.value} times")

    add_space(8)
    with Group():
        label("inside a Group")
        button_clicked("grouped button")


@page("selection")
def page_selection(ctx):
    heading("Selection")
    separator()
    add_space(8)

    checkbox(enabled, "enabled")
    toggle_value(notifications, "notifications")

    add_space(8)
    with Layout(LayoutType.Horizontal):
        radio_value(choice, 0, "red")
        radio_value(choice, 1, "green")
        radio_value(choice, 2, "blue")

    with Layout(LayoutType.Horizontal):
        selectable_value(choice, 0, "red")
        selectable_value(choice, 1, "green")
        selectable_value(choice, 2, "blue")

    selectable_label(toggle_a, "selectable_label")
    radio(toggle_b, "radio")

    add_space(8)
    combo_box(choice, [0, 1, 2], COLOURS, "combo box")
    label(f"choice is {COLOURS[choice.value]}")

    add_space(8)
    if button_clicked("close the menu"):
        close_menu()


@page("numbers")
def page_numbers(ctx):
    heading("Numbers")
    separator()
    add_space(8)

    slider_float(volume, 0.0, 1.0, "volume")
    label(f"volume is {volume.value:.2f}")

    slider_int(count, 0, 10, "count")

    drag_float(volume, 0.0, 1.0, 0.01)
    drag_int(count, 0, 10, 1)

    add_space(8)
    drag_angle(angle)
    drag_angle_tau(angle)
    label(f"angle is {angle.value:.2f} rad")

    add_space(8)
    progress(0.6)
    spinner()


@page("colours")
def page_colours(ctx):
    heading("Colour pickers")
    separator()
    add_space(8)
    label("egui 0.31.1 exposes eight colour spaces; pyegui wraps all eight.")

    add_space(8)
    color_edit_button_rgb(colour_rgb)
    color_edit_button_rgba_unmultiplied(colour_rgba)
    color_edit_button_hsva(colour_hsva)
    color_edit_button_srgb(colour_srgb)
    color_edit_button_srgba(colour_srgba)

    add_space(8)
    date_picker_button(date)
    label(f"date is {date.value}")


@page("response")
def page_response(ctx):
    heading("Response")
    separator()
    add_space(8)
    label("Every widget returns a Response. The *_response functions expose it.")

    add_space(8)
    response = button_response("hover me for a tooltip")
    if response.clicked:
        last_action.value = "clicked"
    if response.hovered:
        response.on_hover_text("this tooltip comes from Response.on_hover_text")
    label(f"last action: {last_action.value}")

    add_space(8)
    checkbox_response(enabled, "checkbox with a Response")
    if response.has_focus:
        label("the button has keyboard focus")

    add_space(8)
    slider_float_response(volume, 0.0, 1.0, "slider with a Response")
    if slider_float_response(volume, 0.0, 1.0, "").changed:
        last_action.value = "slider moved"

    add_space(8)
    if button_response("right click for a context menu").context_menu(menu_contents):
        label("context menu is open")


def scroll_rows():
    """Rows for the scroll area on the layout page."""
    for i in range(20):
        label(f"row {i}")


def demo_scoped_ui_state():
    """Show set_opacity, which applies to everything after it in this Ui.

    disable() and set_invisible() are deliberately not demonstrated here: both
    are permanent for the rest of the Ui, so calling either would blank every
    later widget on the page. They are one-liners in the README instead.
    """
    set_opacity(0.5)
    button_clicked("set_opacity(0.5)")
    set_opacity(1.0)


def menu_contents():
    """Contents of the context menu on the response page."""
    if button_clicked("cut"):
        last_action.value = "cut"
    if button_clicked("copy"):
        last_action.value = "copy"


@page("layout")
def page_layout(ctx):
    heading("Layout and scopes")
    separator()
    add_space(8)

    label("horizontal:")
    with Layout(LayoutType.Horizontal):
        button_clicked("one")
        button_clicked("two")
        button_clicked("three")

    add_space(8)
    label("centered:")
    with Layout(LayoutType.VerticalCentered):
        button_clicked("centered")

    add_space(8)
    label("collapsing:")
    collapsing("click to expand", lambda: label("hidden until expanded"))

    add_space(8)
    label("group:")
    with Group():
        label("inside a group")

    add_space(8)
    label("scroll area:")
    scroll_area_vertical(scroll_rows)

    add_space(8)
    label("indent:")
    indent(lambda: label("indented"))


@page("state")
def page_state(ctx):
    heading("Ui state")
    separator()
    add_space(8)

    button_clicked("normal")
    add_enabled(False, lambda: button_clicked("add_enabled(False)"))

    # disable / set_invisible / set_opacity apply to the rest of the frame and
    # cannot be undone, so they go last, each in its own sub-Ui where possible.
    add_enabled(True, lambda: demo_scoped_ui_state())
    add_space(12)
    separator()
    add_space(12)

    label(f"enabled={enabled.value} notifications={notifications.value}")
    label(f"name={name.value!r} choice={COLOURS[choice.value]}")


# ---------------------------------------------------------------- driver

# The screenshot has to catch a fully drawn window, so the gallery repaints for
# a fixed time and then closes itself via ctx.close().
#
# MAX_SECONDS is the bound that matters. A frame counter alone can never be
# reached if the window fails to map, and the app would then hang until the
# CI job's own timeout -- six hours by default. The CI job wraps each page in
# `timeout` as a second line of defence, but the app should not depend on
# being killed.
MAX_SECONDS = 6.0
SETTLE_SECONDS = 1.0


def run_page(page_name, page_fn):
    """Draw one page for a fixed time, then close the window.

    Repaints explicitly each frame. egui only repaints when it detects a
    change, so without the explicit request the page would draw once and then
    idle -- which is fine interactively but leaves the screenshot racing the
    compositor.
    """

    import time

    started = time.monotonic()

    def update(ctx):
        page_fn(ctx)

        if time.monotonic() - started >= MAX_SECONDS:
            # Hold briefly so the last frame is composited before closing,
            # otherwise the capture can catch an undrawn window.
            time.sleep(SETTLE_SECONDS)
            ctx.close()
            return

        ctx.request_repaint()

    run_native(
        f"pyegui gallery - {page_name}",
        update,
        inner_width=GALLERY_WIDTH,
        inner_height=GALLERY_HEIGHT,
    )


GALLERY_WIDTH = 560
GALLERY_HEIGHT = 820


def main():
    wanted = sys.argv[1] if len(sys.argv) > 1 else "all"
    lookup = dict(PAGES)

    if wanted == "list":
        print("\n".join(name for name, _ in PAGES))
        return

    if wanted == "all":
        print("this gallery renders one page per invocation:")
        print("  python examples/gallery.py list")
        print("  python examples/gallery.py text")
        print()
        print("CI loops over the pages and screenshots each one. See")
        print(".github/workflows/screenshot.yml.")
        return

    if wanted not in lookup:
        print(f"unknown page {wanted!r}; try one of:")
        print("  " + ", ".join(lookup))
        sys.exit(1)

    run_page(wanted, lookup[wanted])


if __name__ == "__main__":
    main()
