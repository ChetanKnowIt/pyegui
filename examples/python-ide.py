from pyegui import *
import time

buf = Str("")
output = Str("")

def main_contents(ctx):
  code_editor(buf)  
  separator()
  label(output.value)
  if button_clicked("Run"):
    output.value = ""
    try:
      t1 = time.time_ns()
      exec(buf.value)
      t2 = time.time_ns()
      output.value = f"Run {t2-t1}ns"
    except Exception as e:
      output.value = str(e)

def update_func(ctx):
  central_panel(ctx, lambda: main_contents(ctx))

if __name__ == "__main__":
  run_native("Python IDE", update_func)
