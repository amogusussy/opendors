#!/bin/bash

set -e
name="dors"
path="./target/debug/$name"

cargo build
echo "chowning my shit"
sudo chown root:root "$path"
sudo chmod u+s "$path"
