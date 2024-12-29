#!/bin/bash
set -e

# Function to cleanup processes on exit
cleanup() {
    echo "Cleaning up..."
    pkill -f "antnode" || true
    pkill -f "evm-testnet" || true
    rm -rf "$HOME/Library/Application Support/autonomi/node" || true
}

# Function to check if a port is in use
check_port() {
    nc -z localhost $1 >/dev/null 2>&1
    return $?
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

# Build ant-node with test feature (which includes local)
echo "Building ant-node..."
cargo build -p ant-node --features test

# Build evm-testnet
echo "Building evm-testnet..."
cargo build -p evm-testnet

# Kill any existing antnode processes
pkill -f "antnode" || true
rm -rf "$HOME/Library/Application Support/autonomi/node" || true

# Check if EVM network is already running
if check_port 4343 || check_port 8545; then
    echo "EVM network is already running, using existing instance..."
else
    echo "Starting new EVM testnet..."
    RPC_PORT=4343 ./target/debug/evm-testnet --genesis-wallet 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 &
    EVM_PID=$!
    # Wait for EVM testnet to be ready
    sleep 5
fi

# Run the tests with test feature
echo "Running tests..."
RUST_LOG=trace cargo test -p autonomi --features test -- --nocapture

# Wait for tests to complete
wait 