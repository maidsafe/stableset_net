# Python API Documentation

The Python implementation of Autonomi provides a flexible and intuitive interface for data science and general-purpose development. It's ideal for:

- Data analysis and machine learning applications
- Web applications and APIs
- Research and prototyping
- Integration with existing Python ecosystems

## Installation

```bash
# Install with pip
pip install autonomi

# Or with specific features
pip install autonomi[quantum-secure,compression]

# For data science applications
pip install autonomi[data-science]
```

## Client Initialization

The client provides flexible initialization options to match your security and performance needs:

```python
from autonomi import Client

# Initialize a read-only client for browsing
client = Client.init_read_only()

# Initialize with write capabilities and custom configuration
config = {
    'quantum_security': True,
    'compression': True,
    'cache_size': '1GB'
}
client = Client.init_with_wallet_and_config(wallet, config)

# Upgrade a read-only client to read-write
client.upgrade_to_read_write(wallet)
```

## Core Data Types

### Chunk - Quantum-Secure Storage

Store and retrieve immutable, quantum-secure encrypted data with maximum efficiency:

```python
from autonomi import Chunk
import numpy as np

# Store raw data as a chunk with optional compression
data = b"Hello, World!"
chunk = client.store_chunk(data)

# Store numpy array as compressed chunk
array_data = np.random.randn(1000, 1000)
chunk = client.store_chunk_compressed(array_data.tobytes())

# Retrieve chunk data with automatic decompression
retrieved = client.get_chunk(chunk.address)
assert data == retrieved

# Get chunk metadata including storage metrics
metadata = client.get_chunk_metadata(chunk.address)
print(f"Size: {metadata.size}, Replicas: {metadata.replicas}")

# Store multiple chunks efficiently
chunks = client.store_chunks(data_list)
```

### Pointer - Mutable References

Create and manage version-tracked references with atomic updates:

```python
from autonomi import Pointer
from datetime import datetime

# Create a pointer with custom metadata
metadata = {
    'created_at': datetime.utcnow(),
    'description': 'Latest model weights'
}
pointer = client.create_pointer_with_metadata(
    target_address,
    metadata
)

# Atomic pointer updates with version checking
client.update_pointer(pointer.address, new_target_address)

# Resolve pointer with caching
target = client.resolve_pointer_cached(pointer.address)

# Get pointer metadata and version history
metadata = client.get_pointer_metadata(pointer.address)
print(f"Version: {metadata.version}, Updates: {metadata.update_count}")
```

### LinkedList - Transaction Chains

Build high-performance decentralized DAG structures:

```python
from autonomi import LinkedList
import pandas as pd

# Create a new linked list with configuration
config = {
    'fork_detection': True,
    'history_compression': True
}
list = client.create_linked_list_with_config(config)

# Efficient batch appends
client.append_to_list_batch(list.address, items)

# Stream list contents with generator
for item in client.stream_list(list.address):
    process_item(item)

# Stream as pandas DataFrame for data analysis
for chunk in client.stream_list_as_dataframe(list.address):
    process_dataframe(chunk)

# Advanced fork detection and resolution
forks = client.detect_forks_detailed(list.address)
if not forks:
    print("No forks detected")
else:
    resolved = client.resolve_fork_automatically(forks.branches)
    print(f"Fork resolved: {resolved}")
```

### ScratchPad - Temporary Workspace

Efficient unstructured data storage with CRDT properties:

```python
from autonomi import ScratchPad, ContentType

# Create a scratchpad with custom configuration
config = {
    'compression': True,
    'encryption': True
}
pad = client.create_scratchpad_with_config(
    ContentType.USER_SETTINGS,
    config
)

# Batch updates for efficiency
updates = [
    ('key1', value1),
    ('key2', value2)
]
client.update_scratchpad_batch(pad.address, updates)

# Store pandas DataFrame
df = pd.DataFrame({'A': range(1000), 'B': range(1000)})
client.update_scratchpad_dataframe(pad.address, df)

# Stream updates with generator
for update in client.stream_scratchpad_updates(pad.address):
    process_update(update)
```

## File System Operations

High-performance file and directory operations:

