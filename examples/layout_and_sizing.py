"""Layout and sizing: columns, explicit sizes, and measurement.

egui sizes a `Ui` either explicitly (`set_width`, `set_max_height`) or from its
contents (`shrink_width_to_current`). It also hands back the numbers that
describe the space it has: `available_size`, `cursor`, `min_rect`. Those are
what a custom widget needs in order to draw itself.

Run with:

    python examples/layout_and_sizing.py
"""

from pyegui import *

# Module level: checkbox writes into the Bool it is given, so a Bool created
# inside the callback would reset the tick on every frame.
ticks = [Bool(True), Bool(False), Bool(True)]

# One Str per row, for the push_id demonstration. Without push_id egui gives
# these three widgets the same id and they share state.
values = [Str("") for _ in range(3)]


def col_left():
    heading("left")
    label("each column is its own Ui,")
    label("so its cursor is independent.")
    button_clicked("a button in column one")


def col_middle():
    heading("middle")
    label("3 columns of unequal content,")
    label("each sized to what it needs.")
    slider_float(0.5, 0.0, 1.0, "a slider")
    button_clicked("a button in column two")


def col_right():
    heading("right")
    label("end_row is not needed here:")
    label("egui wraps automatically when")
    label("the last column fills.")
    checkbox(ticks[0], "a checkbox")


def columns_demo():
    """Multi-column layout, one callable per column."""
    label("columns(3, [...]):")
    add_space(4.0)
    columns(3, [col_left, col_middle, col_right])
    add_space(8.0)

    # A row can be closed early, which is what end_row is for.
    label("columns(2, [...]) with end_row between them:")
    add_space(4.0)

    def first():
        button_clicked("row 1, left")

    def second():
        button_clicked("row 1, right")

    def third():
        button_clicked("row 2, left, after end_row")

    columns(2, [first, second])
    end_row()
    columns(2, [third, lambda: label("(empty)")])
    add_space(8.0)


def fixed_sizes():
    """Explicitly sized scopes. Each callback gets its own Ui, so a size set
    inside one does not leak into the next."""
    label("set_max_width(220.0) around a shrink-to-fit group:")
    add_space(4.0)

    def narrow():
        set_max_width(220.0)
        shrink_width_to_current()
        label("bounded, then shrunk to content")
        button_clicked("click me")

    frame_group(narrow)
    add_space(8.0)

    label("set_height on a fixed-height area:")
    add_space(4.0)

    def fixed():
        set_height(60.0)
        label("exactly 60 points tall")
        label("whatever does not fit is clipped")

    frame_group(fixed)
    add_space(8.0)

    label("set_width_range((120.0, 300.0)):")
    add_space(4.0)

    def ranged():
        set_width_range((120.0, 300.0))
        shrink_width_to_current()
        label("between 120 and 300 points wide")

    frame_group(ranged)
    add_space(8.0)


def measurement():
    """What the current Ui reports about its own space.

    These are the numbers a custom widget needs to size itself, and they are in
    egui points rather than pixels -- multiply by pixels_per_point().
    """
    label("measurement of the current Ui:")
    add_space(4.0)

    def readings():
        w, h = available_size()
        label(f"available_size        {w:.1f} x {h:.1f}")
        label(f"available_width       {available_width():.1f}")
        label(f"available_height      {available_height():.1f}")

        bw, bh = available_size_before_wrap()
        label(f"before wrap           {bw:.1f} x {bh:.1f}")

        c = cursor()
        label(f"cursor                ({c.min[0]:.1f}, {c.min[1]:.1f})")

        mr = min_rect()
        label(f"min_rect              {mr.width:.1f} x {mr.height:.1f}")

        mw, mh = max_rect().size
        label(f"max_rect              {mw:.1f} x {mh:.1f}")

        label(f"pixels_per_point      {pixels_per_point():.2f}")

        nx, ny = next_widget_position()
        label(f"next_widget_position  ({nx:.1f}, {ny:.1f})")

        # is_rect_visible is how a widget decides to skip expensive drawing.
        if is_rect_visible(cursor()):
            label("cursor is visible")

    frame_group(readings)
    add_space(8.0)


def ids_demo():
    """push_id gives a repeated widget its own id space.

    egui derives widget ids from position in the Ui tree, so identical widgets
    in a loop share one id and their state leaks between iterations. push_id
    fixes that, which is what makes a list of inputs work at all.
    """
    label("push_id, so each of these holds its own value:")
    add_space(4.0)

    for index, value in enumerate(values):
        push_id(index, lambda index=index, value=value: text_edit_singleline(
            value, hint_text=f"row {index}"
        ))
    add_space(4.0)
    label("the three fields above are independent")
    add_space(8.0)


def main_contents():
    heading("Layout and sizing")
    columns_demo()
    separator()
    add_space(8.0)
    fixed_sizes()
    separator()
    add_space(8.0)
    measurement()
    separator()
    add_space(8.0)
    ids_demo()


def update_func(ctx):
    central_panel(ctx, main_contents)


if __name__ == "__main__":
    run_native("Layout and sizing", update_func)
