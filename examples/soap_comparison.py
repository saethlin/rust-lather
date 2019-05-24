import numpy as np
import matplotlib.pyplot as plt
import lather

time = np.linspace(0, 25.05, 100, endpoint=False)

sim = lather.Simulation('soap.toml')

rv, ccfs = sim.observe_rv(time, 5293e-10, 5294e-10)

soap_rv = np.loadtxt('soap_rv.txt') * 1e3
plt.plot(time, rv, label='lather')
plt.plot(time, soap_rv, label='SOAP-2')
plt.legend()
plt.show()

plt.plot(time, rv - soap_rv)
plt.show()

