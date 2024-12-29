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

The local EVM network provides a complete testing environment for blockchain interactions:

### Pre-deployed Contracts

The following contracts are automatically deployed:

- Payment Vault Contract (`PaymentVaultNoProxy`)
  - Handles data storage payments
  - Manages token approvals and transfers
  - Verifies payment proofs
- Test Token Contract (`TestToken`)
  - ERC20 token for testing payments
  - Pre-minted supply for test accounts
  - Automatic approval for test wallets

### Test Accounts

Several accounts are pre-funded and ready to use:

```
Primary Test Account:
Address: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
Private Key: 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
Balance: 10000 TEST tokens

Secondary Test Account:
Address: 0x70997970C51812dc3A010C7d01b50e0d17dc79C8
Private Key: 0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d
Balance: 1000 TEST tokens
```

### RPC Endpoint

The local EVM network exposes an RPC endpoint at `http://localhost:8545` with:

- Full JSON-RPC API support
- WebSocket subscriptions
- Low block time (1 second)
- Zero gas costs
- Instant transaction confirmations

### Interacting with the Network

#### JavaScript/TypeScript

```typescript
import { ethers } from 'ethers';

// Connect to local network
const provider = new ethers.JsonRpcProvider('http://localhost:8545');
const wallet = new ethers.Wallet(PRIVATE_KEY, provider);

// Get contract instances
const paymentVault = new ethers.Contract(
  PAYMENT_VAULT_ADDRESS,
  PAYMENT_VAULT_ABI,
  wallet
);

// Interact with contracts
await paymentVault.getQuote([metrics]);
await paymentVault.payForQuotes(payments);
```

#### Python

```python
from web3 import Web3
from eth_account import Account

# Connect to local network
w3 = Web3(Web3.HTTPProvider('http://localhost:8545'))
account = Account.from_key(PRIVATE_KEY)

# Get contract instances
payment_vault = w3.eth.contract(
    address=PAYMENT_VAULT_ADDRESS,
    abi=PAYMENT_VAULT_ABI
)

# Interact with contracts
payment_vault.functions.getQuote([metrics]).call()
payment_vault.functions.payForQuotes(payments).transact()
```

#### Rust

```rust
use ethers::prelude::*;

// Connect to local network
let provider = Provider::<Http>::try_from("http://localhost:8545")?;
let wallet = LocalWallet::from_bytes(&PRIVATE_KEY)?;
let client = SignerMiddleware::new(provider, wallet);

// Get contract instances
let payment_vault = PaymentVault::new(
    PAYMENT_VAULT_ADDRESS,
    Arc::new(client)
);

// Interact with contracts
payment_vault.get_quote(metrics).call().await?;
payment_vault.pay_for_quotes(payments).send().await?;
```

## Environment Variables

The following environment variables are set up automatically:

- `ANT_PEERS` - Local node endpoint
- `ANT_LOG` - Logging level
- `CLIENT_DATA_PATH` - Client data directory

## Monitoring and Debugging

### Logging

#### Node Logs

The Autonomi node generates detailed logs that can be controlled via `RUST_LOG`:

```bash
# Trace level for maximum detail
RUST_LOG=trace ./test-local.sh

# Focus on specific modules
RUST_LOG=autonomi=debug,ant_node=trace ./test-local.sh

# Log locations:
- Node logs: $NODE_DATA_DIR/node.log
- EVM logs: $NODE_DATA_DIR/evm.log
```

#### Log Levels

- `error`: Critical issues that need immediate attention
- `warn`: Important events that aren't failures
- `info`: General operational information
- `debug`: Detailed information for debugging
- `trace`: Very detailed protocol-level information

#### Following Logs

```bash
# Follow node logs
tail -f "$NODE_DATA_DIR/node.log"

# Follow EVM logs
tail -f "$NODE_DATA_DIR/evm.log"

# Filter for specific events
tail -f "$NODE_DATA_DIR/node.log" | grep "payment"
```

### Debugging

#### Node Debugging

Using `rust-lldb`:

```bash
# Start node with debugger
rust-lldb target/debug/antnode -- --features test

# Common commands:
b autonomi::client::payment::pay  # Set breakpoint
r                                # Run
bt                              # Backtrace
p variable                      # Print variable
c                              # Continue
```

Using `rust-gdb`:

```bash
# Start node with debugger
rust-gdb target/debug/antnode -- --features test

# Common commands:
break autonomi::client::payment::pay  # Set breakpoint
run                                  # Run
backtrace                           # Show backtrace
print variable                      # Examine variable
continue                            # Continue execution
```

#### Network Monitoring

Monitor network activity:

```bash
# Watch network connections
netstat -an | grep 5000  # Default node port

# Monitor network traffic
sudo tcpdump -i lo0 port 5000

# Check EVM network
curl -X POST -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
  http://localhost:8545
```

#### Contract Debugging

Debug contract interactions:

```bash
# Get payment vault state
cast call $PAYMENT_VAULT_ADDRESS \
  "payments(bytes32)" \
  $QUOTE_HASH \
  --rpc-url http://localhost:8545

# Watch for payment events
cast events $PAYMENT_VAULT_ADDRESS \
  --rpc-url http://localhost:8545
```

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
