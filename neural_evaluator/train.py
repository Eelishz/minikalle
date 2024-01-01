import tensorflow as tf
from tensorflow.keras.models import Sequential
from tensorflow.keras.layers import Dense
from tensorflow.keras.callbacks import TensorBoard
import keras
import numpy as np
from datetime import datetime

# Data generation consumes so much memeory that processed data is generated
# in two batches and appended together.
# This is probably not a good solution but it seems to work.

data_a = np.load("processed/dataset_A_4M.npz")
X_a = data_a["arr_0"]
y_a = data_a["arr_1"]

data_b = np.load("processed/dataset_B_4M.npz")
X_b = data_b["arr_0"]
y_b = data_b["arr_1"]

X = np.append(X_a, X_b, axis=0)
y = np.append(y_a, y_b, axis=0)

# X = X_a
# y = y_a

# cleanup temp objects

del data_a
del data_b
del X_a
del y_a
del X_b
del y_b

dense_0_sizes = [0, 128, 64, 32]
dense_1_sizes = [0, 128, 64, 32, 16]
dense_2_sizes = [0, 64, 32, 16, 8]
dropout_freqs = [0.0, 0.5, 0.2]

for dropout_freq in dropout_freqs:
    for dense_2 in dense_2_sizes:
        for dense_1 in dense_1_sizes:
            for dense_0 in dense_0_sizes:
                model_name = f"{dense_0}-{dense_1}-{dense_2}-{dropout_freq}-{datetime.now().strftime('%Y%m%d-%H%M%S')}"
                log_dir = "logs/fit/" + model_name
                tensorboard_callback = TensorBoard(log_dir=log_dir, histogram_freq=1)

                model = keras.Sequential()

                model.add(keras.layers.Input(shape=(768), name='data_in'))
                model.add(keras.layers.Dropout(dropout_freq))
                if dense_0 != 0:
                    model.add(keras.layers.Dense(dense_0, activation='relu'))
                    model.add(keras.layers.Dropout(dropout_freq))
                if dense_1 != 0:
                    model.add(keras.layers.Dense(dense_1, activation='relu'))
                    model.add(keras.layers.Dropout(dropout_freq))
                if dense_2 != 0:
                    model.add(keras.layers.Dense(dense_2, activation='relu'))
                    model.add(keras.layers.Dropout(dropout_freq))
                model.add(keras.layers.Dense(1, name='data_out'))

                early_stopping = tf.keras.callbacks.EarlyStopping(monitor='loss')
                tensorboard_callback = TensorBoard(log_dir=log_dir, histogram_freq=1)

                batch_size = 1024
                epochs = 10

                opt = keras.optimizers.Adam()
                model.compile(loss='mean_squared_error', optimizer=opt, metrics=['accuracy'])

                print(model_name)
                history = model.fit(X, y, batch_size=batch_size, epochs=epochs, validation_split=0.2, callbacks=[early_stopping, tensorboard_callback])

                # Cleanup allocated objects.
                # Causes a memory leak for some reason if these are not explicitly de-allocated.
                del model
                del early_stopping
                del tensorboard_callback
                del opt
                del history

# model.save("model")
