#!/bin/bash
set -e

# Configuration
NODE_DATA_DIR="$HOME/Library/Application Support/autonomi/node"
EVM_PORT=4343
EVM_RPC_URL="http://localhost:8545"
WALLET_ADDRESS="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
TOKEN_ADDRESS="0x5FbDB2315678afecb367f032d93F642f64180aa3"
LOG_LEVEL="trace"

# Function to cleanup processes on exit
cleanup() {
    echo "Cleaning up..."
    pkill -f "antnode" || true
    pkill -f "evm-testnet" || true
    rm -rf "$NODE_DATA_DIR" || true
}

# Function to check if a port is in use
check_port() {
    nc -z localhost $1 >/dev/null 2>&1
    return $?
}

# Function to start the EVM testnet
start_evm_testnet() {
    if ! check_port $EVM_PORT && ! check_port ${EVM_PORT#*:}; then
        echo "Starting new EVM testnet..."
        RPC_PORT=$EVM_PORT ./target/debug/evm-testnet --genesis-wallet $WALLET_ADDRESS &
        EVM_PID=$!
        echo "Waiting for EVM testnet to be ready..."
        sleep 5
    else
        echo "EVM network is already running, using existing instance..."
    fi
}

# Function to start the local node
start_local_node() {
    echo "Starting local node..."
    ./target/debug/antnode \
        --local \
        --rewards-address $WALLET_ADDRESS \
        --home-network \
        --first \
        --ignore-cache \
        evm-custom \
        --data-payments-address $WALLET_ADDRESS \
        --payment-token-address $TOKEN_ADDRESS \
        --rpc-url $EVM_RPC_URL &
    NODE_PID=$!
    
    echo "Waiting for node to be ready..."
    sleep 10
}

# Function to build required binaries
build_binaries() {
    echo "Building ant-node..."
    cargo build -p ant-node --features local

    echo "Building evm-testnet..."
    cargo build -p evm-testnet
}

# Function to run tests
run_tests() {
    echo "Running tests..."
    RUST_LOG=$LOG_LEVEL cargo test -p autonomi --features test -- --nocapture
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

# Main execution
echo "Cleaning up any existing processes..."
cleanup

build_binaries
start_evm_testnet
start_local_node
run_tests

# Wait for background processes
if [[ -n "${EVM_PID}" ]]; then
    wait ${EVM_PID}
fi
if [[ -n "${NODE_PID}" ]]; then
    wait ${NODE_PID}
fi 