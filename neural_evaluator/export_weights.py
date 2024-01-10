from torch import load

SCALE = 128

state = load("value.pth")

i = 0

for key in state:
    layer = state[key]
    if key.endswith("weight"):
        with open(f"W{i}.in", "w") as f:
            f.write("[")
            for w in layer.flatten():
                f.write(f"{int(w*SCALE)},")
            f.write("]")
    elif key.endswith("bias"):
        with open(f"B{i}.in", "w") as f:
            f.write("[")
            bs = layer
            for b in bs:
                f.write(f"{int(b*SCALE)},")
            f.write("]")

        i += 1
