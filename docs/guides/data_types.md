# Data Types Guide

This guide explains the fundamental data types in Autonomi and how they can be used to build higher-level abstractions like files and directories.

## Fundamental Data Types

Autonomi provides four fundamental data types that serve as building blocks for all network operations:

### 1. Chunk

The most basic unit of data storage in the network. Chunks are immutable blocks of bytes with content-addressed storage.

```rust
// Store raw bytes as a chunk
let data = b"Hello, World!";
let chunk_address = client.store_chunk(data).await?;

// Retrieve chunk data
let retrieved = client.get_chunk(chunk_address).await?;
assert_eq!(data, retrieved);
```

Key characteristics:

- Immutable
- Content-addressed (address is derived from data)
- Size-limited (maximum chunk size)
- Encrypted at rest
- Efficient for small to medium-sized data

### 2. Pointer

A mutable reference to any other data type. Pointers allow updating references while maintaining a stable address.

```rust
// Create a pointer to some data
let pointer = client.create_pointer(target_address).await?;

// Update pointer target
client.update_pointer(pointer.address(), new_target_address).await?;

// Resolve pointer to get current target
let target = client.resolve_pointer(pointer.address()).await?;
```

Key characteristics:

- Mutable reference
- Single owner (controlled by secret key)
- Version tracking
- Atomic updates
- Useful for mutable data structures

### 3. LinkedList

An ordered collection of items that can be appended to or modified.

```rust
// Create a new linked list
let list = client.create_linked_list().await?;

// Append items
client.append_to_list(list.address(), item1).await?;
client.append_to_list(list.address(), item2).await?;

// Read list contents
let items = client.get_list(list.address()).await?;
```

Key characteristics:

- Append-only structure
- Ordered items
- Efficient for sequential access
- Supports large collections
- Version control via counter

### 4. ScratchPad

A mutable workspace for temporary or frequently changing data.

```rust
// Create a scratchpad
let pad = client.create_scratchpad(content_type).await?;

// Update scratchpad data
client.update_scratchpad(pad.address(), new_data).await?;

// Read current data
let data = client.get_scratchpad(pad.address()).await?;
```

Key characteristics:

- Mutable workspace
- Type-tagged content
- Efficient for frequent updates
- Owner-controlled access
- Temporary storage

## Higher-Level Abstractions

These fundamental types can be combined to create higher-level data structures:

### File System

The Autonomi file system is built on top of these primitives:

```rust
// Create a directory
let dir = client.create_directory("my_folder").await?;

// Create a file
let file = client.create_file("example.txt", content).await?;

// Add file to directory
client.add_to_directory(dir.address(), file.address()).await?;

// List directory contents
let entries = client.list_directory(dir.address()).await?;
```

#### Files

Files are implemented using a combination of chunks and pointers:

- Large files are split into chunks
- File metadata stored in pointer
- Content addressing for deduplication

```rust
// Store a large file
let file_map = client.store_file("large_file.dat").await?;

// Read file contents
client.get_file(file_map, "output.dat").await?;
```

#### Directories

Directories use linked lists and pointers to maintain a mutable collection of entries:

- LinkedList stores directory entries
- Pointer maintains current directory state
- Hierarchical structure support

```rust
// Create nested directory structure
let root = client.create_directory("/").await?;
let docs = client.create_directory("docs").await?;
client.add_to_directory(root.address(), docs.address()).await?;

// List recursively
let tree = client.list_recursive(root.address()).await?;
```

## Common Patterns

### Data Organization

1. **Static Content**
   - Use chunks for immutable data
   - Content addressing enables deduplication
   - Efficient for read-heavy workloads

2. **Mutable References**
   - Use pointers for updateable references
   - Maintain stable addresses
   - Version tracking built-in

3. **Collections**
   - Use linked lists for ordered data
   - Efficient for append operations
   - Good for logs and sequences

4. **Temporary Storage**
   - Use scratchpads for working data
   - Frequent updates supported
   - Type-tagged content

### Best Practices

1. **Choose the Right Type**
   - Chunks for immutable data
   - Pointers for mutable references
   - LinkedLists for collections
   - ScratchPads for temporary storage

2. **Efficient Data Structures**

   ```rust
   // Bad: Using chunks for frequently changing data
   let chunk = client.store_chunk(changing_data).await?;
   
   // Good: Using scratchpad for frequently changing data
   let pad = client.create_scratchpad(content_type).await?;
   client.update_scratchpad(pad.address(), changing_data).await?;
   ```

3. **Version Management**

   ```rust
   // Track versions with pointers
   let versions = Vec::new();
   versions.push(pointer.version());
   client.update_pointer(pointer.address(), new_data).await?;
   versions.push(pointer.version());
   ```

4. **Error Handling**

   ```rust
   match client.get_chunk(address).await {
       Ok(data) => process_data(data),
       Err(ChunkError::NotFound) => handle_missing_chunk(),
       Err(ChunkError::InvalidSize) => handle_size_error(),
       Err(e) => handle_other_error(e),
   }
   ```

## Common Issues

1. **Size Limitations**
   - Chunk size limits
   - Solution: Split large data across multiple chunks

2. **Update Conflicts**
   - Concurrent pointer updates
   - Solution: Use version checking

3. **Performance**
   - LinkedList traversal costs
   - Solution: Use appropriate data structures for access patterns
