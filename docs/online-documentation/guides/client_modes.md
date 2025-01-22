# Client Modes Guide

This guide explains how to use Autonomi's client modes to browse the network (read-only) and optionally upgrade to write capabilities.

## Overview

Autonomi clients can operate in two modes:

1. **Read-Only Mode**: Browse and read data from the network without requiring a wallet
2. **Read-Write Mode**: Full access to both read and write operations, requires a wallet

## Read-Only Client

A read-only client allows you to browse and read data from the network without needing a wallet or making payments.

### Rust

```rust
use autonomi::Client;

// Initialize a read-only client
let client = Client::init_read_only().await?;

// Verify it's read-only
assert!(!client.can_write());
assert!(client.wallet().is_none());

// Read operations work normally
let data = client.get_bytes(address).await?;
let file = client.get_file(file_map, "output.txt").await?;
```

### TypeScript/JavaScript

```typescript
import { Client } from '@autonomi/client';

// Initialize a read-only client
const client = await Client.connect({
    readOnly: true,
    peers: ['/ip4/127.0.0.1/tcp/12000']
});

// Read operations
const data = await client.dataGetPublic(address);
const list = await client.GraphEntryGet(listAddress);
```

### Python

```python
from autonomi import Client

# Initialize a read-only client
client = Client.new()

# Read operations
data = client.get_bytes("safe://example_address")
file = client.get_file(file_map, "output.txt")
```

## Upgrading to Read-Write Mode

You can upgrade a read-only client to read-write mode by adding a wallet. This enables write operations like storing data or updating graphs.

### Rust

```rust
use autonomi::{Client, EvmWallet};

// Start with a read-only client
let mut client = Client::init_read_only().await?;

// Get a wallet (e.g., from a private key or create new)
let wallet = EvmWallet::from_private_key(private_key)?;

// Upgrade to read-write mode
client.upgrade_to_read_write(wallet)?;

// Now write operations are available
let address = client.store_bytes(data).await?;
```

### TypeScript/JavaScript

```typescript
import { Client } from '@autonomi/client';

// Start with a read-only client
const client = await Client.connect({
    readOnly: true
});

// Upgrade with a wallet
await client.upgradeToReadWrite({
    type: 'wallet',
    wallet: 'your_wallet_address'
});

// Now you can perform write operations
const address = await client.dataPutPublic(
    Buffer.from('Hello World'),
    { type: 'wallet', wallet: client.wallet }
);
```

### Python

```python
from autonomi import Client, Wallet

# Start with a read-only client
client = Client.new()

# Create or import a wallet
wallet = Wallet.from_private_key("your_private_key")

# Upgrade to read-write mode
client.upgrade_to_read_write(wallet)

# Now write operations are available
address = client.store_bytes(b"Hello World")
```

## Write Operations

The following operations require a wallet (read-write mode):

- Storing public data (`dataPutPublic`)
- Creating/updating graphs (`GraphEntryPut`)
- Setting pointers (`pointerPut`)
- Writing to vaults (`writeBytesToVault`)
- Updating user data (`putUserDataToVault`)

Attempting these operations in read-only mode will result in an error.

## Best Practices

1. **Start Read-Only**: Begin with a read-only client if you only need to read data. This is simpler and more secure since no wallet is needed.

2. **Lazy Wallet Loading**: Only upgrade to read-write mode when you actually need to perform write operations.

3. **Error Handling**: Always handle potential errors when upgrading modes or performing write operations:

```typescript
try {
    await client.upgradeToReadWrite(wallet);
    await client.dataPutPublic(data, payment);
} catch (error) {
    if (error.code === 'NO_WALLET') {
        console.error('Write operation attempted without wallet');
    } else if (error.code === 'ALREADY_READ_WRITE') {
        console.error('Client is already in read-write mode');
    }
}
```

4. **Check Capabilities**: Use the provided methods to check client capabilities:

```rust
if client.can_write() {
    // Perform write operation
} else {
    // Handle read-only state
}
```

## Common Issues

1. **Attempting Write Operations in Read-Only Mode**
   - Error: `NO_WALLET` or `WriteAccessRequired`
   - Solution: Upgrade to read-write mode by adding a wallet

2. **Multiple Upgrade Attempts**
   - Error: `ALREADY_READ_WRITE`
   - Solution: Check client mode before attempting upgrade

3. **Invalid Wallet**
   - Error: `InvalidWallet` or `WalletError`
   - Solution: Ensure wallet is properly initialized with valid credentials
