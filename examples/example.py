import numpy as np
import matplotlib.pyplot as plt
import lather

time = np.linspace(0, 25.05, 100)

sim = lather.Simulation('random.toml')
print(sim)

flux = sim.observe_flux(time, 4000e-10, 7000e-10)
plt.plot(time, flux)
plt.title('A lather light curve')
plt.xlabel('Time (days)')
plt.ylabel('Relative flux')
plt.show()

rv, ccfs = sim.observe_rv(time, 4000e-10, 7000e-10)
plt.plot(time, rv)
plt.title('A lather RV curve')
plt.xlabel('Time (days)')
plt.ylabel('RV (m/s)')
plt.show()

plt.plot(np.linspace(-2e4, 2e4, 401), ccfs[50])
plt.title('A lather CCF')
plt.xlabel('Velocity (m/s)')
plt.show()

bisector = lather.compute_bisector(ccfs[50])
plt.plot(bisector, np.linspace(0.0, 1.0, bisector.size))
plt.title('A lather bisector')
plt.xlabel('Velocity (m/s)')
plt.show()

print(sim)
