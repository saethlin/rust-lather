#from .lather import PySimulation as Simulation
#del lather  # This does not feel right

from lather._native import ffi, lib

def test():
    return lib.a_function_from_rust()

