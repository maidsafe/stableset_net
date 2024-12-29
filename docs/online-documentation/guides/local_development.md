# Local Development Environment

This guide will help you set up a local development environment for building applications with Autonomi. We'll use a script that sets up a local network with all the necessary components for development and testing.

## Prerequisites

- Rust toolchain installed
- Git repository cloned
- Basic understanding of terminal/command line

## Setup Script

Save the following script as `start-local-network.sh` in your project root:

```bash
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

# ... (rest of the script content) ...
```

Make the script executable:

```bash
chmod +x start-local-network.sh
```

## Using the Development Environment

1. Start the local network:

   ```bash
   ./start-local-network.sh
   ```

2. The script will:
   - Build all necessary components (ant-node, evm-testnet, ant CLI)
   - Start a local EVM testnet
   - Start a local Autonomi node
   - Set up the development environment

3. Once running, you'll see information about:
   - Network endpoints
   - Environment variables
   - Example commands

## Environment Variables

The following environment variables should be set for your development environment:

```bash
export ANT_PEERS=/ip4/127.0.0.1/udp/5000/quic-v1
export ANT_LOG=info
export CLIENT_DATA_PATH=$HOME/Library/Application Support/autonomi/client
```

## Example Usage

### File Operations

Upload a file:

```bash
./target/debug/ant file upload path/to/file
```

Download a file:

```bash
./target/debug/ant file download <file-address>
```

### Node Operations

Check node status:

```bash
./target/debug/ant node status
```

Get wallet balance:

```bash
./target/debug/ant wallet balance
```

## Development Tips

1. **Local Testing**: The local network is perfect for testing your applications without affecting the main network.

2. **Quick Iterations**: Changes to your application can be tested immediately without waiting for network confirmations.

3. **Clean State**: Each time you start the network, it begins with a clean state, making it ideal for testing different scenarios.

4. **Debugging**: The local environment provides detailed logs and quick feedback for debugging.

## Customization

You can customize the development environment by modifying the configuration variables at the top of the script:

- `NODE_PORT`: Change the port the node listens on
- `LOG_LEVEL`: Adjust logging verbosity ("trace", "debug", "info", "warn", "error")
- `EVM_PORT`: Change the EVM testnet port
- Other settings as needed

## Troubleshooting

1. **Port Conflicts**: If you see port-in-use errors, modify the `NODE_PORT` or `EVM_PORT` in the script.

2. **Process Cleanup**: If the script fails to start, ensure no old processes are running:

   ```bash
   pkill -f "antnode"
   pkill -f "evm-testnet"
   ```

3. **Data Cleanup**: To start completely fresh, remove the data directories:

   ```bash
   rm -rf "$HOME/Library/Application Support/autonomi/node"
   rm -rf "$HOME/Library/Application Support/autonomi/client"
   ```
