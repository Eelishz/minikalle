import numpy as np
import torch
import torch.nn as nn
import torch.nn.functional as F
from torch.utils.data import Dataset
from torch import optim
import pandas as pd
from tqdm import tqdm
import os
# from math import floor

class ChessDataset(Dataset):

    def __init__(self, path, total_rows, lines_per_file):
        self.path = path
        self.total_rows = total_rows
        self.files = []
        self.lines_per_file = lines_per_file
        self.cache = ["", None]

        for filename in os.listdir(self.path):
            file_path = os.path.join(self.path, filename)

            if os.path.isfile(file_path):
                self.files.append(file_path)

    
    def __len__(self):
        return self.total_rows

    def __getitem__(self, idx):
        chunk_idx = idx // self.lines_per_file
        file = self.files[chunk_idx]
        start = idx - chunk_idx * self.lines_per_file

        df = None

        if self.cache[0] == file:
            df = self.cache[1]
        else:
            df = pd.read_csv(
                file,
                header=None,
                dtype=np.int16,
                compression="gzip",
                # skiprows=start,
                # nrows=1,
                engine="c",
            )
            self.cache[0] = file
            self.cache[1] = df

        # Sometimes the dataframes are 1 shorter than they should be.
        # This might be an issue with gnu split

        # This means that on occaision a sample will be repeated in on epoch.

        dflen = len(df) - 1
        if start >= dflen:
            start = dflen % start

        X = df.to_numpy()[start, 1:]
        Y = df.to_numpy()[start, 0]

        return X, Y


class Model(nn.Module):
    def __init__(self):
        super(Model, self).__init__()
        self.fc1 = nn.Linear(770, 64)
        self.fc2 = nn.Linear(64, 64)
        self.fc3 = nn.Linear(64, 32)
        self.fc4 = nn.Linear(32, 32)
        self.fc5 = nn.Linear(32, 1)

    def forward(self, x):
        x = F.relu(self.fc1(x))
        
        x = F.relu(self.fc2(x))
        
        x = F.relu(self.fc3(x))
        
        x = F.relu(self.fc4(x))
        
        x = self.fc5(x)

        return x


if __name__ == "__main__":
    torch.set_num_threads(32)

    BATCH_SIZE = 50_000

    chess_dataset = ChessDataset("processed/", 70_970_000, 10000)
    train_loader = torch.utils.data.DataLoader(
            chess_dataset, 
            batch_size=BATCH_SIZE,
            shuffle=False,
            num_workers=16,
            prefetch_factor=4
    )
    model = Model()
    optimizer = optim.Adam(model.parameters())
    criterion = nn.MSELoss()

    for epoch in range(100):
        all_loss = 0.0
        num_loss = 0
        iter = tqdm(
            enumerate(train_loader), 
            total=len(chess_dataset)//BATCH_SIZE,
            ncols=80
        )
        for batch_idx, (data, target) in iter:
            target = target.unsqueeze(-1)
            data = data.float()
            target = target.float()

            optimizer.zero_grad()
            output = model(data)

            loss = criterion(output, target)
            loss.backward()
            optimizer.step()

            all_loss += loss.item()
            num_loss += 1
        
        print(f"Epoch [{epoch+1}], Loss: {all_loss / num_loss:.4f}")

        torch.save(model.state_dict(), "value.pth")
