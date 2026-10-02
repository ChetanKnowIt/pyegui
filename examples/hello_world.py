from pyegui import *

def main_contents():
  heading("Hello, World!")

def update_func(ctx):
  central_panel(ctx, main_contents)

if __name__ == "__main__":
  run_native("Hello World App", update_func)
