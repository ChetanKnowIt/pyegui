from pyegui import *


def main_contents(ctx):
    hyperlink_to("pyeguion GitHub", "https://github.com/GachiLord/pyegui")

    if button_clicked("Open url"):
        ctx.open_url("https://github.com")

def update_func(ctx):
    central_panel(ctx, lambda: main_contents(ctx))

if __name__ == "__main__":
    run_native("Hello World App", update_func)
  
