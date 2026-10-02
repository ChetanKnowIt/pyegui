from pyegui import *

def main_contents(ctx):
  # provide absolute or relative path to the file
  ctx.set_font("NotoSansJP-VariableFont_wght.ttf")
  # do cool stuff with Japanese fonts
  heading("天気の子")

def update_func(ctx):
  central_panel(ctx, lambda: main_contents(ctx))

if __name__ == "__main__":
  run_native("fonts", update_func)

