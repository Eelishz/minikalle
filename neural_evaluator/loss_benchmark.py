# The point of this script is to figure out what a
# "good" loss would be.
# 
# The program checks the MSE loss of a 
# linear regression.

# TODO: add more models including a classical
# evaluation function.

import numpy as np
from sklearn.linear_model import Ridge, SGDRegressor
from sklearn.metrics import mean_squared_error
from sklearn.model_selection import train_test_split

data_a = np.load("processed/dataset_A_4M_no_cap.npz")

X = data_a["arr_0"][:1_000_000]
y = data_a["arr_1"][:1_000_000]

X_train, X_test, y_train, y_test = train_test_split(
        X, y, test_size=0.33, random_state=42)

ridge = Ridge().fit(X_train, y_train)
ridge_pred = ridge.predict(X_test)
print("Ridge done!")

sgd = SGDRegressor().fit(X_train, y_train)
sgd_pred = sgd.predict(X_test)
print("SGD done!")

print(f"Guessing that white wins MSE: {mean_squared_error(np.ones(len(y_test)), y_test)}")
print(f"Guessing tie             MSE: {mean_squared_error(np.zeros(len(y_test)), y_test)}")
print(f"Guess the average        MSE: {mean_squared_error(np.zeros(len(y_test)) + np.mean(y_train), y_test)}")
print(f"Ridge regression         MSE: {mean_squared_error(ridge_pred, y_test)}")
print(f"SGD regression           MSE: {mean_squared_error(sgd_pred, y_test)}")

# print the coefs of the regression

# for coef in reg.coef_:
#     print(float(coef), end=", ")
# print()
