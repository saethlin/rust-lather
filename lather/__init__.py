import numpy as np
from lather._native import ffi, lib


class Simulation:
    def __init__(self, filename):
        self._native = lib.simulation_new(filename.encode())

    def __repr__(self):
        return ffi.string(lib.simulation_tostring(self._native)).decode('utf-8')

    # there is potential to double-free here because I'm not making this pointer null
    # I think... not sure if the del saves us
    def __del__(self):
        lib.simulation_free(self._native)
        del self._native

    def observe_flux(self, time, wave_start, wave_end):
        time_ptr = ffi.cast("double *", time.ctypes.data)
        flux_ptr = lib.simulation_observe_flux(self._native, time_ptr, time.size, wave_start, wave_end)
        flux_buffer = ffi.buffer(flux_ptr, time.size*8)
        return np.frombuffer(flux_buffer, dtype=float, count=time.size)

    def observe_rv(self, time, wave_start, wave_end):
        time_ptr = ffi.cast("double *", time.ctypes.data)

        rv_ptr = lib.simulation_observe_rv(self._native, time_ptr, time.size, wave_start, wave_end)
        rv_buffer = ffi.buffer(rv_ptr, time.size*8)
        rv_array = np.frombuffer(rv_buffer, dtype=float, count=time.size)

        bisector_ptr = rv_ptr + (time.size)
        bisector_buffer = ffi.buffer(bisector_ptr, 8*(time.size * 1_000))
        bisector_array = np.frombuffer(bisector_buffer, dtype=float, count=time.size*1_000)
        bisector_array.shape = (time.size, 1_000)

        return rv_array, bisector_array
