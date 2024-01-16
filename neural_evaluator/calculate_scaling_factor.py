from torch import load
from math import floor

state = load("value.pth")

for key in state:
    layer = state[key]

    lmax = max(layer.flatten())
    lmin = min(layer.flatten())

    scale_factor = floor(127/(max(lmax, -lmin)))

    print(scale_factor)
