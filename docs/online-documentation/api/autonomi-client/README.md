# Autonomi API Documentation

## Installation

Choose your preferred language:

=== "Node.js"
    ```bash
    # Note: Package not yet published to npm
    # Clone the repository and build from source
    git clone https://github.com/dirvine/autonomi.git
    cd autonomi
    npm install
    ```

=== "Python"
    ```bash
    pip install autonomi
    ```

=== "Rust"
    ```toml
    # Add to Cargo.toml:
    [dependencies]
    autonomi = "0.3.1"
    ```

## Client Initialization

Initialize a client in read-only mode for browsing data, or with write capabilities for full access:

=== "Node.js"
    ```typescript
    import { Client } from 'autonomi';

    // Initialize a read-only client
    const client = await Client.initReadOnly();

    // Or initialize with write capabilities and configuration
    const config = {
        // Add your configuration here
    };
    const client = await Client.initWithConfig(config);
    ```

=== "Python"
    ```python
    from autonomi import Client

    # Initialize a read-only client
    client = Client.init_read_only()

    # Or initialize with write capabilities and configuration
    config = {
        # Add your configuration here
    }
    client = Client.init_with_config(config)
    ```

=== "Rust"
    ```rust
    use autonomi::Client;

    // Initialize a read-only client
    let client = Client::new_local().await?;

    // Or initialize with configuration
    let config = ClientConfig::default();
    let client = Client::new(config).await?;
    ```

## Core Data Types

Autonomi provides four fundamental data types that serve as building blocks for all network operations. For detailed information about each type, see the [Data Types Guide](../../guides/data_types.md).

### 1. Chunk

Immutable, quantum-secure encrypted data blocks:

=== "Node.js"
    ```typescript
    import { Chunk } from 'autonomi';

    // Store raw data as a chunk
    const data = Buffer.from('Hello, World!');
    const chunk = await client.storeChunk(data);

    // Retrieve chunk data
    const retrieved = await client.getChunk(chunk.address);
    assert(Buffer.compare(data, retrieved) === 0);

    // Get chunk metadata
    const metadata = await client.getChunkMetadata(chunk.address);
    console.log(`Size: ${metadata.size}`);
    ```

=== "Python"
    ```python
    from autonomi import Chunk

    # Store raw data as a chunk
    data = b"Hello, World!"
    chunk = client.store_chunk(data)

    # Retrieve chunk data
    retrieved = client.get_chunk(chunk.address)
    assert data == retrieved

    # Get chunk metadata
    metadata = client.get_chunk_metadata(chunk.address)
    print(f"Size: {metadata.size}")
    ```

=== "Rust"
    ```rust
    use autonomi::Chunk;

    // Store raw data as a chunk
    let data = b"Hello, World!";
    let chunk = client.store_chunk(data).await?;

    // Retrieve chunk data
    let retrieved = client.get_chunk(chunk.address()).await?;
    assert_eq!(data, &retrieved[..]);

    // Get chunk metadata
    let metadata = client.get_chunk_metadata(chunk.address()).await?;
    println!("Size: {}", metadata.size);
    ```

### 2. Pointer

Mutable references with version tracking:

=== "Node.js"
    ```typescript
    import { Pointer } from 'autonomi';

    // Create a pointer to some data
    const pointer = await client.createPointer(targetAddress);

    // Update pointer target
    await client.updatePointer(pointer.address, newTargetAddress);

    // Resolve pointer to get current target
    const target = await client.resolvePointer(pointer.address);

    // Get pointer metadata and version
    const metadata = await client.getPointerMetadata(pointer.address);
    console.log(`Version: ${metadata.version}`);
    ```

=== "Python"
    ```python
    from autonomi import Pointer

    # Create a pointer to some data
    pointer = client.create_pointer(target_address)

    # Update pointer target
    client.update_pointer(pointer.address, new_target_address)

    # Resolve pointer to get current target
    target = client.resolve_pointer(pointer.address)

    # Get pointer metadata and version
    metadata = client.get_pointer_metadata(pointer.address)
    print(f"Version: {metadata.version}")
    ```

