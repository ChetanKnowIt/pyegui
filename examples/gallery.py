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

import os
import sys

from pyegui import *

# ---------------------------------------------------------------- preflight
# pyegui is a Rust extension module, so a build of this branch cannot be
# installed without a Rust toolchain. This repo has none on purpose --
# GitHub Actions is the only place anything is compiled. The result is that
# a local `uv run examples/gallery.py` happily imports whatever pyegui
# happens to be installed, and if that is an older wheel the gallery dies
# partway through with a bare `NameError: name 'RGBA' is not defined`.
#
# Catch that here instead, where the message can say what to do about it.

# Names this gallery needs that 0.5.0 did not have. If any are missing, the
# installed pyegui predates the 0.31.1 coverage work.
_REQUIRED = {
    "Response": "interaction state (button_response and friends)",
    "RGBA": "the alpha colour pickers",
    "HSVA": "the non-RGB colour pickers",
    "drag_angle": "the angle dials",
}

# Snapshot the names once. Do not call bare `dir()` inside the comprehension:
# on Python 3.11 (which CI runs) `dir()` there sees only the comprehension's
# own locals, so every name looks missing and this reports a false positive.
# PEP 709 changed that in 3.12, which is why it passes locally and fails in CI.
_have = set(dir())

try:
    _missing = {n: why for n, why in _REQUIRED.items() if n not in _have}
except NameError:  # from pyegui import * found no pyegui at all
    _missing = {"pyegui": "the module did not import"}

if _missing:
    import textwrap

    print(
        textwrap.dedent(
            """\
            This gallery needs a pyegui build from the egui-0.31-coverage
            branch, but the installed one is too old.

            Missing:
            """
        )
        + "\n".join(f"  {n:14} {why}" for n, why in sorted(_missing.items()))
        + textwrap.dedent(
            """

            The installed pyegui is a release build, not this branch. Because
            pyegui is a Rust extension module you cannot build it without a
            Rust toolchain, and this repo deliberately has none -- CI compiles
            everything.

            To see the gallery, run it in CI, which builds the current branch
            and renders every page:

                gh workflow run screenshot.yml --repo ChetanKnowIT/pyegui
                gh run download --repo ChetanKnowIT/pyegui \\
                    --name gallery-screenshots --dir .

            To build it locally instead, install Rust first:

                curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
                uv venv && uv pip install "maturin>=1.8,<2.0"
                uv run maturin develop

            Then re-run this script.
            """
        )
    )
    sys.exit(1)

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


def page(name, raw_frame=False):
    """Register a page so `gallery.py all` can find it.

    `raw_frame=True` means the page composes its own frame: `run_page` will
    not wrap it in a `central_panel`, so the page can draw its own panels in
    whatever order egui requires. A page that only draws widgets leaves this
    False and lets `run_page` supply the central panel.
    """

    def wrap(fn):
        PAGES.append((name, fn, raw_frame))
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
    label("scroll area, both axes, bounded:")
    scroll_area_both(
        lambda: horizontal(lambda: label("wide content that scrolls sideways too")),
        max_height=60.0,
    )

    add_space(8)
    label("collapsing, open by default and framed:")
    collapsing_response(
        "always visible",
        lambda: label("default_open=True, show_background=True"),
        default_open=True,
        show_background=True,
    )

    add_space(8)
    label("frames -- egui's presets, each reading the current style:")
    frame_group(lambda: label("frame_group"))
    frame_popup(lambda: label("frame_popup"))
    frame_canvas(lambda: label("frame_canvas"))

    add_space(8)
    label("a frame built by hand:")
    frame(
        lambda: label("fill, stroke, corner_radius, inner_margin"),
        fill=(40, 60, 90, 200),
        stroke=(1.0, (120, 160, 220, 255)),
        corner_radius=6,
        inner_margin=(10, 6),
    )

    add_space(8)
    label("menus -- click for a popup menu:")
    horizontal(lambda: menu_button("File", lambda: menu_button("Recent", menu_contents)))

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


# ---------------------------------------------------------------- containers

