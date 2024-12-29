#!/bin/bash
set -e

# Configuration
NODE_DATA_DIR="$HOME/Library/Application Support/autonomi/node"
CLIENT_DATA_DIR="$HOME/Library/Application Support/autonomi/client"
EVM_PORT=4343
EVM_RPC_URL="http://localhost:8545"
WALLET_ADDRESS="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
TOKEN_ADDRESS="0x5FbDB2315678afecb367f032d93F642f64180aa3"
LOG_LEVEL="info"
NODE_PORT=5000

# Helper Functions
cleanup() {
    echo "Cleaning up processes..."
    pkill -f "antnode" || true
    pkill -f "evm-testnet" || true
}

check_port() {
    local port=$1
    if lsof -i :$port > /dev/null; then
        echo "Port $port is already in use. Please choose a different port or stop the process using it."
        exit 1
    fi
}

install_foundry() {
    if ! command -v forge &> /dev/null; then
        echo "Installing Foundry..."
        curl -L https://foundry.paradigm.xyz | bash
        source $HOME/.bashrc
        foundryup
    fi
}

start_evm_testnet() {
    echo "Starting EVM testnet..."
    cd evm-testnet
    cargo run --release &
    cd ..
    sleep 2
}

start_local_node() {
    echo "Starting local node..."
    RUST_LOG=$LOG_LEVEL ./target/debug/antnode \
        --data-dir "$NODE_DATA_DIR" \
        --port $NODE_PORT \
        --features test \
        &
    sleep 2
}

build_binaries() {
    echo "Building ant-node..."
    cargo build -p ant-node --features test

    echo "Building evm-testnet..."
    cargo build -p evm-testnet --release

    echo "Building ant CLI..."
    cargo build -p ant
}

print_dev_info() {
    echo "
Development Environment Ready!

Network Information:
------------------
Node Endpoint: /ip4/127.0.0.1/udp/$NODE_PORT/quic-v1
EVM RPC URL: $EVM_RPC_URL
Wallet Address: $WALLET_ADDRESS
Token Address: $TOKEN_ADDRESS

Environment Variables:
--------------------
export ANT_PEERS=/ip4/127.0.0.1/udp/$NODE_PORT/quic-v1
export ANT_LOG=$LOG_LEVEL
export CLIENT_DATA_PATH=\"$CLIENT_DATA_DIR\"

Example Commands:
---------------
Upload file:   ./target/debug/ant file upload path/to/file
Download file: ./target/debug/ant file download <file-address>
Node status:   ./target/debug/ant node status
Get balance:   ./target/debug/ant wallet balance

Press Ctrl+C to stop the network
"
}

# Main Script
trap cleanup EXIT

# Check ports
check_port $NODE_PORT
check_port $EVM_PORT

# Install foundry if needed
install_foundry

# Create directories
mkdir -p "$NODE_DATA_DIR"
mkdir -p "$CLIENT_DATA_DIR"

# Build all binaries
build_binaries

# Start services
start_evm_testnet
start_local_node

# Print development information
print_dev_info

# Wait for Ctrl+C
wait 