=== "Rust"
    ```rust
    use autonomi::Pointer;

    // Create a pointer to some data
    let pointer = client.create_pointer(target_address).await?;

    // Update pointer target
    client.update_pointer(pointer.address(), new_target_address).await?;

    // Resolve pointer to get current target
    let target = client.resolve_pointer(pointer.address()).await?;

    // Get pointer metadata and version
    let metadata = client.get_pointer_metadata(pointer.address()).await?;
    println!("Version: {}", metadata.version);
    ```

### 3. GraphEntry

Decentralized Graph structures for linked data:

=== "Node.js"
    ```typescript
    import { GraphEntry } from 'autonomi';

    // Create a new graph
    const entry = await client.createGraphEntry();

    // Append items
    await client.appendToGraph(entry.address, item1);
    await client.appendToGraph(entry.address, item2);

    // Read graph contents
    const items = await client.getGraph(entry.address);

    // Get graph history
    const history = await client.getGraphHistory(entry.address);
    for (const entry of history) {
        console.log(`Version ${entry.version}: ${entry.data}`);
    }

    // Check for forks
    const forks = await client.detectForks(entry.address);
    if (!forks) {
        console.log('No forks detected');
    } else {
        handleForks(forks.branches);
    }
    ```

=== "Python"
    ```python
    from autonomi import GraphEntry

    # Create a new graph
    entry = client.create_graph_entry()

    # Append items
    client.append_to_graph(entry.address, item1)
    client.append_to_graph(entry.address, item2)

    # Read list contents
    items = client.get_graph(entry.address)

    # Get graph history
    history = client.get_graph_history(entry.address)
    for entry in history:
        print(f"Version {entry.version}: {entry.data}")
    ```

=== "Rust"
    ```rust
    use autonomi::GraphEntry;

    // Create a new graph
    let entry = client.create_graph_entry().await?;

    // Append items
    client.append_to_graph(entry.address(), item1).await?;
    client.append_to_graph(entry.address(), item2).await?;

    // Read graph contents
    let items = client.get_graph(entry.address()).await?;

    // Get graph history
    let history = client.get_graph_history(entry.address()).await?;
    for entry in history {
        println!("Version {}: {:?}", entry.version, entry.data);
    }
    ```

### 4. ScratchPad

Unstructured data with CRDT properties:

=== "Node.js"
    ```typescript
    import { ScratchPad, ContentType } from 'autonomi';

    // Create a scratchpad
    const pad = await client.createScratchpad(ContentType.UserSettings);

    // Update with data
    await client.updateScratchpad(pad.address, settingsData);

    // Read current data
    const current = await client.getScratchpad(pad.address);

    // Get metadata
    const metadata = await client.getScratchpadMetadata(pad.address);
    console.log(`Updates: ${metadata.updateCounter}`);
    ```

=== "Python"
    ```python
    from autonomi import ScratchPad, ContentType

    # Create a scratchpad
    pad = client.create_scratchpad(ContentType.USER_SETTINGS)

    # Update with data
    client.update_scratchpad(pad.address, settings_data)

    # Read current data
    current = client.get_scratchpad(pad.address)

    # Get metadata
    metadata = client.get_scratchpad_metadata(pad.address)
    print(f"Updates: {metadata.update_counter}")
    ```

=== "Rust"
    ```rust
    use autonomi::{ScratchPad, ContentType};

    // Create a scratchpad
    let pad = client.create_scratchpad(ContentType::UserSettings).await?;

    // Update with data
    client.update_scratchpad(pad.address(), settings_data).await?;

    // Read current data
    let current = client.get_scratchpad(pad.address()).await?;

    // Get metadata
    let metadata = client.get_scratchpad_metadata(pad.address()).await?;
    println!("Updates: {}", metadata.update_counter);
    ```

## File System Operations

Create and manage files and directories:

