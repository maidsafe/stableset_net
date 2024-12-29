# Local Network Setup Guide

This guide explains how to set up and run a local Autonomi network for development and testing purposes.

## Prerequisites

- Rust toolchain (with `cargo` installed)
- Git (for cloning the repository)

That's it! Everything else needed will be built from source.

## Quick Start

1. Clone the repository:

```bash
git clone https://github.com/dirvine/autonomi
cd autonomi
```

2. Start the local network:

```bash
./test-local.sh
```

This script will:

- Build all necessary components
- Start a local EVM testnet
- Start a local Autonomi node
- Set up the development environment

## Network Components

The local network consists of:

- An Autonomi node running in local mode
- A local EVM test network with pre-funded accounts
- Test wallets for development

## Testing with EVM Networks

The local EVM network comes with:

- Pre-deployed payment contracts
- Pre-funded test wallets
- Local RPC endpoint

You can interact with it using:

- Web3.js/ethers.js for JavaScript
- Web3.py for Python
- ethers-rs for Rust

## Environment Variables

The following environment variables are set up automatically:

- `ANT_PEERS` - Local node endpoint
- `ANT_LOG` - Logging level
- `CLIENT_DATA_PATH` - Client data directory

## Monitoring and Debugging

### Logging

- Set `RUST_LOG=trace` for detailed logs
- Logs are written to the data directory
- Use `tail -f` to follow logs in real-time

### Debugging

- Use `rust-gdb` or `rust-lldb` for debugging the node
- Monitor network activity through log output
- Check EVM testnet state through RPC calls

## Common Issues and Solutions

### Port Conflicts

If you see port-in-use errors:

1. Check if another instance is running
2. Use different ports in the script
3. Kill existing processes if needed

### Build Issues

1. Make sure Rust toolchain is up to date
2. Clean and rebuild: `cargo clean && cargo build`
3. Check for missing dependencies

### Network Issues

1. Verify the node is running
2. Check log output for errors
3. Ensure EVM testnet is accessible

## Advanced Usage

### Custom Configuration

You can modify the test script to:

- Change ports
- Adjust logging levels
- Configure node parameters

### Multiple Nodes

To run multiple nodes:

1. Copy the script
2. Modify ports and directories
3. Run each instance separately
