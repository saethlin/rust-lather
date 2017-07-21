import numpy as np
from . import _rather

class Simulation:
    def __init__(self, filename):
        self._simulation = _rather.Simulation(filename)
    
    def __repr__(self):
        return self._simulation.__repr__()
    
    def __str__(self):
        return self._simulation.__str__()

    def observe_flux(self, time, wavelength_min, wavelength_max):
        return np.array(self._simulation.observe_flux(list(time), wavelength_min, wavelength_max))

    def observe_rv(self, time, wavelength_min, wavelength_max):
        obs_list = self._simulation.observe_rv(list(time), wavelength_min, wavelength_max)
        return np.array([o[0] for o in obs_list]), np.array([o[1] for o in obs_list])
