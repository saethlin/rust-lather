import numpy as np
import matplotlib.pyplot as plt
import lather

time = np.linspace(0, 25.05, 100)
sim = lather.Simulation('sun.cfg')
rv, bisectors = sim.observe_rv(time, 4000e-10, 7000e-10)
plt.plot(time, rv)
plt.show()
