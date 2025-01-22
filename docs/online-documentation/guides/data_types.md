# Data Types Guide

This guide explains the fundamental data types in Autonomi and how they can be used to build higher-level abstractions like files and directories.

## Fundamental Data Types

Autonomi provides four fundamental data types that serve as building blocks for all network operations. Each type is designed for specific use cases and together they provide a complete system for decentralized data management.

### 1. Chunk

Chunks are the foundation of secure data storage in Autonomi, primarily used as the output of self-encrypting files. This provides quantum-secure encryption for data at rest.

```rust
// Store raw bytes as a chunk
let data = b"Hello, World!";
let chunk_address = client.store_chunk(data).await?;

// Retrieve chunk data
let retrieved = client.get_chunk(chunk_address).await?;
assert_eq!(data, retrieved);
```

Key characteristics:
- Quantum-secure encryption through self-encryption
- Immutable content
- Content-addressed (address is derived from data)
- Size-limited (maximum chunk size)
- Efficient for small to medium-sized data

#### Self-Encryption Process
1. Data is split into fixed-size sections
2. Each section is encrypted using data from other sections
3. Results in multiple encrypted chunks
4. Original data can only be recovered with all chunks

### 2. Pointer

Pointers provide a fixed network address that can reference any other data type, including other pointers. They enable mutable data structures while maintaining stable addresses.

```rust
// Create a pointer to some data
let pointer = client.create_pointer(target_address).await?;

// Update pointer target
client.update_pointer(pointer.address(), new_target_address).await?;

// Resolve pointer to get current target
let target = client.resolve_pointer(pointer.address()).await?;

// Chain pointers for indirection
let pointer_to_pointer = client.create_pointer(pointer.address()).await?;
```

Key characteristics:
- Fixed network address
- Mutable reference capability
- Single owner (controlled by secret key)
- Version tracking with monotonic counter
- Atomic updates
- Support for pointer chains and indirection

#### Common Use Cases
1. **Mutable Data References**
   ```rust
   // Update data while maintaining same address
   let pointer = client.create_pointer(initial_data).await?;
   client.update_pointer(pointer.address(), updated_data).await?;
   ```

2. **Latest Version Publishing**
   ```rust
   // Point to latest version while maintaining history
   let history = client.create_graph_entry().await?;
   let latest = client.create_pointer(history.address()).await?;
   ```

3. **Indirection and Redirection**
   ```rust
   // Create chain of pointers for flexible data management
   let data_pointer = client.create_pointer(data).await?;
   let redirect_pointer = client.create_pointer(data_pointer.address()).await?;
   ```

### 3. GraphEntry

Graphs in Autonomi are powerful structures of connected data on the network. They provide both historical tracking and CRDT-like properties.

```rust
// Create a new graph entry, signed with a secret key
let graph_content = [42u8; 32]; // 32 bytes of content
let graph_entry = GraphEntry::new(
    public_key,         // Graph entry address and owner
    vec![parent_pks],   // Parent graph entries
    graph_content,      // 32 bytes graph content
    vec![],             // Optional outputs (links to other graph entries)
    &secret_key         // Secret key for signing
);

// Calculate the cost to create a graph entry
let cost = client.graph_entry_cost(secret_key).await?;

// Store the entry in the network
client.graph_entry_put(graph_entry, &wallet).await?;

// Retrieve the entry from the network
let retrieved_entry = client.graph_entry_get(graph_entry.address()).await?;
```

Key characteristics:
- Decentralized Graph structure
- Each entry is signed by a unique key (sk) and addressed at that key (pk)
- CRDT-like conflict resolution
- Graph Traversal
- Can be used for value transfer (cryptocurrency-like)

### 4. ScratchPad

ScratchPad provides a flexible, unstructured data storage mechanism with CRDT properties through counter-based versioning. It's ideal for user account data, application configurations, and other frequently updated small data packets.

```rust
// Create a scratchpad for user settings
let pad = client.create_scratchpad(ContentType::UserSettings).await?;

// Update with encrypted data
let encrypted = encrypt_aes(settings_data, user_key)?;
client.update_scratchpad(pad.address(), encrypted).await?;

// Read and decrypt current data
let encrypted = client.get_scratchpad(pad.address()).await?;
let settings = decrypt_aes(encrypted, user_key)?;
```

Key characteristics:
- Unstructured data storage
- Counter-based CRDT for conflict resolution
- Type-tagged content
- Support for user-managed encryption
- Efficient for frequent updates
- Ideal for small data packets

#### Security Considerations

1. **Encryption**
   ```rust
   // Example of AES encryption for scratchpad data
   let key = generate_aes_key();
   let encrypted = aes_encrypt(data, key)?;
   client.update_scratchpad(pad.address(), encrypted).await?;
   ```

2. **Access Control**
   ```rust
   // Create encrypted scratchpad with access control
   let (public_key, private_key) = generate_keypair();
   let encrypted_key = encrypt_with_public_key(aes_key, public_key);
   let metadata = ScratchpadMetadata {
       encrypted_key,
       allowed_users: vec![public_key],
   };
   client.create_scratchpad_with_access(metadata).await?;
   ```

#### Common Applications

1. **User Profiles**
   ```rust
   // Store encrypted user profile
   let profile = UserProfile { name, settings };
   let encrypted = encrypt_profile(profile, user_key);
   client.update_scratchpad(profile_pad, encrypted).await?;
   ```

2. **Application State**
   ```rust
   // Maintain application configuration
   let config = AppConfig { preferences, state };
   let pad = client.get_or_create_config_pad().await?;
   client.update_scratchpad(pad, config).await?;
   ```

3. **Temporary Storage**
   ```rust
   // Use as temporary workspace
   let workspace = client.create_scratchpad(ContentType::Workspace).await?;
   client.update_scratchpad(workspace, working_data).await?;
   ```

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

Directories use graphs and pointers to maintain a mutable collection of entries:

- GraphEntry stores directory entries
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
   - Use graphs for linked data
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
   - GraphEntry for collections
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
   - GraphEntry traversal costs
   - Solution: Use appropriate data structures for access patterns
