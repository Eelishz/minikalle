import tensorflow as tf

SCALE = 128

model = tf.keras.models.load_model("model")

for i, layer in enumerate(model.layers):
    with open(f"W{i}.in", "w") as f:
        f.write("[")
        ws = layer.get_weights()[0].flatten()
        for w in ws:
            f.write(f"{int(w*SCALE)},")
        f.write("]")

    with open(f"B{i}.in", "w") as f:
        f.write("[")
        bs = layer.get_weights()[1].flatten()
        for b in bs:
            f.write(f"{int(b*SCALE)},")
        f.write("]")
