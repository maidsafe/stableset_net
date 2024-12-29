# Autonomi API Documentation

Autonomi is a new layer of the internet built from a multitude of everyday devices, providing autonomous, secure, and perpetual data storage. The API enables you to interact with this decentralized network, leveraging quantum-secure protocols and distributed storage to ensure your data remains safe and accessible.

## Core Features

- **Quantum Secure**: Built with cutting-edge quantum security protocols, ensuring unmatched safety for your data
- **Autonomous & Distributed**: Control remains in the hands of users rather than centralized entities
- **Perpetual Data**: Permanent storage that provides true self-sovereignty over your digital life
- **Flexible Access**: Start with read-only access and upgrade to write capabilities as needed

## Getting Started

The Autonomi API provides a flexible interface for interacting with the network. You can start with a read-only client for browsing and reading data, then optionally upgrade to write capabilities when needed. See the [Client Modes Guide](../guides/client_modes.md) for details.

## Language Support

Autonomi provides native support for multiple programming languages:

- [Rust API](rust/README.md) - For systems programming and maximum performance
- [TypeScript/Node.js API](nodejs/README.md) - For web and server applications
- [Python API](python/README.md) - For data science and general-purpose development

## Core Data Types

Autonomi provides four fundamental data types that serve as building blocks for all network operations. These types enable quantum-secure storage, mutable references, transaction chains, and temporary workspaces. For detailed information about each type, see the [Data Types Guide](../guides/data_types.md).

### 1. Chunk - Quantum-Secure Storage

Immutable, quantum-secure encrypted data blocks that form the foundation of permanent storage:

```rust
// Rust
let chunk = client.store_chunk(data).await?;
```

```typescript
// TypeScript
const chunk = await client.storeChunk(data);
```

```python
# Python
chunk = client.store_chunk(data)
```

### 2. Pointer - Mutable References

Version-tracked references that enable updating data while maintaining stable addresses:

```rust
// Rust
let pointer = client.create_pointer(target).await?;
client.update_pointer(pointer.address(), new_target).await?;
```

```typescript
// TypeScript
const pointer = await client.createPointer(target);
await client.updatePointer(pointer.address, newTarget);
```

```python
# Python
pointer = client.create_pointer(target)
client.update_pointer(pointer.address(), new_target)
```

### 3. LinkedList - Transaction Chains

Decentralized DAG structures for maintaining ordered history and enabling value transfers:

```rust
// Rust
let list = client.create_linked_list().await?;
client.append_to_list(list.address(), item).await?;
```

```typescript
// TypeScript
const list = await client.createLinkedList();
await client.appendToList(list.address, item);
```

```python
# Python
list = client.create_linked_list()
client.append_to_list(list.address(), item)
```

### 4. ScratchPad - Temporary Workspace

Unstructured data storage with CRDT properties for configuration and temporary data:

```rust
// Rust
let pad = client.create_scratchpad(content_type).await?;
client.update_scratchpad(pad.address(), data).await?;
```

```typescript
// TypeScript
const pad = await client.createScratchpad(contentType);
await client.updateScratchpad(pad.address, data);
```

```python
# Python
pad = client.create_scratchpad(content_type)
client.update_scratchpad(pad.address(), data)
```

## Higher-Level Operations

### File System Operations

Built on top of the fundamental types, providing familiar file and directory operations:

```rust
// Rust
let file = client.store_file("example.txt", content).await?;
let dir = client.create_directory("docs").await?;
```

```typescript
// TypeScript
const file = await client.storeFile("example.txt", content);
const dir = await client.createDirectory("docs");
```

```python
# Python
file = client.store_file("example.txt", content)
dir = client.create_directory("docs")
```

### Common Use Cases

1. **Permanent Content Storage**

   ```rust
   // Store and retrieve quantum-secure data
   let address = client.store_chunk(data).await?;
   let retrieved = client.get_chunk(address).await?;
   ```

2. **Mutable Data Management**

   ```rust
   // Update data while maintaining stable addresses
   let pointer = client.create_pointer(initial_data).await?;
   client.update_pointer(pointer.address(), new_data).await?;
   ```

3. **Transaction History**

   ```rust
   // Create and verify autonomous transaction chains
   let list = client.create_linked_list().await?;
   client.append_to_list(list.address(), transaction).await?;
   let history = client.get_list_history(list.address()).await?;
   ```

4. **User Settings & Configuration**

   ```rust
   // Store encrypted user preferences
   let pad = client.create_scratchpad(ContentType::UserSettings).await?;
   client.update_scratchpad(pad.address(), encrypted_settings).await?;
   ```

## Error Handling

Each language provides appropriate error handling mechanisms aligned with its idioms:

```rust
// Rust
match client.get_chunk(address).await {
    Ok(data) => process_data(data),
    Err(ChunkError::NotFound) => handle_missing(),
    Err(e) => handle_other_error(e),
}
```

```typescript
// TypeScript
try {
    const data = await client.getChunk(address);
    processData(data);
} catch (error) {
    if (error instanceof ChunkNotFoundError) {
        handleMissing();
    } else {
        handleOtherError(error);
    }
}
```