```python
from autonomi.fs import File, Directory, FileOptions
import pandas as pd

# Store a file with custom options
options = FileOptions(
    compression=True,
    encryption=True,
    redundancy=3
)
file = client.store_file_with_options(
    "example.txt",
    content,
    options
)

# Store pandas DataFrame as CSV
df = pd.DataFrame({'A': range(1000), 'B': range(1000)})
file = client.store_dataframe(df, "data.csv")

# Create a directory with custom metadata
dir = client.create_directory_with_metadata(
    "docs",
    metadata
)

# Efficient recursive operations
client.add_to_directory_recursive(dir.address, file.address)

# Stream directory entries
for entry in client.stream_directory(dir.address):
    if entry.is_file:
        print(f"File: {entry.name}")
    else:
        print(f"Dir: {entry.name}")
```

## Error Handling

Comprehensive error handling with detailed exceptions:

```python
from autonomi.errors import ChunkError, PointerError, ListError, ScratchPadError

# Handle chunk operations with detailed errors
try:
    data = client.get_chunk(address)
    process_data(data)
except ChunkError.NotFound as e:
    print(f"Chunk not found: {e.address}")
    handle_missing()
except ChunkError.NetworkError as e:
    print(f"Network error: {e}")
    handle_network_error(e)
except ChunkError.ValidationError as e:
    print(f"Validation failed: expected {e.expected}, got {e.actual}")
    handle_validation_error()
except Exception as e:
    handle_other_error(e)

# Handle pointer updates with version conflicts
try:
    client.update_pointer(address, new_target)
    print("Update successful")
except PointerError.VersionConflict as e:
    print(f"Version conflict: current {e.current}, attempted {e.attempted}")
    handle_conflict()
except Exception as e:
    handle_other_error(e)
```

## Advanced Usage

### Data Science Integration

```python
import pandas as pd
import numpy as np
from autonomi.data import DataFrameStore

# Create a data store for efficient DataFrame operations
store = DataFrameStore(client)

# Store DataFrame with automatic optimization
store.put("dataset", df, optimize=True)

# Retrieve with lazy loading for large datasets
retrieved_df = store.get("dataset", lazy=True)

# Perform operations on stored DataFrames
result = store.apply("dataset", lambda df: df.groupby('column').mean())

# Stream large datasets in chunks
for chunk in store.stream("dataset", chunk_size=10000):
    process_chunk(chunk)
```

### Custom Types with Pydantic

```python
from pydantic import BaseModel
from datetime import datetime

class UserProfile(BaseModel):
    name: str
    age: int
    preferences: dict
    created_at: datetime = datetime.utcnow()

# Use the model with Autonomi
profile = UserProfile(
    name="Alice",
    age=30,
    preferences={
        "theme": "light",
        "notifications": True
    }
)

# Store with validation
pad = client.create_scratchpad(ContentType.Custom("UserProfile"))
client.update_scratchpad_validated(pad.address, profile.dict())
```

## Performance Optimization

### Connection Pooling

```python
from autonomi.pool import Pool

# Create a connection pool
pool = Pool(
    min_connections=5,
    max_connections=20,
    idle_timeout=30
)

# Get a client from the pool
with pool.get() as client:
    process_data(client)
```

### Batch Operations

```python
# Batch chunk storage
chunks = client.store_chunks_batch(data_list)

# Batch pointer updates
updates = [
    PointerUpdate(addr1, target1),
    PointerUpdate(addr2, target2)
]
client.update_pointers_batch(updates)
```

## Best Practices

1. **Data Science Integration**
   - Use pandas integration for DataFrames
   - Leverage numpy array support
   - Stream large datasets
   - Use compression for numerical data

2. **Error Handling**
   - Use detailed exception types
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

## Type Hints

The Python API uses type hints throughout for better IDE support and code quality:

```python
from typing import List, Optional, Union
from autonomi.types import Address, Data, Metadata

def store_chunk(self, data: bytes) -> Address: ...
def get_chunk(self, address: Address) -> bytes: ...
def create_pointer(self, target: Address) -> Pointer: ...
def update_pointer(self, address: Address, target: Address) -> None: ...
```

## Further Reading

- [Data Science Guide](/guides/data_science)
- [Quantum Security Guide](/guides/quantum_security)
- [Error Handling Guide](/guides/error_handling)
- [API Reference](https://autonomi.readthedocs.io)
- [Examples Repository](https://github.com/autonomi/examples)
