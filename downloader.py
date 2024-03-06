#!/bin/python3
import os
import sys
import requests


dir = sys.argv[1]


# Check if directory exists
if not os.path.exists(dir):
    os.mkdir(dir)

# Read each line in 2grams.txt
with open(dir + ".txt", "r") as file:
    for line in file:
        line = line.strip()
        file_name = os.path.basename(line)

        # Check if file already exists
        if not os.path.exists(file_name):
            # Use requests to download the file
            print(f"Downloading {file_name}")
            response = requests.get(line)
            with open(file_name, "wb") as new_file:
                new_file.write(response.content)
        else:
            print(f"File {file_name} already exists. Skipping download.")
