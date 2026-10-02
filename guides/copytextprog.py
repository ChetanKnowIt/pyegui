from pyegui import *

# This item is to big to be inserted into clipboard
# text_to_copy = "your mom" 

text_to_copy = "smol item"

def main_contents(ctx):
    if button_clicked("Copy"):
        ctx.copy_text(text_to_copy)

def update_func(ctx):
    central_panel(ctx, lambda: main_contents(ctx))

if __name__ == "__main__":
    run_native("Hello World App", update_func)
  
