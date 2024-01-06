import tensorflow as tf
from tensorflow.keras.models import Sequential
from tensorflow.keras.layers import Dense
from tensorflow.keras.callbacks import TensorBoard
from tensorflow import keras
from tensorflow.keras import regularizers 
import numpy as np
from datetime import datetime

data = np.load("processed/dataset_3M_no_cap.npz")
X = data["arr_0"]
y = data["arr_1"]

dense_0_sizes = [8,]
dense_1_sizes = [4,]
dense_2_sizes = [0,]

for dense_2 in dense_2_sizes:
    for dense_1 in dense_1_sizes:
        for dense_0 in dense_0_sizes:
            if dense_1 > dense_0 or dense_2 > dense_1:
                continue

            model_name = f"{dense_0}-{dense_1}-{dense_2}-{datetime.now().strftime('%Y%m%d-%H%M%S')}"

            log_dir = "logs/fit/" + model_name
            tensorboard_callback = TensorBoard(log_dir=log_dir, histogram_freq=1)

            model = keras.Sequential()

            model.add(keras.layers.Input(shape=(770), name='data_in'))
            if dense_0 != 0:
                model.add(keras.layers.Dense(
                    dense_0,
                    activation='relu',
                    # kernel_regularizer=regularizers.L1L2(l1=1e-7, l2=1e-7),
                    # bias_regularizer=regularizers.L2(1e-7),
                    # activity_regularizer=regularizers.L2(1e-7)
                ))
            if dense_1 != 0:
                model.add(keras.layers.Dense(
                    dense_1,
                    activation='relu',
                    # kernel_regularizer=regularizers.L1L2(l1=1e-7, l2=1e-7),
                    # bias_regularizer=regularizers.L2(1e-7),
                    # activity_regularizer=regularizers.L2(1e-7)
                ))
            if dense_2 != 0:
                model.add(keras.layers.Dense(
                    dense_2,
                    activation='relu',
                    # kernel_regularizer=regularizers.L1L2(l1=1e-7, l2=1e-7),
                    # bias_regularizer=regularizers.L2(1e-7),
                    # activity_regularizer=regularizers.L2(1e-7)
                ))
            model.add(keras.layers.Dense(1, name='data_out'))

            early_stopping = tf.keras.callbacks.EarlyStopping(monitor='loss')
            tensorboard_callback = TensorBoard(log_dir=log_dir, histogram_freq=1)

            batch_size = 1024
            epochs = 1024

            opt = keras.optimizers.Adam()
            model.compile(loss='mean_squared_error', optimizer=opt)

            print(model_name)
            history = model.fit(X, y, batch_size=batch_size, epochs=epochs, validation_split=0.1, callbacks=[early_stopping, tensorboard_callback])

            model.save("model")

            keras.backend.clear_session()