=== "Node.js"
    ```typescript
    import { File, Directory } from 'autonomi/fs';

    // Store a file
    const file = await client.storeFile('example.txt', content);

    // Create a directory
    const dir = await client.createDirectory('docs');

    // Add file to directory
    await client.addToDirectory(dir.address, file.address);

    // List directory contents
    const entries = await client.listDirectory(dir.address);
    for (const entry of entries) {
        if (entry.isFile) {
            console.log(`File: ${entry.name}`);
        } else {
            console.log(`Dir: ${entry.name}`);
        }
    }
    ```

=== "Python"
    ```python
    from autonomi.fs import File, Directory

    # Store a file
    file = client.store_file("example.txt", content)

    # Create a directory
    dir = client.create_directory("docs")

    # Add file to directory
    client.add_to_directory(dir.address, file.address)

    # List directory contents
    entries = client.list_directory(dir.address)
    for entry in entries:
        if entry.is_file:
            print(f"File: {entry.name}")
        else:
            print(f"Dir: {entry.name}")
    ```

=== "Rust"
    ```rust
    use autonomi::fs::{File, Directory};

    // Store a file
    let file = client.store_file("example.txt", content).await?;

    // Create a directory
    let dir = client.create_directory("docs").await?;

    // Add file to directory
    client.add_to_directory(dir.address(), file.address()).await?;

    // List directory contents
    let entries = client.list_directory(dir.address()).await?;
    for entry in entries {
        match entry {
            DirEntry::File(f) => println!("File: {}", f.name),
            DirEntry::Directory(d) => println!("Dir: {}", d.name),
        }
    }
    ```

## Error Handling

Each language provides appropriate error handling mechanisms:

=== "Node.js"
    ```typescript
    import { ChunkError, PointerError } from 'autonomi/errors';

    // Handle chunk operations
    try {
        const data = await client.getChunk(address);
        processData(data);
    } catch (error) {
        if (error instanceof ChunkError.NotFound) {
            handleMissing();
        } else if (error instanceof ChunkError.NetworkError) {
            handleNetworkError(error);
        } else {
            handleOtherError(error);
        }
    }

    // Handle pointer updates
    try {
        await client.updatePointer(address, newTarget);
        console.log('Update successful');
    } catch (error) {
        if (error instanceof PointerError.VersionConflict) {
            handleConflict();
        } else {
            handleOtherError(error);
        }
    }
    ```

=== "Python"
    ```python
    from autonomi.errors import ChunkError, PointerError

    # Handle chunk operations
    try:
        data = client.get_chunk(address)
        process_data(data)
    except ChunkError.NotFound:
        handle_missing()
    except ChunkError.NetworkError as e:
        handle_network_error(e)
    except Exception as e:
        handle_other_error(e)

    # Handle pointer updates
    try:
        client.update_pointer(address, new_target)
        print("Update successful")
    except PointerError.VersionConflict:
        handle_conflict()
    except Exception as e:
        handle_other_error(e)
    ```

=== "Rust"
    ```rust
    use autonomi::error::{ChunkError, PointerError, GraphError, ScratchPadError};

    // Handle chunk operations
    match client.get_chunk(address).await {
        Ok(data) => process_data(data),
        Err(ChunkError::NotFound) => handle_missing(),
        Err(ChunkError::NetworkError(e)) => handle_network_error(e),
        Err(e) => handle_other_error(e),
    }

    // Handle pointer updates
    match client.update_pointer(address, new_target).await {
        Ok(_) => println!("Update successful"),
        Err(PointerError::VersionConflict) => handle_conflict(),
        Err(e) => handle_other_error(e),
    }
    ```

## Advanced Usage

### Custom Types

=== "Node.js"
    ```typescript
    interface MyData {
        field1: string;
        field2: number;
    }

    // Store custom type in a scratchpad
    const data: MyData = {
        field1: 'test',
        field2: 42
    };
    const pad = await client.createScratchpad(ContentType.Custom('MyData'));
    await client.updateScratchpad(pad.address, data);
    ```

=== "Python"
    ```python
    from dataclasses import dataclass

    @dataclass
    class MyData:
        field1: str
        field2: int

    # Store custom type in a scratchpad
    data = MyData(field1="test", field2=42)
    pad = client.create_scratchpad(ContentType.CUSTOM("MyData"))
    client.update_scratchpad(pad.address, data)
    ```

