"""Menus, submenus, and the popup surface egui 0.31.1 actually has.

egui 0.31.1 has no menu-bar container and no `Popup` type. Its whole menu
surface is `menu_button`, and a `menu_button` inside a `menu_button` is a
submenu. The popup side is tooltips and context menus, reached through
`Response`.

Run with:

    python examples/menus_and_popups.py
"""

from pyegui import *

# Menu state. `checkbox` writes into a `Bool` it is given every frame, so
# these are module level rather than local to a callback that runs once.
search = Str("")
wrap = Bool(True)


def recent_menu():
    """The submenu under File > Recent."""
    for name in ("notes.txt", "todo.md", "scratch.rs"):
        if button_clicked(name):
            label("would open", name)

    close_menu()


def file_menu():
    """Items of the File menu. A menu item is an ordinary widget."""
    if button_clicked("New"):
        label("would create a new file")
    if button_clicked("Open..."):
        label("would open a file")
    if button_clicked("Save"):
        label("would save")

    # A menu_button inside a menu is a submenu. It opens to the side.
    menu_button("Recent", recent_menu)

    separator()

    if button_clicked("Quit"):
        close_menu()


def find_menu():
    """A submenu holding a real input widget."""
    text_edit_singleline(search)
    if button_clicked("Done"):
        close_menu()


def edit_menu():
    """A second top-level menu, to show they sit side by side."""
    if button_clicked("Undo"):
        label("would undo")

    # egui ships checkbox and radio widgets, so a menu can hold real controls
    # rather than only buttons.
    checkbox(wrap, "wrap lines")

    menu_button("Find", find_menu)

    close_menu()


def help_menu():
    if button_clicked("About"):
        label("pyegui, an egui binding")

    close_menu()


def menu_row():
    """The buttons of the hand-built menu bar."""
    menu_button("File", file_menu)
    menu_button("Edit", edit_menu)
    menu_button("Help", help_menu)


def menu_bar():
    """A hand-built menu bar.

    egui has no MenuBar container, so this is a `frame_menu` around a row of
    menu buttons -- which is what a menu bar is, structurally. Worth replacing
    with egui's own `MenuBar` when the pin moves to a version that ships it.
    """
    horizontal(menu_row)


def context_menu_contents():
    """Opened by right-clicking the button in popup_surface."""
    if button_clicked("Copy"):
        label("would copy")
    if button_clicked("Paste"):
        label("would paste")

    close_menu()


def popup_surface():
    """The popup surface that does exist in 0.31.1.

    A tooltip on hover, and a context menu on right-click. Both come from
    `Response`, not from a `Popup` type.
    """
    heading("Tooltips and context menus")

    # `on_hover_text` shows a tooltip while the widget is hovered.
    response = button_response("Hover me")
    response.on_hover_text("here is the tooltip")
    if response.clicked:
        label("clicked")

    # `context_menu` opens a menu on right-click, and its callback runs only
    # while the menu is open.
    button_response("Right-click me").context_menu(context_menu_contents)

    # A disabled tooltip is a separate method, because egui does not show the
    # normal one for a disabled widget.
    button_response("Disabled").on_disabled_hover_text("nothing to do here")


def collapsing_body():
    label("This body starts visible, because default_open=True.")
    label("It is also framed, because show_background=True.")
    if button_clicked("a button inside the body"):
        label("the body ran its callback")


def main_contents():
    heading("pyegui menus")
    frame_menu(menu_bar, inner_margin=4)
    add_space(12.0)

    heading("Frames")
    label("egui's eight Frame presets are frame_* constructors. Each reads the")
    label("current style, so they look right in any theme.")

    frame_group(lambda: label("frame_group: a filled, rounded group"))
    frame_popup(lambda: label("frame_popup: how popups and tooltips are drawn"))
    frame_canvas(lambda: label("frame_canvas: the flat surface behind a plot"))

    # `frame` builds one from scratch rather than from a preset, so the fill,
    # stroke, corner radius and padding are all named explicitly.
    frame(
        lambda: label("frame: built by hand"),
        fill=(40, 60, 90, 200),
        stroke=(1.0, (120, 160, 220, 255)),
        corner_radius=6,
        inner_margin=(10, 6),
    )

    add_space(12.0)
    popup_surface()

    add_space(12.0)

    # CollapsingHeader with the builder options the old `collapsing` helper
    # could not reach.
    collapsing_response(
        "Details", collapsing_body, default_open=True, show_background=True
    )


def update_func(ctx):
    central_panel(ctx, main_contents)


if __name__ == "__main__":
    run_native("Menus and popups", update_func)
