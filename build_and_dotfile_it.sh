#!/bin/bash

# This script builds galdrar and then copies the exec to ~/.config/niri

cargo build --release;
cp ~/Repositories/galdrar/target/release/galdrar ~/.config/niri/;
echo -e "\n\033[0;32m\033[1mgaldrar built and copied to ~/.config/niri\033[0m";