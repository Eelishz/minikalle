from tpot import TPOTRegressor
import numpy as np
from sklearn.model_selection import train_test_split

data_a = np.load("processed/dataset_A_4M_no_cap.npz")
X = data_a["arr_0"][:100_000]
y = data_a["arr_1"][:100_000]

X_train, X_test, y_train, y_test = train_test_split(X, y, train_size=0.75, test_size=0.25)

pipeline_optimizer = TPOTRegressor(generations=100, population_size=100, cv=5,
                                    random_state=42, verbosity=2, n_jobs=1)

pipeline_optimizer.fit(X_train, y_train)

print(pipeline_optimizer.score(X_test, y_test))

pipeline_optimizer.export('tpot_exported_pipeline.py')
