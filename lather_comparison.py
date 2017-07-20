import os
import numpy as np
import matplotlib.pyplot as plt

os.chdir('/home/ben/lather/cmake-build-debug')
os.system('cmake ..')
os.system('make -j4')
os.system('./lather > /home/ben/rather/lather_output.txt')

os.chdir('/home/ben/rather')
os.system('cargo build --release')
os.system('target/release/rather > /home/ben/rather/rather_output.txt')

lather = np.loadtxt('lather_output.txt')
rather = np.loadtxt('rather_output.txt')

plt.plot(lather - rather)
plt.title('LATHER - RATHER')
plt.show()
