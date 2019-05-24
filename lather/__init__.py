import numpy as np
import scipy.interpolate
import scipy.signal
import toml
from lather._native import ffi, lib

# This isn't actually exactly equal to what's in the resource file
# but because of a bug in numpy; there's a few f64 epsilon of difference
RV_FOR_CCFS = np.linspace(-20e3, 20e3, num=401, endpoint=True, dtype=np.float64)


class Simulation:
    def __init__(self, filename):
        error_ptr = ffi.new("char **")
        self._native = lib.simulation_new(filename.encode(), error_ptr)

        if self._native == ffi.NULL:
            error = ffi.string(error_ptr[0]).decode("utf-8")
            raise RuntimeError(error)

        config = toml.load(filename)
        self.instrument_profile = None
        instrument_resolution = config.get("instrument_resolution", None)
        if instrument_resolution is not None:
            c = 299792458.0  # speed of light in m/s
            instrument_profile_fwhm = c / instrument_resolution
            instrument_profile_sigma = instrument_profile_fwhm / (
                2 * np.sqrt(2 * np.log(2))
            )
            self.instrument_profile = np.exp(
                -RV_FOR_CCFS ** 2 / (2 * (instrument_profile_sigma) ** 2)
            )
        
        quiet_ccf = np.empty(401, dtype=np.float64)
        quiet_ccf_ptr = ffi.cast("double *", quiet_ccf.ctypes.data)
        lib.simulation_get_quiet_ccf(self._native, quiet_ccf_ptr)
        
        quiet_ccf = self._apply_resolution_correction(quiet_ccf)
        self.zero_rv = compute_rv(quiet_ccf)

    def __repr__(self):
        return ffi.string(lib.simulation_tostring(self._native)).decode("utf-8")

    # there is potential to double-free here because I'm not making this pointer null
    # I think... not sure if the del saves us
    def __del__(self):
        lib.simulation_free(self._native)
        del self._native

    def observe_flux(self, time, wave_start, wave_end):
        time_ptr = ffi.cast("double *", time.ctypes.data)
        flux = np.empty_like(time)
        flux_ptr = ffi.cast("double *", flux.ctypes.data)
        lib.simulation_observe_flux(
            self._native, time_ptr, time.size, wave_start, wave_end, flux_ptr
        )
        return flux

    def observe_rv(self, time, wave_start, wave_end):
        time_ptr = ffi.cast("double *", time.ctypes.data)

        ccfs = np.empty((time.size, 401))
        ccfs_ptr = ffi.cast("double *", ccfs.ctypes.data)

        lib.simulation_observe_rv(
            self._native, time_ptr, time.size, wave_start, wave_end, ccfs_ptr
        )

        rv = np.empty(time.size)
        for i in range(time.size):
            ccfs[i] = self._apply_resolution_correction(ccfs[i])
            rv[i] = compute_rv(ccfs[i]) - self.zero_rv

        return rv, ccfs

    # This would be implemented in the Rust part but I can't make the FFT work
    def _apply_resolution_correction(self, ccf):
        ccf = ccf.copy()
        ccf *= -1;
        ccf -= ccf.min()
        ccf /= ccf.max()
        if self.instrument_profile is None:
            return ccf
        else:
            return scipy.signal.convolve(ccf, self.instrument_profile, mode="same")

    def draw_bgr(self, time, out=None):
        if out is None:
            image = np.empty((1000, 1000, 3), dtype=np.uint8)
        elif (out.dtype != np.uint8) or (out.shape != (1000, 1000, 3)):
            raise ValueError(
                "image argument must be an array of np.uint, with shape (1000, 1000, 3)"
            )
        else:
            image = out

        image_ptr = ffi.cast("char *", image.ctypes.data)

        lib.simulation_draw_bgr(self._native, time, image_ptr)
        return image


# TODO: This is SOAP-2.0's RV calculation and IMO the technique is wrong
def compute_rv(ccf):
    import scipy.optimize
    def gauss(x, a, b, c, d):
        return a * np.exp(-(x - b)**2 / (2 * c**2)) + d

    p, cov = scipy.optimize.curve_fit(gauss, RV_FOR_CCFS, ccf, p0=[1.0, 0.0, 2000.0, 0.0])
    return p[1]


def compute_bisector(ccf, size=1000):
    """
    Compute the line bisector values for a given Lather CCF

    The CCF values for the returned bisector are arbitrary in some sense but can be safely treated as np.linspace(0.0, 1.0, size)
    But keep in mind some papers quote line bisectors with the top and bottom 10% removed, and use these bisectors to calculate the bisector inverse slope
    """
    rel_max_inds = scipy.signal.argrelmax(ccf)[0]
    max_ind = np.argmax(ccf[rel_max_inds])
    ccf_peak = rel_max_inds[max_ind]

    ccf = ccf.copy()
    ccf -= ccf.min()
    ccf /= ccf[ccf_peak]

    # Split into red and blue parts
    red_end = scipy.signal.argrelmin(ccf[ccf_peak:])[0][0] + ccf_peak
    red_ccf = ccf[ccf_peak:red_end][::-1]
    red_rv = RV_FOR_CCFS[ccf_peak:red_end][::-1]

    blue_end = scipy.signal.argrelmin(ccf[:ccf_peak])[0][-1]
    blue_ccf = ccf[blue_end:ccf_peak]
    blue_rv = RV_FOR_CCFS[blue_end:ccf_peak]

    red_mask = red_ccf > 0.025
    red_ccf = red_ccf[red_mask]
    red_rv = red_rv[red_mask]

    blue_mask = blue_ccf > 0.025
    blue_ccf = blue_ccf[blue_mask]
    blue_rv = blue_rv[blue_mask]

    # Build spline coefficients for interpolation
    ccf_eval = np.linspace(0.0, 1.0, size)

    red_tck = scipy.interpolate.splrep(red_ccf, red_rv)
    blue_tck = scipy.interpolate.splrep(blue_ccf, blue_rv)

    bisector = (
        scipy.interpolate.splev(ccf_eval, red_tck)
        + scipy.interpolate.splev(ccf_eval, blue_tck)
    ) / 2

    return bisector
