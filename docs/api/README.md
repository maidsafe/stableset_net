# Autonomi API Reference

The Autonomi API provides a consistent interface across multiple programming languages, making it easy to build applications in your preferred language while maintaining the same powerful capabilities.

## Installation

### Rust

```toml
[dependencies]
autonomi = "0.1.0"
```

### Python

```bash
pip install autonomi
```

### TypeScript/Node.js

```bash
npm install autonomi
```

## Client Initialization

Initialize a client with flexible options for security and performance:

### Rust

```rust
use autonomi::Client;

// Initialize a read-only client
let client = Client::init_read_only().await?;

// Initialize with write capabilities and config
let config = ClientConfig::builder()
    .with_quantum_security(true)
    .with_compression(true)
    .build();
let client = Client::init_with_wallet_and_config(wallet, config).await?;
```

### Python

```python
from autonomi import Client

# Initialize a read-only client
client = Client.init_read_only()

# Initialize with write capabilities and config
config = {
    'quantum_security': True,
    'compression': True,
    'cache_size': '1GB'
}
client = Client.init_with_wallet_and_config(wallet, config)
```

### TypeScript/Node.js

```typescript
import { Client, ClientConfig } from 'autonomi';

// Initialize a read-only client
const client = await Client.initReadOnly();

// Initialize with write capabilities and config
const config: ClientConfig = {
    quantumSecurity: true,
    compression: true,
    cacheSize: '1GB'
};
const client = await Client.initWithWalletAndConfig(wallet, config);
```

## Core Data Types

### Chunk - Quantum-Secure Storage

Store and retrieve immutable, quantum-secure encrypted data:

### Rust

```rust
use autonomi::Chunk;

// Store raw data
let data = b"Hello, World!";
let chunk = client.store_chunk(data).await?;

// Retrieve data
let retrieved = client.get_chunk(chunk.address()).await?;
assert_eq!(data, &retrieved[..]);

// Get metadata
let metadata = client.get_chunk_metadata(chunk.address()).await?;
println!("Size: {}, Replicas: {}", metadata.size, metadata.replicas);
```

### Python

```python
from autonomi import Chunk
import numpy as np

# Store raw data
data = b"Hello, World!"
chunk = client.store_chunk(data)

# Store numpy array
array_data = np.random.randn(1000, 1000)
chunk = client.store_chunk_compressed(array_data.tobytes())

# Retrieve data
retrieved = client.get_chunk(chunk.address)
assert data == retrieved

# Get metadata
metadata = client.get_chunk_metadata(chunk.address)
print(f"Size: {metadata.size}, Replicas: {metadata.replicas}")
```

### TypeScript/Node.js

```typescript
import { Chunk } from 'autonomi';

// Store raw data
const data = Buffer.from('Hello, World!');
const chunk = await client.storeChunk(data);

// Store with streaming
const stream = createReadStream('large-file.dat');
const chunk = await client.storeChunkStream(stream);

// Retrieve data
const retrieved = await client.getChunk(chunk.address);
assert(Buffer.compare(data, retrieved) === 0);

// Get metadata
const metadata = await client.getChunkMetadata(chunk.address);
console.log(`Size: ${metadata.size}, Replicas: ${metadata.replicas}`);
```

### Pointer - Mutable References

Create and manage version-tracked references:

### Rust

```rust
use autonomi::Pointer;

// Create a pointer with metadata
let pointer = client.create_pointer_with_metadata(
    target_address,
    metadata,
).await?;

// Update with version checking
client.update_pointer(pointer.address(), new_target_address).await?;

// Resolve with caching
let target = client.resolve_pointer_cached(pointer.address()).await?;

// Get metadata
let metadata = client.get_pointer_metadata(pointer.address()).await?;
println!("Version: {}, Updates: {}", metadata.version, metadata.update_count);
```

### Python

```python
from autonomi import Pointer
from datetime import datetime

# Create a pointer with metadata
metadata = {
    'created_at': datetime.utcnow(),
    'description': 'Latest model weights'
}
pointer = client.create_pointer_with_metadata(
    target_address,
    metadata
)

# Update with version checking
client.update_pointer(pointer.address, new_target_address)

# Resolve with caching
target = client.resolve_pointer_cached(pointer.address)

# Get metadata
metadata = client.get_pointer_metadata(pointer.address)
print(f"Version: {metadata.version}, Updates: {metadata.update_count}")
```

### TypeScript/Node.js

```typescript
import { Pointer } from 'autonomi';

// Create a pointer with metadata
const metadata = {
    createdAt: new Date(),
    description: 'Latest application state'
};
const pointer = await client.createPointerWithMetadata(
    targetAddress,
    metadata
);

// Update with version checking
await client.updatePointer(pointer.address, newTargetAddress);

// Subscribe to updates
client.subscribeToPointer(pointer.address, (update) => {
    console.log(`New target: ${update.target}`);
});

// Get metadata
const metadata = await client.getPointerMetadata(pointer.address);
console.log(`Version: ${metadata.version}, Updates: ${metadata.updateCount}`);
```

### LinkedList - Transaction Chains

Build decentralized DAG structures:

### Rust