# State for the window page. Module-level so it survives across frames: the
# window's open state has to persist between them or it would flicker.

win_open = Bool(True)
sized_open = Bool(True)

# The modal is open by default so the screenshot shows it.
show_modal = Bool(True)

# What window() returned last frame: True when visible. Recorded so the page
# can prove egui honoured the `open` Bool, not merely that pixels appeared.
win_visible = Bool(True)
sized_visible = Bool(True)


def window_contents():
    heading("A window")
    label("drawn inside window()")
    if button_clicked("close this window"):
        win_open.value = False
    if button_clicked("reopen"):
        win_open.value = True


def sized_window_contents():
    heading("Sized window")
    label("default_size=(420.0, 260.0), resizable=True")
    label(f"win_open.value = {win_open.value}")
    if button_clicked("close"):
        sized_open.value = False
    if button_clicked("reopen"):
        sized_open.value = True


def nested_window_contents():
    # The central panel sits behind the floating windows, so text there is
    # clipped in the render and cannot be read back. Report the state from
    # here instead, where the capture shows it in full.
    heading("Nested containers")
    separator()
    label(f"win_open.value     = {win_open.value}")
    label(f"window() returned  = {win_visible.value}")
    label(f"sized_open.value   = {sized_open.value}")
    label(f"window() returned  = {sized_visible.value}")
    separator()

    def inner():
        label("collapsing() inside a window")
        strong("still inside the window")

    collapsing("open me", inner)


def outer_collapsing_contents():
    heading("Collapsing inside the central panel")
    label("the window is drawn after the central panel")


@page("containers", raw_frame=True)
def page_containers(ctx):
    """Windows, and a window that never closes on its own.

    egui requires the central panel to be added after every other top-level
    panel, so this page draws its own frame rather than taking run_page's
    default single central panel.
    """

    # The central panel is added first here because this page has no side or
    # top/bottom panel to order against it. Windows come after, which is what
    # egui asks for.
    def main_contents():
        outer_collapsing_contents()

    central_panel(ctx, main_contents)

    win_visible.value = window(
        ctx,
        "A window",
        "w1",
        window_contents,
        open=win_open,
        default_pos=(60.0, 60.0),
        default_size=(420.0, 220.0),
        resizable=True,
    )

    sized_visible.value = window(
        ctx,
        "Sized window",
        "w2",
        sized_window_contents,
        open=sized_open,
        default_size=(420.0, 260.0),
        resizable=True,
        order="foreground",
    )

    window(
        ctx,
        "Nested",
        "w3",
        nested_window_contents,
        default_pos=(470.0, 40.0),
        default_size=(400.0, 320.0),
        collapsible=True,
    )


@page("panels", raw_frame=True)
def page_panels(ctx):
    """A real multi-panel layout: side, top, bottom, then central last.

    This is the ordering egui requires, and it is the only way to see whether
    the panels are actually independent. If `central_panel` were still being
    opened implicitly, this page would either fail to draw the side panel or
    paint over it.
    """

    def side_contents():
        heading("Side")
        label("Left panel")
        separator()
        label("egui wants the central panel")
        label("drawn last, so it takes")
        label("what is left over.")

    def top_contents():
        label("Top panel")

    def bottom_contents():
        label("Bottom panel")

    def main_contents():
        heading("Central panel")
        label("Drawn last, so it fills the middle.")
        separator()
        label("If the side panel is visible here, the panels overlap.")

    # Order matters and egui is explicit about it: side, top, bottom, then
    # central. Reversing central into the middle produces overlapping panels
    # rather than an error, which is why this page exists.
    side_panel_left(ctx, "side", side_contents, default_width=200.0)
    top_panel(ctx, "top", top_contents, default_height=44.0)
    bottom_panel(ctx, "bottom", bottom_contents, default_height=44.0)
    central_panel(ctx, main_contents)


