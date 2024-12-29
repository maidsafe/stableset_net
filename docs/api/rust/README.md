# Rust API Documentation

The Rust implementation of Autonomi provides maximum performance and systems-level control. It's ideal for:

- Building high-performance applications
- Implementing custom storage solutions
- Developing network nodes
- Creating secure, native applications

## Installation

Add Autonomi to your `Cargo.toml`:

```toml
[dependencies]
autonomi = "0.1.0"
```

Or with specific features:

```toml
[dependencies]
autonomi = { version = "0.1.0", features = ["quantum-secure", "compression"] }
```

## Client Initialization

The client provides flexible initialization options to match your security and performance needs:

```rust
use autonomi::Client;

// Initialize a read-only client for browsing
let client = Client::init_read_only().await?;

// Initialize with write capabilities and custom configuration
let config = ClientConfig::builder()
    .with_quantum_security(true)
    .with_compression(true)
    .build();
let client = Client::init_with_wallet_and_config(wallet, config).await?;

// Upgrade a read-only client to read-write
client.upgrade_to_read_write(wallet)?;
```

## Core Data Types

### Chunk - Quantum-Secure Storage

Store and retrieve immutable, quantum-secure encrypted data with maximum efficiency:

```rust
use autonomi::Chunk;

// Store raw data as a chunk with optional compression
let data = b"Hello, World!";
let chunk = client.store_chunk(data).await?;

// Retrieve chunk data with automatic decompression
let retrieved = client.get_chunk(chunk.address()).await?;
assert_eq!(data, &retrieved[..]);

// Get chunk metadata including storage metrics
let metadata = client.get_chunk_metadata(chunk.address()).await?;
println!("Size: {}, Replicas: {}", metadata.size, metadata.replicas);

// Store multiple chunks efficiently
let chunks = client.store_chunks(data_vec).await?;
```

### Pointer - Mutable References

Create and manage version-tracked references with atomic updates:

```rust
use autonomi::Pointer;

// Create a pointer with custom metadata
let pointer = client.create_pointer_with_metadata(
    target_address,
    metadata,
).await?;

// Atomic pointer updates with version checking
client.update_pointer(pointer.address(), new_target_address).await?;

// Resolve pointer with caching
let target = client.resolve_pointer_cached(pointer.address()).await?;

// Get pointer metadata and version history
let metadata = client.get_pointer_metadata(pointer.address()).await?;
println!("Version: {}, Updates: {}", metadata.version, metadata.update_count);
```

### LinkedList - Transaction Chains

Build high-performance decentralized DAG structures:

```rust
use autonomi::LinkedList;

// Create a new linked list with configuration
let config = LinkedListConfig::new()
    .with_fork_detection(true)
    .with_history_compression(true);
let list = client.create_linked_list_with_config(config).await?;

// Efficient batch appends
client.append_to_list_batch(list.address(), items).await?;

// Stream list contents with async iterator
let mut items = client.stream_list(list.address());
while let Some(item) = items.next().await {
    process_item(item?);
}

// Advanced fork detection and resolution
match client.detect_forks_detailed(list.address()).await? {
    Fork::None => println!("No forks detected"),
    Fork::Detected(branches) => {
        let resolved = client.resolve_fork_automatically(branches).await?;
        println!("Fork resolved: {:?}", resolved);
    }
}
```

### ScratchPad - Temporary Workspace

Efficient unstructured data storage with CRDT properties:

```rust
use autonomi::{ScratchPad, ContentType};

// Create a scratchpad with custom configuration
let config = ScratchpadConfig::new()
    .with_compression(true)
    .with_encryption(true);
let pad = client.create_scratchpad_with_config(
    ContentType::UserSettings,
    config,
).await?;

// Batch updates for efficiency
let updates = vec![Update::new(key1, value1), Update::new(key2, value2)];
client.update_scratchpad_batch(pad.address(), updates).await?;

// Stream updates with async iterator
let mut updates = client.stream_scratchpad_updates(pad.address());
while let Some(update) = updates.next().await {
    process_update(update?);
}
```

## File System Operations

High-performance file and directory operations:

