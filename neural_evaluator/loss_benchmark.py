# The point of this script is to figure out what a
# "good" loss would be.
# 
# The program checks the MSE loss of a 
# linear regression.

# TODO: add more models including a classical
# evaluation function.

import numpy as np
from sklearn.linear_model import Ridge
from sklearn.metrics import mean_squared_error
from sklearn.model_selection import train_test_split

data_a = np.load("processed/dataset_A_4M_no_cap.npz")

X = data_a["arr_0"][:100_000]
y = data_a["arr_1"][:100_000]

X_train, X_test, y_train, y_test = train_test_split(
        X, y, test_size=0.33, random_state=42)

reg = Ridge().fit(X_train, y_train)

pred = reg.predict(X_test)

print(f"Guessing that white wins MSE: {mean_squared_error(np.ones(len(y_test)), y_test)}")
print(f"Linear regression        MSE: {mean_squared_error(pred, y_test)}")