```python
# Python
try:
    data = client.get_chunk(address)
    process_data(data)
except ChunkNotFoundError:
    handle_missing()
except Exception as e:
    handle_other_error(e)
```

## Best Practices

1. **Data Type Selection**
   - Use Chunks for permanent, immutable data
   - Use Pointers for mutable references
   - Use LinkedLists for ordered history
   - Use ScratchPads for temporary storage

2. **Security First**
   - Leverage quantum-secure storage
   - Encrypt sensitive data
   - Validate data integrity
   - Use access control features

3. **Performance Optimization**
   - Choose appropriate data types
   - Batch operations when possible
   - Consider data size limitations
   - Cache frequently accessed data

4. **Error Resilience**
   - Handle network errors gracefully
   - Implement retry logic
   - Validate data before storage
   - Check for version conflicts

## Join the Network

Autonomi is built by its community. Consider [running a node](https://autonomi.com/start) to:

- Contribute to the decentralized infrastructure
- Help secure the network
- Earn rewards for participation
- Shape the future of the internet

## Supporting Libraries

Autonomi provides several specialized libraries that can be used independently or as part of the main API:

### Self-Encryption

A quantum-secure data encryption library that provides self-encrypting files and data chunks:

```rust
// Rust
use self_encryption::{DataMap, SelfEncryptor};

let encryptor = SelfEncryptor::new(data)?;
let (data_map, chunks) = encryptor.encrypt()?;
```

```python
# Python
from self_encryption import SelfEncryptor

encryptor = SelfEncryptor(data)
data_map, chunks = encryptor.encrypt()
```

Key features:

- Quantum-secure encryption
- Content-based chunking
- Deduplication support
- Parallel processing
- Streaming support

### Node Implementation (antnode)

The core node implementation for participating in the Autonomi network:

```rust
// Rust
use antnode::{Node, NodeConfig};

let config = NodeConfig::builder()
    .rewards_address("0x1234...")
    .evm_network("arbitrum_sepolia")
    .build()?;
let node = Node::new(config)?;
node.run().await?;
```

```python
# Python
from antnode import AntNode

node = AntNode()
node.run(
    rewards_address="0x1234...",
    evm_network="arbitrum_sepolia",
    ip="0.0.0.0",
    port=12000,
    initial_peers=[
        "/ip4/142.93.37.4/udp/40184/quic-v1/p2p/12D3KooWPC8q7QGZsmuTtCYxZ2s3FPXPZcS8LVKkayXkVFkqDEQB",
    ]
)
```

Key features:

- Network participation
- Storage management
- Reward collection
- Peer discovery
- Data validation

### BLS Threshold Cryptography (blsttc)

A high-performance BLS threshold cryptography implementation:

```rust
// Rust
use blsttc::{SecretKey, PublicKey, Signature};

let sk = SecretKey::random();
let pk = sk.public_key();
let msg = b"Hello, World!";
let sig = sk.sign(msg);
assert!(pk.verify(&sig, msg));
```

```python
# Python
from blsttc import SecretKey, PublicKey, Signature

sk = SecretKey.random()
pk = sk.public_key()
msg = b"Hello, World!"
sig = sk.sign(msg)
assert pk.verify(sig, msg)
```

Key features:

- BLS12-381 curve support
- Threshold signatures
- Key aggregation
- Batch verification
- Secure serialization

## Library Details

### Self-Encryption

The self-encryption library provides a secure way to encrypt and chunk data:

```rust
// Rust - Detailed Usage
use self_encryption::{
    DataMap, SelfEncryptor, EncryptionConfig,
    ChunkStore, InMemoryChunkStore,
};

// Configure encryption
let config = EncryptionConfig::new()
    .with_chunk_size(1024 * 1024)  // 1MB chunks
    .with_compression(true);

// Create encryptor with custom chunk store
let chunk_store = InMemoryChunkStore::new();
let encryptor = SelfEncryptor::with_config(
    data,
    chunk_store,
    config,
)?;

// Encrypt data
let (data_map, chunks) = encryptor.encrypt()?;

// Decrypt data
let decryptor = SelfEncryptor::from_data_map(
    data_map,
    chunk_store,
)?;
let decrypted = decryptor.decrypt()?;
```

```python
# Python - Detailed Usage
from self_encryption import (
    SelfEncryptor, EncryptionConfig,
    ChunkStore, InMemoryChunkStore
)

# Configure encryption
config = EncryptionConfig(
    chunk_size=1024 * 1024,  # 1MB chunks
    compression=True
)

# Create encryptor with custom chunk store
chunk_store = InMemoryChunkStore()
encryptor = SelfEncryptor(
    data,
    chunk_store=chunk_store,
    config=config
)

# Encrypt data
data_map, chunks = encryptor.encrypt()

# Decrypt data
decryptor = SelfEncryptor.from_data_map(
    data_map,
    chunk_store=chunk_store
)
decrypted = decryptor.decrypt()
```

### Node Implementation (antnode)

The node implementation provides comprehensive network participation:

```rust
// Rust - Detailed Usage
use antnode::{
    Node, NodeConfig, StorageConfig,
    RewardsConfig, NetworkConfig,
};

// Configure node
let config = NodeConfig::builder()
    .rewards_address("0x1234...")
    .evm_network("arbitrum_sepolia")
    .storage(StorageConfig {
        max_capacity: 1024 * 1024 * 1024,  // 1GB
        min_free_space: 1024 * 1024 * 100,  // 100MB
    })
    .network(NetworkConfig {
        ip: "0.0.0.0".parse()?,
        port: 12000,
        initial_peers: vec![
            "/ip4/142.93.37.4/udp/40184/quic-v1/p2p/12D3KooWPC8q7QGZsmuTtCYxZ2s3FPXPZcS8LVKkayXkVFkqDEQB".parse()?,
        ],
    })
    .build()?;

// Create and run node
let node = Node::new(config)?;

// Get node information
println!("Peer ID: {}", node.peer_id());
println!("Rewards Address: {}", node.get_rewards_address());

// Storage operations
node.store_record(key, value, "chunk")?;
let data = node.get_record(key)?;
println!("Storage Size: {}", node.get_stored_records_size());

// Directory management
println!("Root Dir: {}", node.get_root_dir());
println!("Logs Dir: {}", node.get_logs_dir());
println!("Data Dir: {}", node.get_data_dir());

// Run node
node.run().await?;
```

```python
# Python - Detailed Usage
from antnode import AntNode, StorageConfig, NetworkConfig

# Create node
node = AntNode()

# Configure and run node
node.run(
    rewards_address="0x1234...",
    evm_network="arbitrum_sepolia",
    ip="0.0.0.0",
    port=12000,
    initial_peers=[
        "/ip4/142.93.37.4/udp/40184/quic-v1/p2p/12D3KooWPC8q7QGZsmuTtCYxZ2s3FPXPZcS8LVKkayXkVFkqDEQB",
    ],
    storage_config=StorageConfig(
        max_capacity=1024 * 1024 * 1024,  # 1GB
        min_free_space=1024 * 1024 * 100,  # 100MB
    )
)

# Node information
print(f"Peer ID: {node.peer_id()}")
print(f"Rewards Address: {node.get_rewards_address()}")

# Storage operations
node.store_record(key, value, "chunk")
data = node.get_record(key)
print(f"Storage Size: {node.get_stored_records_size()}")

# Directory management
print(f"Root Dir: {node.get_root_dir()}")
print(f"Logs Dir: {node.get_logs_dir()}")
print(f"Data Dir: {node.get_data_dir()}")
```

### BLS Threshold Cryptography (blsttc)

The BLS threshold cryptography library provides advanced cryptographic operations:

```rust
// Rust - Detailed Usage
use blsttc::{
    SecretKey, PublicKey, Signature,
    SecretKeySet, PublicKeySet,
};

// Basic signing
let sk = SecretKey::random();
let pk = sk.public_key();
let msg = b"Hello, World!";
let sig = sk.sign(msg);
assert!(pk.verify(&sig, msg));

// Threshold signatures
let threshold = 3;
let total = 5;
let sk_set = SecretKeySet::random(threshold, total);
let pk_set = sk_set.public_keys();

// Generate shares
let shares: Vec<_> = (0..total)
    .map(|i| sk_set.secret_key_share(i))
    .collect();

// Sign with shares
let sigs: Vec<_> = shares.iter()
    .map(|sk| sk.sign(msg))
    .collect();

// Combine signatures
let combined_sig = pk_set.combine_signatures(&sigs[..threshold + 1])?;
assert!(pk_set.public_key().verify(&combined_sig, msg));
```

```python
# Python - Detailed Usage
from blsttc import (
    SecretKey, PublicKey, Signature,
    SecretKeySet, PublicKeySet
)

# Basic signing
sk = SecretKey.random()
pk = sk.public_key()
msg = b"Hello, World!"
sig = sk.sign(msg)
assert pk.verify(sig, msg)

# Threshold signatures
threshold = 3
total = 5
sk_set = SecretKeySet.random(threshold, total)
pk_set = sk_set.public_keys()

# Generate shares
shares = [sk_set.secret_key_share(i) for i in range(total)]

# Sign with shares
sigs = [sk.sign(msg) for sk in shares]

# Combine signatures
combined_sig = pk_set.combine_signatures(sigs[:threshold + 1])
assert pk_set.public_key().verify(combined_sig, msg)
```

## Installation

### Rust

Add to your `Cargo.toml`:

```toml
[dependencies]
autonomi = "0.1.0"
self-encryption = "0.1.0"
antnode = "0.3.2"
blsttc = "0.1.0"
```

### Python

Install via pip:

```bash
pip install autonomi self-encryption antnode blsttc
```

## Further Reading

- [Data Types Guide](../guides/data_types.md) - Detailed information about fundamental data types
- [Client Modes Guide](../guides/client_modes.md) - Understanding read-only and read-write modes
- [Local Network Setup](../guides/local_network.md) - Setting up your local development environment
- [White Paper](https://autonomi.com/whitepaper) - Technical details about Autonomi's architecture
- [Documentation](https://autonomi.com/docs) - Complete platform documentation
