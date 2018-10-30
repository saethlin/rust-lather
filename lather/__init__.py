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
        return np.frombuffer(flux_buffer, dtype=float, count=time.size).copy()

    def observe_rv(self, time, wave_start, wave_end):
        time_ptr = ffi.cast("double *", time.ctypes.data)

        data_ptr = lib.simulation_observe_rv(self._native, time_ptr, time.size, wave_start, wave_end)
        # Total size is rv and bisectors
        data_buffer = ffi.buffer(data_ptr, 8*time.size + 8*(time.size * 1_000))
        data = np.frombuffer(data_buffer, dtype=np.float64)

        rv = data[:time.size].copy()
        bisectors = data[time.size:].reshape(time.size, 1_000).copy()

        return rv, bisectors
