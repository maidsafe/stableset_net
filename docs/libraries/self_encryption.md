# Self-Encryption Library

The self-encryption library provides quantum-secure data encryption with content-based chunking and deduplication. It's a core component of Autonomi's security infrastructure.

## Overview

Self-encryption is a unique approach to data security that:
- Encrypts data using its own content as the key
- Splits data into chunks for efficient storage and retrieval
- Provides automatic deduplication
- Enables parallel processing and streaming
- Ensures quantum security through advanced cryptographic techniques

## Installation

### Rust

```toml
[dependencies]
self-encryption = "0.1.0"

# Optional features
self-encryption = { version = "0.1.0", features = ["compression", "parallel"] }
```

### Python

```bash
pip install self-encryption

# With optional features
pip install self-encryption[compression,parallel]
```

## Basic Usage

### Simple Encryption/Decryption

```rust
// Rust
use self_encryption::{DataMap, SelfEncryptor};

// Create an encryptor
let encryptor = SelfEncryptor::new(data)?;

// Encrypt data
let (data_map, chunks) = encryptor.encrypt()?;

// Decrypt data
let decryptor = SelfEncryptor::from_data_map(data_map, chunks)?;
let decrypted = decryptor.decrypt()?;
```

```python
# Python
from self_encryption import SelfEncryptor

# Create an encryptor
encryptor = SelfEncryptor(data)

# Encrypt data
data_map, chunks = encryptor.encrypt()

# Decrypt data
decryptor = SelfEncryptor.from_data_map(data_map, chunks)
decrypted = decryptor.decrypt()
```

## Advanced Usage

### Custom Configuration

```rust
// Rust
use self_encryption::{
    EncryptionConfig, ChunkConfig,
    CompressionConfig, ParallelConfig,
};

// Configure encryption parameters
let config = EncryptionConfig::new()
    .with_chunk_size(1024 * 1024)  // 1MB chunks
    .with_min_chunks(3)
    .with_compression(CompressionConfig {
        algorithm: "zstd",
        level: 3,
    })
    .with_parallel(ParallelConfig {
        max_threads: 4,
        chunk_size: 1024 * 1024,
    });

// Create encryptor with config
let encryptor = SelfEncryptor::with_config(data, config)?;
```

```python
# Python
from self_encryption import (
    EncryptionConfig, ChunkConfig,
    CompressionConfig, ParallelConfig
)

# Configure encryption parameters
config = EncryptionConfig(
    chunk_size=1024 * 1024,  # 1MB chunks
    min_chunks=3,
    compression=CompressionConfig(
        algorithm="zstd",
        level=3
    ),
    parallel=ParallelConfig(
        max_threads=4,
        chunk_size=1024 * 1024
    )
)

# Create encryptor with config
encryptor = SelfEncryptor(data, config=config)
```

### Custom Chunk Store

```rust
// Rust
use self_encryption::{ChunkStore, ChunkInfo};

struct MyChunkStore {
    // Your storage implementation
}

impl ChunkStore for MyChunkStore {
    fn store(&mut self, chunk: &[u8]) -> Result<ChunkInfo> {
        // Store chunk and return info
    }

    fn retrieve(&self, info: &ChunkInfo) -> Result<Vec<u8>> {
        // Retrieve chunk using info
    }
}

// Use custom store
let store = MyChunkStore::new();
let encryptor = SelfEncryptor::with_store(data, store)?;
```

```python
# Python
from self_encryption import ChunkStore, ChunkInfo

class MyChunkStore(ChunkStore):
    def store(self, chunk: bytes) -> ChunkInfo:
        # Store chunk and return info
        pass

    def retrieve(self, info: ChunkInfo) -> bytes:
        # Retrieve chunk using info
        pass

# Use custom store
store = MyChunkStore()
encryptor = SelfEncryptor(data, chunk_store=store)
```

### Streaming Interface

```rust
// Rust
use self_encryption::streaming::{StreamEncryptor, StreamDecryptor};

// Create streaming encryptor
let mut encryptor = StreamEncryptor::new(config)?;

// Process data in chunks
while let Some(chunk) = stream.next().await {
    encryptor.write(&chunk).await?;
}

// Finalize encryption
let (data_map, chunks) = encryptor.finalize().await?;

// Streaming decryption
let mut decryptor = StreamDecryptor::from_data_map(data_map)?;
while let Some(chunk) = decryptor.next().await {
    process_chunk(chunk?).await;
}
```

