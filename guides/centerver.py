from pyegui import *

def update_func(ctx):
    with Layout(LayoutType.VerticalCentered):
        heading("I'm horizontal")

if __name__ == "__main__":
    run_native("Hello World App", update_func)
  
