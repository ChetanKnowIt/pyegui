from pyegui import *
import logging

FORMAT = '%(levelname)s %(name)s %(asctime)-15s %(filename)s:%(lineno)d %(message)s'
logging.basicConfig(format=FORMAT)
logging.getLogger().setLevel(logging.DEBUG)

def main_contents(ctx):
    with Group():
        heading("Debugging your mom")
        heading("I'm pretty good at that")
    heading("I'm pretty good at that")



def update_func(ctx):
    central_panel(ctx, lambda: main_contents(ctx))

run_native("Debug", update_func)

