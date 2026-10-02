from pyegui import *
import logging

FORMAT = '%(levelname)s %(name)s %(asctime)-15s %(filename)s:%(lineno)d %(message)s'
logging.basicConfig(format=FORMAT)
logging.getLogger().setLevel(logging.DEBUG)

def main_contents():
    heading("logging now")

def update_func(ctx):
    central_panel(ctx, main_contents)

run_native("logging", update_func)
