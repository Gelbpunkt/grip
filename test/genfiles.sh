#!/usr/bin/env bash
for i in {0..100}; do
    mkdir "dir$i"
    cd "dir$i"
    for u in {0..100}; do
        base64 /dev/urandom | head -c 10000000 > "file$u.txt"
    done
    cd ..
done
