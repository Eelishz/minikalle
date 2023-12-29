import tensorflow as tf

model = tf.keras.models.load_model("model")

for layer in model.layers:
    print(layer.get_config())

    print("WEIGHTS")

    ws = layer.get_weights()[0].flatten()
    for w in ws:
        print(f"{int(w*10_000)}, ", end="")

    print("\nBIASES")

    bs = layer.get_weights()[1].flatten()
    for b in bs:
        print(f"{int(b*10_000)}, ", end="")

    print("\n---")
