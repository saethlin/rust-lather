import numpy as np
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
import lather

f = 40  #set to 1 to see a single rotation period, >1 for more than one
time = np.linspace(0, f*30., f*30)
print(time)

##randomized star spots, fill 
sim = lather.Simulation('sun.toml')
print(sim)

flux_b = sim.observe_flux(time, 4000e-10, 5000e-10)
flux_r = sim.observe_flux(time, 7000e-10, 8000e-10)

rv_b, bisectors_b = sim.observe_rv(time, 4000e-10, 5000e-10)
rv_r, bisectors_r = sim.observe_rv(time, 7000e-10, 8000e-10)


plt.subplot(211)
plt.plot(time, flux_r,'r')
plt.plot(time, flux_b,'b')
plt.title('Flux')

plt.subplot(212)
plt.plot(time, rv_r,'r')
plt.plot(time, rv_b,'b')
plt.title('RV')

plt.tight_layout()

plt.gcf().savefig('test.png')

'''
plt.subplot(211)
plt.plot(time,flux_b-flux_r)
plt.title('Delta Flux')

plt.subplot(212)
plt.plot(time, rv_b-rv_r)
plt.title('Delta RV')

plt.tight_layout()

plt.show()
'''
