# Payments Guide

This guide explains how payments work in Autonomi, particularly for put operations that store data on the network.

## Overview

When storing data on the Autonomi network, you need to pay for the storage space. Payments are made using EVM-compatible tokens through a smart contract system. There are two ways to handle payments:

1. Direct payment using an EVM wallet
2. Pre-paid operations using a receipt

## Payment Options

### Using an EVM Wallet

The simplest way to pay for put operations is to use an EVM wallet:

```python
# Python
from autonomi import Client, PaymentOption
from autonomi.evm import EvmWallet

# Initialize client
client = Client()

# Create or load a wallet
wallet = EvmWallet.create()  # or load from private key
payment = PaymentOption.from_wallet(wallet)

# Put data with wallet payment
data = b"Hello, World!"
address = client.data_put_public(data, payment)
```

```typescript
// Node.js
import { Client, PaymentOption } from '@autonomi/client';
import { EvmWallet } from '@autonomi/evm';

// Initialize client
const client = new Client();

// Create or load a wallet
const wallet = EvmWallet.create();  // or load from private key
const payment = PaymentOption.fromWallet(wallet);

// Put data with wallet payment
const data = Buffer.from("Hello, World!");
const address = await client.dataPutPublic(data, payment);
```

```rust
// Rust
use autonomi::{Client, PaymentOption};
use ant_evm::EvmWallet;

// Initialize client
let client = Client::new()?;

// Create or load a wallet
let wallet = EvmWallet::create()?;  // or load from private key
let payment = wallet.into();  // Converts to PaymentOption

// Put data with wallet payment
let data = b"Hello, World!".to_vec();
let address = client.data_put_public(data.into(), payment).await?;
```

### Using Pre-paid Receipts

For better efficiency when doing multiple put operations, you can pre-pay for storage and reuse the receipt:

```python
# Python
from autonomi import Client, PaymentOption
from autonomi.evm import EvmWallet

# Initialize client
client = Client()
wallet = EvmWallet.create()

# Get receipt for multiple operations
data1 = b"First piece of data"
data2 = b"Second piece of data"

# Create payment receipt
receipt = client.create_payment_receipt([data1, data2], wallet)
payment = PaymentOption.from_receipt(receipt)

# Use receipt for puts
addr1 = client.data_put_public(data1, payment)
addr2 = client.data_put_public(data2, payment)
```

```typescript
// Node.js
import { Client, PaymentOption } from '@autonomi/client';
import { EvmWallet } from '@autonomi/evm';

// Initialize client
const client = new Client();
const wallet = EvmWallet.create();

// Get receipt for multiple operations
const data1 = Buffer.from("First piece of data");
const data2 = Buffer.from("Second piece of data");

// Create payment receipt
const receipt = await client.createPaymentReceipt([data1, data2], wallet);
const payment = PaymentOption.fromReceipt(receipt);

// Use receipt for puts
const addr1 = await client.dataPutPublic(data1, payment);
const addr2 = await client.dataPutPublic(data2, payment);
```

```rust
// Rust
use autonomi::{Client, PaymentOption};
use ant_evm::EvmWallet;

// Initialize client
let client = Client::new()?;
let wallet = EvmWallet::create()?;

// Get receipt for multiple operations
let data1 = b"First piece of data".to_vec();
let data2 = b"Second piece of data".to_vec();

// Create payment receipt
let receipt = client.create_payment_receipt(
    vec![data1.clone(), data2.clone()].into_iter(), 
    &wallet
).await?;
let payment = receipt.into();  // Converts to PaymentOption

// Use receipt for puts
let addr1 = client.data_put_public(data1.into(), payment.clone()).await?;
let addr2 = client.data_put_public(data2.into(), payment).await?;
```

## Cost Calculation

The cost of storing data depends on several factors:

- Size of the data
- Network density
- Storage duration
- Current network conditions

You can calculate the cost before performing a put operation:

```python
# Python
cost = client.calculate_storage_cost(data)
print(f"Storage will cost {cost} tokens")
```

```typescript
// Node.js
const cost = await client.calculateStorageCost(data);
console.log(`Storage will cost ${cost} tokens`);
```

```rust
// Rust
let cost = client.calculate_storage_cost(&data).await?;
println!("Storage will cost {} tokens", cost);
```

## Token Management

Before you can pay for storage, you need to ensure your wallet has sufficient tokens and has approved the payment contract to spend them:

```python
# Python
# Check balance
balance = wallet.get_balance()

# Approve tokens if needed
if not wallet.has_approved_tokens():
    wallet.approve_tokens()
```

```typescript
// Node.js
// Check balance
const balance = await wallet.getBalance();

// Approve tokens if needed
if (!await wallet.hasApprovedTokens()) {
    await wallet.approveTokens();
}
```

```rust
// Rust
// Check balance
let balance = wallet.get_balance().await?;

// Approve tokens if needed
if !wallet.has_approved_tokens().await? {
    wallet.approve_tokens().await?;
}
```

## Error Handling

Common payment-related errors you might encounter:

1. `InsufficientBalance` - Wallet doesn't have enough tokens
2. `TokenNotApproved` - Token spending not approved for the payment contract
3. `PaymentExpired` - Payment quote has expired (when using receipts)
4. `PaymentVerificationFailed` - Payment verification failed on the network

Example error handling:

```python
# Python
try:
    address = client.data_put_public(data, payment)
except InsufficientBalance:
    print("Not enough tokens in wallet")
except TokenNotApproved:
    print("Need to approve token spending")
except PaymentError as e:
    print(f"Payment failed: {e}")
```

```typescript
// Node.js
try {
    const address = await client.dataPutPublic(data, payment);
} catch (e) {
    if (e instanceof InsufficientBalance) {
        console.log("Not enough tokens in wallet");
    } else if (e instanceof TokenNotApproved) {
        console.log("Need to approve token spending");
    } else {
        console.log(`Payment failed: ${e}`);
    }
}
```

```rust
// Rust
match client.data_put_public(data.into(), payment).await {
    Err(PutError::InsufficientBalance) => {
        println!("Not enough tokens in wallet");
    }
    Err(PutError::TokenNotApproved) => {
        println!("Need to approve token spending");
    }
    Err(e) => {
        println!("Payment failed: {}", e);
    }
    Ok(address) => {
        println!("Data stored at {}", address);
    }
}
```

## Best Practices

1. **Pre-approve Tokens**: Approve token spending before starting put operations to avoid extra transactions.

2. **Use Receipts**: When doing multiple put operations, use receipts to avoid making separate payments for each operation.

3. **Check Costs**: Always check storage costs before proceeding with large data uploads.

4. **Handle Errors**: Implement proper error handling for payment-related issues.

5. **Monitor Balance**: Keep track of your wallet balance to ensure sufficient funds for operations.

## Testing Payments

When testing your application, you can use the local development environment which provides a test EVM network with pre-funded wallets. See the [Local Development Guide](local_development.md) for details.