```rust
use autonomi::LinkedList;

// Create with configuration
let config = LinkedListConfig::new()
    .with_fork_detection(true)
    .with_history_compression(true);
let list = client.create_linked_list_with_config(config).await?;

// Batch appends
client.append_to_list_batch(list.address(), items).await?;

// Stream contents
let mut items = client.stream_list(list.address());
while let Some(item) = items.next().await {
    process_item(item?);
}

// Fork detection
match client.detect_forks_detailed(list.address()).await? {
    Fork::None => println!("No forks detected"),
    Fork::Detected(branches) => {
        let resolved = client.resolve_fork_automatically(branches).await?;
        println!("Fork resolved: {:?}", resolved);
    }
}
```

### Python

```python
from autonomi import LinkedList
import pandas as pd

# Create with configuration
config = {
    'fork_detection': True,
    'history_compression': True
}
list = client.create_linked_list_with_config(config)

# Batch appends
client.append_to_list_batch(list.address, items)

# Stream contents
for item in client.stream_list(list.address):
    process_item(item)

# Stream as DataFrame
for chunk in client.stream_list_as_dataframe(list.address):
    process_dataframe(chunk)

# Fork detection
forks = client.detect_forks_detailed(list.address)
if not forks:
    print("No forks detected")
else:
    resolved = client.resolve_fork_automatically(forks.branches)
    print(f"Fork resolved: {resolved}")
```

### TypeScript/Node.js

```typescript
import { LinkedList } from 'autonomi';

// Create with configuration
const config = {
    forkDetection: true,
    historyCompression: true,
    realtime: true
};
const list = await client.createLinkedListWithConfig(config);

// Batch appends
await client.appendToListBatch(list.address, items);

// Subscribe to updates
client.subscribeToList(list.address, (update) => {
    console.log(`New item: ${update.data}`);
});

// Fork detection
const forks = await client.detectForksDetailed(list.address);
if (!forks) {
    console.log('No forks detected');
} else {
    const resolved = await client.resolveForkAutomatically(forks.branches);
    console.log(`Fork resolved: ${resolved}`);
}
```

### ScratchPad - Temporary Workspace

Efficient unstructured data storage with CRDT properties:

### Rust

```rust
use autonomi::{ScratchPad, ContentType};

// Create with configuration
let config = ScratchpadConfig::new()
    .with_compression(true)
    .with_encryption(true);
let pad = client.create_scratchpad_with_config(
    ContentType::UserSettings,
    config,
).await?;

// Batch updates
let updates = vec![Update::new(key1, value1), Update::new(key2, value2)];
client.update_scratchpad_batch(pad.address(), updates).await?;

// Stream updates
let mut updates = client.stream_scratchpad_updates(pad.address());
while let Some(update) = updates.next().await {
    process_update(update?);
}
```

### Python

```python
from autonomi import ScratchPad, ContentType

# Create with configuration
config = {
    'compression': True,
    'encryption': True
}
pad = client.create_scratchpad_with_config(
    ContentType.USER_SETTINGS,
    config
)

# Batch updates
updates = [
    ('key1', value1),
    ('key2', value2)
]
client.update_scratchpad_batch(pad.address, updates)

# Store DataFrame
df = pd.DataFrame({'A': range(1000), 'B': range(1000)})
client.update_scratchpad_dataframe(pad.address, df)

# Stream updates
for update in client.stream_scratchpad_updates(pad.address):
    process_update(update)
```

### TypeScript/Node.js

```typescript
import { ScratchPad, ContentType } from 'autonomi';

// Create with configuration
const config = {
    compression: true,
    encryption: true,
    realtime: true
};
const pad = await client.createScratchpadWithConfig(
    ContentType.UserSettings,
    config
);

// Update with JSON
const settings = { theme: 'dark', fontSize: 14 };
await client.updateScratchpadJson(pad.address, settings);

// Subscribe to updates
client.subscribeToScratchpad(pad.address, (update) => {
    console.log(`New data: ${update.data}`);
});
```

## Error Handling

Each language provides comprehensive error handling:

### Rust

```rust
use autonomi::error::{ChunkError, PointerError};

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
    Err(e) => handle_other_error(e),
}
```

### Python

```python
from autonomi.errors import ChunkError, PointerError

try:
    data = client.get_chunk(address)
    process_data(data)
except ChunkError.NotFound as e:
    print(f"Chunk not found: {e.address}")
    handle_missing()
except ChunkError.NetworkError as e:
    print(f"Network error: {e}")
    handle_network_error(e)
except Exception as e:
    handle_other_error(e)
```

### TypeScript/Node.js

```typescript
import { ChunkError, PointerError } from 'autonomi/errors';

try {
    const data = await client.getChunk(address);
    processData(data);
} catch (error) {
    if (error instanceof ChunkError.NotFound) {
        console.log(`Chunk not found: ${error.address}`);
        handleMissing();
    } else if (error instanceof ChunkError.NetworkError) {
        console.log(`Network error: ${error.message}`);
        handleNetworkError(error);
    } else {
        handleOtherError(error);
    }
}
```

## Further Reading

- [Web Development Guide](../guides/web_development.md)
- [Data Science Guide](../guides/data_science.md)
- [Quantum Security Guide](../guides/quantum_security.md)
- [Error Handling Guide](../guides/error_handling.md)
- [Rust Performance Guide](../guides/rust_performance.md)