```rust
use autonomi::fs::{File, Directory, FileOptions};

// Store a file with custom options
let options = FileOptions::new()
    .with_compression(true)
    .with_encryption(true)
    .with_redundancy(3);
let file = client.store_file_with_options(
    "example.txt",
    content,
    options,
).await?;

// Create a directory with custom metadata
let dir = client.create_directory_with_metadata(
    "docs",
    metadata,
).await?;

// Efficient recursive operations
client.add_to_directory_recursive(dir.address(), file.address()).await?;

// Stream directory entries
let mut entries = client.stream_directory(dir.address());
while let Some(entry) = entries.next().await {
    match entry? {
        DirEntry::File(f) => println!("File: {}", f.name),
        DirEntry::Directory(d) => println!("Dir: {}", d.name),
    }
}
```

## Error Handling

Comprehensive error handling with detailed error types:

```rust
use autonomi::error::{ChunkError, PointerError, ListError, ScratchPadError};

// Handle chunk operations with detailed errors
match client.get_chunk(address).await {
    Ok(data) => process_data(data),
    Err(ChunkError::NotFound { address }) => {
        println!("Chunk not found: {}", address);
        handle_missing()
    },
    Err(ChunkError::NetworkError(e)) => {
        println!("Network error: {}", e);
        handle_network_error(e)
    },
    Err(ChunkError::ValidationError { expected, actual }) => {
        println!("Validation failed: expected {}, got {}", expected, actual);
        handle_validation_error()
    },
    Err(e) => handle_other_error(e),
}

// Handle pointer updates with version conflicts
match client.update_pointer(address, new_target).await {
    Ok(_) => println!("Update successful"),
    Err(PointerError::VersionConflict { current, attempted }) => {
        println!("Version conflict: current {}, attempted {}", current, attempted);
        handle_conflict()
    },
    Err(e) => handle_other_error(e),
}
```

## Advanced Usage

### Custom Types with Serde

```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct MyData {
    field1: String,
    field2: u64,
    #[serde(with = "time::serde::rfc3339")]
    timestamp: OffsetDateTime,
}

// Store custom type with compression
let data = MyData {
    field1: "test".into(),
    field2: 42,
    timestamp: OffsetDateTime::now_utc(),
};
let pad = client.create_scratchpad(ContentType::Custom("MyData")).await?;
client.update_scratchpad_compressed(pad.address(), &data).await?;
```

### Quantum-Secure Encryption

```rust
use autonomi::crypto::{encrypt_quantum_secure, decrypt_quantum_secure};

// Generate quantum-secure keys
let key = generate_quantum_secure_key();

// Encrypt data with quantum security
let encrypted = encrypt_quantum_secure(data, &key)?;
let pad = client.create_scratchpad(ContentType::Encrypted).await?;
client.update_scratchpad(pad.address(), &encrypted).await?;

// Decrypt with quantum security
let encrypted = client.get_scratchpad(pad.address()).await?;
let decrypted = decrypt_quantum_secure(encrypted, &key)?;
```

## Performance Optimization

### Connection Pooling

```rust
use autonomi::pool::{Pool, PoolConfig};

// Create a connection pool
let pool = Pool::new(PoolConfig {
    min_connections: 5,
    max_connections: 20,
    idle_timeout: Duration::from_secs(30),
});

// Get a client from the pool
let client = pool.get().await?;
```

### Batch Operations

```rust
// Batch chunk storage
let chunks = client.store_chunks_batch(data_vec).await?;

// Batch pointer updates
let updates = vec![
    PointerUpdate::new(addr1, target1),
    PointerUpdate::new(addr2, target2),
];
client.update_pointers_batch(updates).await?;
```

## Best Practices

1. **Performance Optimization**
   - Use batch operations for multiple items
   - Enable compression for large data
   - Utilize connection pooling
   - Stream large datasets

2. **Error Handling**
   - Use detailed error types
   - Implement retry logic
   - Handle version conflicts
   - Validate data integrity

3. **Security**
   - Enable quantum security
   - Use encryption for sensitive data
   - Implement access control
   - Validate all inputs

4. **Resource Management**
   - Use connection pools
   - Clean up resources
   - Monitor memory usage
   - Handle backpressure

## Further Reading

- [Rust Performance Guide](/guides/rust_performance)
- [Quantum Security Guide](/guides/quantum_security)
- [Error Handling Guide](/guides/error_handling)
- [API Reference](https://docs.rs/autonomi)
- [Examples Repository](https://github.com/autonomi/examples)
