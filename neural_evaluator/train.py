import numpy as np
import pandas as pd
import torch
import torch.nn as nn
import torch.nn.functional as F
from torch.utils.data import Dataset
from torch import optim
import pandas as pd

class ChessDataset(Dataset):
    def __init__(self):
        df = pd.read_csv("../data6M.csv", header=None, dtype=np.int16, engine="c")
        self.X = df.iloc[:, 1:].to_numpy(dtype=np.int16)
        self.Y = df.iloc[:, 0].to_numpy(dtype=np.int16)

        print("loaded", self.X.shape, self.Y.shape)

    def __len__(self):
        return self.X.shape[0]

    def __getitem__(self, idx):
        return self.X[idx], self.Y[idx]

class Model(nn.Module):
    def __init__(self):
        super(Model, self).__init__()
        self.fc1 = nn.Linear(770, 32)
        self.fc2 = nn.Linear(32, 32)
        self.fc3 = nn.Linear(32, 16)
        self.fc4 = nn.Linear(16, 16)
        self.fc5 = nn.Linear(16, 1)

    def forward(self, x):
        x = F.relu(self.fc1(x))
        
        x = F.relu(self.fc2(x))
        
        x = F.relu(self.fc3(x))
        
        x = F.relu(self.fc4(x))
        
        x = self.fc5(x)

        return x


if __name__ == "__main__":
    torch.set_num_threads(32)

    chess_dataset = ChessDataset()
    train_loader = torch.utils.data.DataLoader(
            chess_dataset, 
            batch_size=1024,
            shuffle=True,
            num_workers=8)
    model = Model()
    optimizer = optim.Adam(model.parameters())
    criterion = nn.MSELoss()

    for epoch in range(10_000):
        running_loss = 0.0
        for batch_idx, (data, target) in enumerate(train_loader):
            target = target.unsqueeze(-1)
            data = data.float()
            target = target.float()

            optimizer.zero_grad()
            output = model(data)

            loss = criterion(output, target)
            loss.backward()
            optimizer.step()

            running_loss += loss.item()
            if batch_idx % 20 == 19:
                print(f"Epoch [{epoch+1}/{batch_idx+1}], Loss: {running_loss / 20:.4f}")
                running_loss = 0.0

        torch.save(model.state_dict(), "value.pth")
