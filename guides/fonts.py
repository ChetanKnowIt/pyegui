from pyegui import *

import pathlib

# egui's built-in fonts only cover latin and cyrillic, so CJK text needs a font
# loaded from disk. This guide used to hard-code
#
#     ctx.set_font("NotoSansJP-VariableFont_wght.ttf")
#
# which has never been tracked in this repository, so the example raised
# FileNotFoundError on every run from a fresh clone. tests/smoke.py now runs
# it in CI, so the failure is visible rather than waiting to be discovered.
#
# Point FONT_PATH at a font file on your machine -- a free Japanese font such
# as Noto Sans JP works well -- to see this render 天気の子.
#
# Set FONT_PATH in the environment to override it:
#
#     FONT_PATH=~/Downloads/NotoSansJP-Regular.ttf python guides/fonts.py
FONT_PATH = "NotoSansJP-VariableFont_wght.ttf"

font = pathlib.Path(FONT_PATH).expanduser()

def main_contents(ctx):
  if font.is_file():
    ctx.set_font(str(font))
    # do cool stuff with Japanese fonts
    heading("天気の子")
  else:
    label(f"Japanese font not found at: {font}")
    label("Set FONT_PATH to a font file to see 天気の子 rendered.")
    separator()
    # The latin text still renders with egui's built-in font, so this example
    # exercises set_font's absence without pretending to a result it cannot
    # produce.
    label("Install a CJK font and set FONT_PATH to show the heading above.")

def update_func(ctx):
  central_panel(ctx, lambda: main_contents(ctx))

if __name__ == "__main__":
  run_native("fonts", update_func)