@page("overlays", raw_frame=True)
def page_overlays(ctx):
    """A modal and a resizable area.

    The modal is drawn before the central panel, because egui wants every
    other top-level container first and the central panel last -- otherwise the
    central panel covers the modal's backdrop and the modal cannot be dismissed
    by clicking outside it.
    """

    def modal_contents():
        heading("Modal")
        label("Click outside this, or use Close, to dismiss.")
        separator()
        if button_clicked("Close"):
            ctx.close()

    # The modal must be drawn before the central panel, so the central panel
    # does not cover its backdrop.
    if show_modal.value:
        if modal(ctx, "demo_modal", modal_contents, default_width=300.0):
            show_modal.value = False

    def main_contents():
        label("The central panel is drawn last.")
        # Widgets need a Ui, and this page composes its own frame, so the
        # button has to live inside a container rather than here.
        if button_clicked("Reopen modal"):
            show_modal.value = True
        separator()
        heading("A resizable area")
        # resize() needs an existing Ui, so it can only be called from inside
        # another container -- not directly from update_func.
        resize(
            lambda: label("Drag my bottom-right corner."),
            default_width=260.0,
            default_height=70.0,
            resizable=True,
        )

    central_panel(ctx, main_contents)


# State for the layout page. Module level, because checkbox writes into the Bool
# it is handed and a fresh one each frame would reset the tick.
layout_ticks = [Bool(True), Bool(False)]


def gallery_col_a():
    heading("column A")
    label("each column is its own Ui,")
    label("with its own cursor.")
    checkbox(layout_ticks[0], "a checkbox")


def gallery_col_b():
    heading("column B")
    label("sized to content,")
    label("bounded by set_max_width.")
    slider_float(0.5, 0.0, 1.0, "a slider")


def gallery_col_c():
    heading("column C")
    checkbox(layout_ticks[1], "another checkbox")
    label("end_row closes a row early;")


@page("layout2")
def page_layout2(ctx):
    """Columns, explicit sizing, and the numbers a Ui reports about itself."""

    def sized():
        set_max_width(200.0)
        shrink_width_to_current()
        label("set_max_width(200) + shrink")
        button_clicked("click me")

    def readings():
        w, h = available_size()
        label(f"available  {w:.0f} x {h:.0f}")
        label(f"ppp        {pixels_per_point():.2f}")
        nx, ny = next_widget_position()
        label(f"next pos   ({nx:.0f}, {ny:.0f})")

    def main_contents():
        label("columns(3, [...]):")
        add_space(4.0)
        columns(3, [gallery_col_a, gallery_col_b, gallery_col_c])
        add_space(8.0)

        frame_group(sized)
        add_space(6.0)
        frame_group(readings)

    central_panel(ctx, main_contents)


# The scene's visible region, which egui writes as the user pans and zooms.
# Module level, because it has to be the same object every frame -- a fresh Rect
# each frame would discard whatever the user just did.
scene_view = Rect.zero()

scene_grid = Bool(True)


def scene_page_contents():
    """Inside the scene: placed in scene coordinates, not screen coordinates."""
    heading("Scene")
    label("Drag to pan, scroll to zoom.")
    separator()
    checkbox(scene_grid, "show the grid")
    add_space(6.0)

    frame(
        lambda: label("a frame, inside a transformed Ui"),
        fill=(40, 60, 90, 200),
        stroke=(1.0, (120, 160, 220, 255)),
        corner_radius=6,
        inner_margin=(10, 6),
    )

    if scene_grid.value:
        add_space(6.0)
        for row in range(10):
            horizontal(lambda row=row: [
                label(f"({col * 130}, {row * 40})") for col in range(6)
            ])


@page("scene")
def page_scene(ctx):
    """A pan-and-zoom canvas, and the Rect egui writes back into it.

    The scene takes whatever space is left, so the readout of its state goes
    first. The numbers below are written by egui as the user interacts, not
    computed by this page -- which is the whole point of the container.
    """

    def readout():
        label(f"view min   {scene_view.min}")
        label(f"view max   {scene_view.max}")
        label(f"view size  {scene_view.size}")
        label(f"finite     {scene_view.is_finite()}")
        if button_clicked("reset"):
            scene_view.min_x = 0.0
            scene_view.min_y = 0.0
            scene_view.max_x = 0.0
            scene_view.max_y = 0.0

    def main_contents():
        frame_group(readout)
        add_space(8.0)
        scene(scene_page_contents, scene_view, zoom_range=(0.1, 4.0))

    central_panel(ctx, main_contents)


