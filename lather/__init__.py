import numpy as np
import scipy.interpolate
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
        # Total size is rv and respective ccfs
        data_buffer = ffi.buffer(data_ptr, 8*time.size + 8*(time.size * 401))
        data = np.frombuffer(data_buffer, dtype=np.float64)

        rv = data[:time.size].copy()
        bisectors = data[time.size:].reshape(time.size, 401).copy()

        return rv, bisectors


def compute_bisector(rv, ccf, size=1000):
    # Split into red and blue parts
    indmax = ccf.argmax()
    red_ccf = ccf[indmax:][::-1]
    red_rv = rv[indmax:][::-1]
    blue_ccf = ccf[:indmax]
    blue_rv = rv[:indmax]

    red_mask = (red_ccf < 0.9) & (red_ccf > 0.1)
    red_ccf = red_ccf[red_mask]
    red_rv = red_rv[red_mask]

    blue_mask = (blue_ccf < 0.9) & (blue_ccf > 0.1)
    blue_ccf = blue_ccf[blue_mask]
    blue_rv = blue_rv[blue_mask]

    # Build spline coefficients for interpolation
    ccf_eval = np.linspace(0.0, 1.0, size)
    try:
        red_tck = scipy.interpolate.splrep(red_ccf, red_rv)
        blue_tck = scipy.interpolate.splrep(blue_ccf, blue_rv)
        bisector = (scipy.interpolate.splev(ccf_eval, red_tck) +
                    scipy.interpolate.splev(ccf_eval, blue_tck)) / 2
    except (ValueError, TypeError):
        bisector = ccf_eval * np.nan

    return bisector

