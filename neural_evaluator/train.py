import numpy as np
import torch
import torch.nn as nn
from torch.utils.data import Dataset
from torch import optim

class ChessDataset(Dataset):
    def __init__(self):
        dat = np.load("processed/dataset_10M.npz")
        self.X = dat['arr_0']
        self.Y = dat['arr_1']
        print("loaded", self.X.shape, self.Y.shape)

    def __len__(self):
        return self.X.shape[0]

    def __getitem__(self, idx):
        return (self.X[idx], self.Y[idx])

class Model(nn.Module):
    def __init__(self, input_size, hidden_size, output_size):
        super(Model, self).__init__()
        self.fc1 = nn.Linear(input_size, hidden_size)
        self.relu = nn.ReLU()
        self.dropout = nn.Dropout(p=0.8)
        self.fc2 = nn.Linear(hidden_size, output_size)

    def forward(self, x):
        x = self.dropout(x)
        x = self.fc1(x)
        x = self.relu(x)
        x = self.dropout(x)
        x = self.fc2(x)
        return x
if __name__ == "__main__":
    torch.set_num_threads(32)

    chess_dataset = ChessDataset()
    train_loader = torch.utils.data.DataLoader(chess_dataset, batch_size=2048, shuffle=True)
    model = Model(770, 20, 1)
    optimizer = optim.Adam(model.parameters())
    criterion = nn.MSELoss() 

    for epoch in range(100):
        for batch_idx, (data, target) in enumerate(train_loader):
            target = target.unsqueeze(-1)
            data = data.float()
            target = target.float()

            optimizer.zero_grad()
            output = model(data)

            loss = criterion(output, target)
            loss.backward()
            optimizer.step()

            print(f"Epoch [{epoch+1}/{batch_idx}], Loss: {loss.item():.4f}")

        torch.save(model.state_dict(), "value.pth")