# ---------------------------------------------------------------- driver

# The screenshot has to be taken while the window still exists, so the gallery
# takes it itself rather than letting CI capture the X root afterwards. An
# earlier version closed the app and then captured the root, which reliably
# produced a photograph of an empty desktop.
#
# MAX_SECONDS bounds the run. A frame counter cannot: if the window never
# mapped, the count is never reached and the process hangs. The CI job also
# wraps each page in `timeout` as a second line of defence.
MAX_SECONDS = 6.0
SETTLE_SECONDS = 1.0

GALLERY_WIDTH = 560
GALLERY_HEIGHT = 820


def capture(path):
    """Grab the window region of the X display into `path`.

    Uses ImageMagick's `import`, which is what the CI runner has. Failing to
    capture is not fatal here: the caller reports it and the workflow's
    blank-frame check is the real gate.
    """
    import shutil
    import subprocess

    if shutil.which("import") is None:
        print("no 'import' binary; cannot capture")
        return False

    display = os.environ.get("DISPLAY", ":0")
    result = subprocess.run(
        [
            "import",
            "-display", display,
            "-window", "root",
            "-crop", f"{GALLERY_WIDTH}x{GALLERY_HEIGHT}+0+0",
            "+repage",
            path,
        ],
        capture_output=True,
    )
    if result.returncode != 0:
        print("capture failed:", result.stderr.decode(errors="replace").strip())
        return False
    return True


def run_page(page_name, page_fn, out_dir=None, raw_frame=False):
    """Draw one page, screenshot it, then close the window.

    Repaints explicitly each frame. egui only repaints when it detects a
    change, so without the request the page would draw once and then idle,
    leaving the capture racing the compositor.

    `raw_frame=True` hands frame composition to the page, for pages that draw
    their own panels.
    """

    import time

    started = time.monotonic()

    def update(ctx):
        # Python composes the frame now: the central panel is drawn here, by
        # us, rather than implicitly by run_native. It must be the only
        # top-level panel on the page.
        #
        # `ctx` belongs to update's frame, so it cannot be closed over by a
        # sibling function defined here; pass it as an argument instead.
        #
        # A raw_frame page draws its own panels instead -- egui requires
        # CentralPanel last, and a container page needs a side panel before
        # it, so the single central panel here would be the wrong shape.
        if raw_frame:
            page_fn(ctx)
        else:
            def draw_page():
                page_fn(ctx)

            central_panel(ctx, draw_page)

        if time.monotonic() - started >= MAX_SECONDS:
            # Let the final frame composite before grabbing it.
            time.sleep(SETTLE_SECONDS)

            if out_dir is not None:
                target = os.path.join(out_dir, f"{page_name}.png")
                if capture(target):
                    print(f"captured {target} ({os.path.getsize(target)} bytes)")

            ctx.close()
            return

        ctx.request_repaint()

    run_native(
        f"pyegui gallery - {page_name}",
        update,
        inner_width=GALLERY_WIDTH,
        inner_height=GALLERY_HEIGHT,
    )


def main():
    args = sys.argv[1:]

    if not args or args[0] in ("all", "-h", "--help"):
        print("usage: gallery.py <page> [--out DIR]")
        print("       gallery.py list")
        print()
        print("pages:")
        for page_name, _, _ in PAGES:
            print(f"  {page_name}")
        return

    if args[0] == "list":
        print("\n".join(name for name, _, _ in PAGES))
        return

    wanted = args[0]
    out_dir = None
    if "--out" in args:
        out_dir = args[args.index("--out") + 1]
        os.makedirs(out_dir, exist_ok=True)

    lookup = {name: (fn, raw) for name, fn, raw in PAGES}
    if wanted not in lookup:
        print(f"unknown page {wanted!r}; try 'gallery.py list'")
        sys.exit(1)

    run_page(wanted, lookup[wanted][0], out_dir, lookup[wanted][1])


if __name__ == "__main__":
    main()
