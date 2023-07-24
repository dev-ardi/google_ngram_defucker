#!/bin/bash

dir="2grams"

# check if directory exists
if [ ! -d "$dir" ]; then
  mkdir "$dir"
fi

# read each line in url.txt
while IFS= read -r line
do
    file="$(basename "$line")"
    # check if file already exists

    if [ ! -f "$file" ]; then
        # use curl to download
        echo "downloading $file"
        curl -L "$line" -o "$file"
    else
        echo "File $file already exists. Skipping download."
    fi
done < "2grams.txt"
