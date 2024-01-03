import tensorflow as tf

SCALE = 100

model = tf.keras.models.load_model("model")

for layer in model.layers:
    print(layer.get_config())

    print("WEIGHTS")

    ws = layer.get_weights()[0].flatten()
    for w in ws:
        print(f"{int(w*SCALE)}, ", end="")

    print("\nBIASES")

    bs = layer.get_weights()[1].flatten()
    for b in bs:
        print(f"{int(b*SCALE)}, ", end="")

    print("\n---")