=== "Rust"
    ```rust
    use serde::{Serialize, Deserialize};

    #[derive(Serialize, Deserialize)]
    struct MyData {
        field1: String,
        field2: u64,
    }

    // Store custom type in a scratchpad
    let data = MyData {
        field1: "test".into(),
        field2: 42,
    };
    let pad = client.create_scratchpad(ContentType::Custom("MyData")).await?;
    client.update_scratchpad(pad.address(), &data).await?;
    ```

### Encryption

=== "Node.js"
    ```typescript
    import { encrypt, decrypt, generateKey } from 'autonomi/crypto';

    // Encrypt data before storage
    const key = await generateAesKey();
    const encrypted = await encryptAes(data, key);
    const pad = await client.createScratchpad(ContentType.Encrypted);
    await client.updateScratchpad(pad.address, encrypted);

    // Decrypt retrieved data
    const encrypted = await client.getScratchpad(pad.address);
    const decrypted = await decryptAes(encrypted, key);
    ```

=== "Python"
    ```python
    from autonomi.crypto import encrypt_aes, decrypt_aes

    # Encrypt data before storage
    key = generate_aes_key()
    encrypted = encrypt_aes(data, key)
    pad = client.create_scratchpad(ContentType.ENCRYPTED)
    client.update_scratchpad(pad.address, encrypted)

    # Decrypt retrieved data
    encrypted = client.get_scratchpad(pad.address)
    decrypted = decrypt_aes(encrypted, key)
    ```

=== "Rust"
    ```rust
    use autonomi::crypto::{encrypt_aes, decrypt_aes};

    // Encrypt data before storage
    let key = generate_aes_key();
    let encrypted = encrypt_aes(data, &key)?;
    let pad = client.create_scratchpad(ContentType::Encrypted).await?;
    client.update_scratchpad(pad.address(), &encrypted).await?;

    // Decrypt retrieved data
    let encrypted = client.get_scratchpad(pad.address()).await?;
    let decrypted = decrypt_aes(encrypted, &key)?;
    ```

## Best Practices

1. **Data Type Selection**
   - Use Chunks for immutable data
   - Use Pointers for mutable references
   - Use GraphEntrys for ordered collections
   - Use ScratchPads for temporary data

2. **Error Handling**
   - Always handle network errors appropriately
   - Use type-specific error handling
   - Implement retry logic for transient failures

3. **Performance**
   - Use batch operations for multiple items
   - Consider chunking large data sets
   - Cache frequently accessed data locally

4. **Security**
   - Encrypt sensitive data before storage
   - Use secure key management
   - Validate data integrity

## Type System

=== "Node.js"
    ```typescript
    import { Address, Data, Metadata } from 'autonomi/types';

    interface Client {
        storeChunk(data: Buffer): Promise<Address>;
        getChunk(address: Address): Promise<Buffer>;
        createPointer(target: Address): Promise<Pointer>;
        updatePointer(address: Address, target: Address): Promise<void>;
    }
    ```

=== "Python"
    ```python
    from typing import List, Optional, Union
    from autonomi.types import Address, Data, Metadata

    class Client:
        def store_chunk(self, data: bytes) -> Address: ...
        def get_chunk(self, address: Address) -> bytes: ...
        def create_pointer(self, target: Address) -> Pointer: ...
        def update_pointer(self, address: Address, target: Address) -> None: ...
    ```

=== "Rust"
    ```rust
    use autonomi::types::{Address, Data, Metadata};

    pub trait Client {
        async fn store_chunk(&self, data: &[u8]) -> Result<Address>;
        async fn get_chunk(&self, address: &Address) -> Result<Vec<u8>>;
        async fn create_pointer(&self, target: Address) -> Result<Pointer>;
        async fn update_pointer(&self, address: Address, target: Address) -> Result<()>;
    }
    ```

## Further Reading

- [Data Types Guide](../../guides/data_types.md)
- [Client Modes Guide](../../guides/client_modes.md)
- [Local Network Setup](../../guides/local_network.md)
