"""A pan-and-zoom canvas, mirroring `egui::Scene`.

A scene takes over the space its parent `Ui` has left. The user drags to pan
and scrolls to zoom, and the visible region is a `Rect` that egui reads and
writes as they do -- so it has to be the same object every frame.

Run with:

    python examples/scene_canvas.py
"""

from pyegui import *

# The visible region. egui mutates this as the user pans and zooms, so it is
# created once at module level and passed again every frame. Creating a fresh
# Rect each frame would throw away the pan and zoom the user just did.
view = Rect.zero()

# Module level, not created inside the callback: checkbox writes into the Bool
# it is handed, and a new Bool each frame would reset the tick every frame.
show_grid = Bool(True)


def grid():
    """A grid of labels, larger than the viewport so panning has somewhere to go."""
    cols, rows = 12, 8
    for row in range(rows):
        horizontal(lambda row=row: [
            label(f"r{row} c{col}  ({col * 120}, {row * 110})")
            for col in range(cols)
        ])


def scene_contents():
    """Drawn inside the scene, in the scene's own coordinates."""
    heading("inside the scene")
    label("this text is placed in scene coordinates, not screen coordinates")
    add_space(6.0)

    horizontal(lambda: [
        button_clicked("a button"),
        button_clicked("another"),
    ])

    add_space(6.0)
    checkbox(show_grid, "show the grid")
    add_space(6.0)

    # A frame, to show the frame bindings compose inside a transformed Ui.
    frame(
        lambda: label("frames work inside a scene too"),
        fill=(40, 60, 90, 200),
        stroke=(1.0, (120, 160, 220, 255)),
        corner_radius=6,
        inner_margin=(10, 6),
    )

    if show_grid.value:
        add_space(6.0)
        grid()


def readout():
    """Drawn outside the scene, reading the state egui wrote back."""
    heading("View state")
    label(f"min      {view.min}")
    label(f"max      {view.max}")
    label(f"size     {view.size}")
    label(f"width    {view.width:.1f}")
    label(f"height   {view.height:.1f}")
    label(f"center   {view.center}")
    label(f"finite   {view.is_finite()}")
    label("")
    label("Drag inside the canvas to pan. Scroll to zoom.")
    label("The view rect above is written by egui, not by this app.")


def reset_button():
    """Reset the view. egui re-fits the contents on the next frame."""
    if button_clicked("reset view"):
        view.min_x = 0.0
        view.min_y = 0.0
        view.max_x = 0.0
        view.max_y = 0.0
        label("view reset; egui will re-fit on the next frame")


def main_contents():
    heading("egui Scene")

    reset_button()
    add_space(8.0)

    # The scene takes all remaining space, so the readout goes first.
    scene(scene_contents, view, zoom_range=(0.1, 4.0))

    add_space(8.0)
    frame_group(readout)


def update_func(ctx):
    central_panel(ctx, main_contents)


if __name__ == "__main__":
    run_native("Scene", update_func)
