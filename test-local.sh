#!/bin/bash
set -e

# Function to cleanup processes on exit
cleanup() {
    echo "Cleaning up..."
    pkill -f "antnode" || true
    pkill -f "evm-testnet" || true
    rm -rf "$HOME/Library/Application Support/autonomi/node" || true
}

# Register the cleanup function to run on script exit
trap cleanup EXIT

# Install Foundry if not already installed
if ! command -v anvil &> /dev/null; then
    echo "Installing Foundry..."
    curl -L https://foundry.paradigm.xyz | bash
    source "$HOME/.bashrc"
    foundryup
fi

# Build ant-node with local feature
echo "Building ant-node with local feature..."
cargo build -p ant-node --features local

# Build evm-testnet
echo "Building evm-testnet..."
cargo build -p evm-testnet

# Start the EVM testnet in the background
echo "Starting EVM testnet..."
./target/debug/evm-testnet --genesis-wallet 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 &
EVM_PID=$!

# Wait for EVM testnet to be ready
sleep 5

# Run the tests with local feature
echo "Running tests..."
RUST_LOG=trace cargo test -p autonomi --features local -- --nocapture

# Wait for tests to complete
wait 