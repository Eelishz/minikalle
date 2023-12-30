import tensorflow as tf
import keras
import numpy as np

data = np.load("processed/dataset_3M.npz")
X = data["arr_0"]
y = data["arr_1"]

model = keras.Sequential(
    [
        keras.layers.Input(shape=(768), name='data_in'),
        keras.layers.Dense(512, activation='relu'),
        keras.layers.Dense(256, activation='relu'),
        keras.layers.Dense(32, activation='relu'),
        keras.layers.Dense(1, name='data_out'),
    ]
)

callback = tf.keras.callbacks.EarlyStopping(monitor='loss')

batch_size = 512
epochs = 64

opt = keras.optimizers.Adam()
model.compile(loss='mean_squared_error', optimizer=opt, metrics=['accuracy'])

history = model.fit(X, y, batch_size=batch_size, epochs=epochs, validation_split=0.1, callbacks=[callback])

model.save("model")
