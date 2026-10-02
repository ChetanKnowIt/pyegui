from pyegui import *

def main_contents():
    with Layout(LayoutType.VerticalCentered):
        heading("I'm horizontal")

def update_func(ctx):
    central_panel(ctx, main_contents)

if __name__ == "__main__":
    run_native("Hello World App", update_func)
  