```python
# Python
from self_encryption.streaming import StreamEncryptor, StreamDecryptor

# Create streaming encryptor
encryptor = StreamEncryptor(config)

# Process data in chunks
async for chunk in stream:
    await encryptor.write(chunk)

# Finalize encryption
data_map, chunks = await encryptor.finalize()

# Streaming decryption
decryptor = StreamDecryptor.from_data_map(data_map)
async for chunk in decryptor:
    await process_chunk(chunk)
```

## Performance Optimization

### Parallel Processing

```rust
// Rust
use self_encryption::parallel::{ParallelEncryptor, WorkerPool};

// Create worker pool
let pool = WorkerPool::new(4);  // 4 worker threads

// Create parallel encryptor
let encryptor = ParallelEncryptor::with_pool(data, pool)?;

// Encrypt with parallel processing
let (data_map, chunks) = encryptor.encrypt()?;
```

```python
# Python
from self_encryption.parallel import ParallelEncryptor, WorkerPool

# Create worker pool
pool = WorkerPool(4)  # 4 worker threads

# Create parallel encryptor
encryptor = ParallelEncryptor(data, worker_pool=pool)

# Encrypt with parallel processing
data_map, chunks = encryptor.encrypt()
```

### Memory Management

```rust
// Rust
use self_encryption::memory::{MemoryConfig, CacheConfig};

// Configure memory usage
let config = EncryptionConfig::new()
    .with_memory(MemoryConfig {
        max_chunk_cache: 100 * 1024 * 1024,  // 100MB
        max_total_memory: 1024 * 1024 * 1024,  // 1GB
    })
    .with_cache(CacheConfig {
        capacity: 1000,
        ttl: Duration::from_secs(300),
    });
```

```python
# Python
from self_encryption.memory import MemoryConfig, CacheConfig

# Configure memory usage
config = EncryptionConfig(
    memory=MemoryConfig(
        max_chunk_cache=100 * 1024 * 1024,  # 100MB
        max_total_memory=1024 * 1024 * 1024,  # 1GB
    ),
    cache=CacheConfig(
        capacity=1000,
        ttl=300  # seconds
    )
)
```

## Security Considerations

### Quantum Security

The self-encryption library uses quantum-secure algorithms:
- Post-quantum cryptographic primitives
- Information-theoretic security properties
- Forward secrecy guarantees
- Quantum-resistant key derivation

### Key Management

```rust
// Rust
use self_encryption::keys::{KeyConfig, KeyStore};

// Configure key management
let config = EncryptionConfig::new()
    .with_keys(KeyConfig {
        rotation_interval: Duration::from_days(30),
        min_entropy: 256,
        quantum_safe: true,
    });

// Use custom key store
let store = KeyStore::new(config)?;
let encryptor = SelfEncryptor::with_key_store(data, store)?;
```

```python
# Python
from self_encryption.keys import KeyConfig, KeyStore

# Configure key management
config = EncryptionConfig(
    keys=KeyConfig(
        rotation_interval=30 * 24 * 60 * 60,  # 30 days
        min_entropy=256,
        quantum_safe=True
    )
)

# Use custom key store
store = KeyStore(config)
encryptor = SelfEncryptor(data, key_store=store)
```

## Error Handling

```rust
// Rust
use self_encryption::error::{Error, Result};

match encryptor.encrypt() {
    Ok((data_map, chunks)) => {
        // Success
    }
    Err(Error::InvalidData(e)) => {
        // Handle invalid data
    }
    Err(Error::ChunkStore(e)) => {
        // Handle storage errors
    }
    Err(Error::Encryption(e)) => {
        // Handle encryption errors
    }
    Err(e) => {
        // Handle other errors
    }
}
```

```python
# Python
from self_encryption.error import (
    Error, InvalidDataError,
    ChunkStoreError, EncryptionError
)

try:
    data_map, chunks = encryptor.encrypt()
except InvalidDataError as e:
    # Handle invalid data
except ChunkStoreError as e:
    # Handle storage errors
except EncryptionError as e:
    # Handle encryption errors
except Error as e:
    # Handle other errors
```

## Best Practices

1. **Data Preparation**
   - Pre-process large files into appropriate chunks
   - Validate data integrity before encryption
   - Use appropriate compression for different data types
   - Consider data locality for chunk storage

2. **Performance**
   - Use parallel processing for large files
   - Enable compression for suitable data
   - Configure appropriate chunk sizes
   - Implement efficient chunk storage

3. **Security**
   - Use quantum-safe configuration
   - Implement proper key management
   - Validate all inputs
   - Handle errors appropriately

4. **Resource Management**
   - Configure appropriate memory limits
   - Use streaming for large files
   - Implement proper cleanup
   - Monitor resource usage

## API Reference

See the complete [API Reference](https://docs.rs/self-encryption) for detailed documentation of all types and functions. 