import numpy as np
import matplotlib.pyplot as plt
import rather

time = np.linspace(0, 25.05, 100)
sim = rather.Simulation('sun.cfg')
rv, bisectors = sim.observe_rv(time, 4000e-10, 7000e-10)
plt.plot(time, rv)
plt.show()
