#!/usr/bin/env bash

export LC_ALL=C.UTF-8

# Update system and install necessary dependencies
sudo apt-get update
sudo apt-get install -y protobuf-compiler

# Install rustup, the Rust toolchain installer
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Set the PATH to include .cargo/bin
source $HOME/.cargo/env

# Install the minimum supported Rust version (MSRV)
rustup install 1.41.1

# Install a newer version of Rust for development/testing
rustup install 1.68.0

echo "Environment setup completed."
