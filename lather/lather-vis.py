import argparse
import lather
import cv2
import numpy as np

parser = argparse.ArgumentParser("Lather Visualizer")
parser.add_argument("config_file")
parser.add_argument("-o", "--output", default="lather.avi", help="output filename, should probably end with .avi")
parser.add_argument("-fps", "--framerate", type=int, default=60, help="framerate for the video")
parser.add_argument("-d", "--duration", type=float, default=25.05, help="durations (days) of the simulation")
parser.add_argument("-fc", "--frame-count", type=int, default=1000, help="number of frames in the video evenly spaced over the provided duration")
args = parser.parse_args()

sim = lather.Simulation(args.config_file)

writer = cv2.VideoWriter(args.output, cv2.VideoWriter_fourcc(*'DIVX'), args.framerate, (1000, 1000))

frame = np.ones((1000, 1000, 3), dtype=np.uint8)
import time
for t in np.linspace(0, args.duration, args.frame_count):
    frame = sim.draw_bgr(t, out=frame)
    writer.write(frame)

