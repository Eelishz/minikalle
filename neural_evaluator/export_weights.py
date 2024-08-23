from torch import load

SCALES = [
    32,
    64,
    128,
    128,
    64,
]

state = load("value.pth")

i = 0

for key in state:
    layer = state[key]
    if key.endswith("weight"):
        with open(f"W{i}.in", "w") as f:
            f.write("[")
            for w in layer.flatten():
                val = int(w*SCALES[i])
                val = max(val, -128)
                val = min(val, 127)
                f.write(f"{val},")
            f.write("]")
    elif key.endswith("bias"):
        with open(f"B{i}.in", "w") as f:
            f.write("[")
            bs = layer
            for b in bs:
                val = int(b*SCALES[i])
                val = max(val, -128)
                val = min(val, 127)
                f.write(f"{val},")
            f.write("]")

        i += 1
