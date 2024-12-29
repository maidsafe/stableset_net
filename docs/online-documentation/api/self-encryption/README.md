# Self Encryption API Reference

A file content self-encryptor that provides convergent encryption on file-based data. It produces a `DataMap` type and several chunks of encrypted data. Each chunk is up to 1MB in size and has an index and a name (SHA3-256 hash of the content), allowing chunks to be self-validating.

## Installation

=== "Python"
    ```bash
    pip install self-encryption
    ```

=== "Rust"
    ```toml
    [dependencies]
    self_encryption = "0.31.0"
    ```

## Core Concepts

### DataMap

Holds the information required to recover the content of the encrypted file, stored as a vector of `ChunkInfo` (list of file's chunk hashes). Only files larger than 3072 bytes (3 * MIN_CHUNK_SIZE) can be self-encrypted.

### Chunk Sizes

- `MIN_CHUNK_SIZE`: 1 byte
- `MAX_CHUNK_SIZE`: 1 MiB (before compression)
- `MIN_ENCRYPTABLE_BYTES`: 3 bytes

## Streaming Operations (Recommended)

### Streaming File Encryption

=== "Python"
    ```python
    from self_encryption import streaming_encrypt_from_file, ChunkStore
    from pathlib import Path
    from typing import Optional

    # Implement your chunk store
    class MyChunkStore(ChunkStore):
        def put(self, name: bytes, data: bytes) -> None:
            # Store the chunk
            pass
            
        def get(self, name: bytes) -> Optional[bytes]:
            # Retrieve the chunk
            pass

    # Create chunk store instance
    store = MyChunkStore()
    
    # Encrypt file using streaming
    file_path = Path("my_file.txt")
    data_map = streaming_encrypt_from_file(file_path, store)
    ```

=== "Rust"
    ```rust
    use self_encryption::{streaming_encrypt_from_file, ChunkStore};
    use std::path::Path;

    // Implement your chunk store
    struct MyChunkStore {
        // Your storage implementation
    }

    impl ChunkStore for MyChunkStore {
        fn put(&mut self, name: &[u8], data: &[u8]) -> Result<(), Error> {
            // Store the chunk
        }
        
        fn get(&self, name: &[u8]) -> Result<Vec<u8>, Error> {
            // Retrieve the chunk
        }
    }

    // Create chunk store instance
    let store = MyChunkStore::new();
    
    // Encrypt file using streaming
    let file_path = Path::new("my_file.txt");
    let data_map = streaming_encrypt_from_file(file_path, store).await?;
    ```

### Streaming File Decryption

=== "Python"
    ```python
    from self_encryption import streaming_decrypt_from_storage
    from pathlib import Path

    # Decrypt to file using streaming
    output_path = Path("decrypted_file.txt")
    streaming_decrypt_from_storage(data_map, store, output_path)
    ```

=== "Rust"
    ```rust
    use self_encryption::streaming_decrypt_from_storage;
    use std::path::Path;

    // Decrypt to file using streaming
    let output_path = Path::new("decrypted_file.txt");
    streaming_decrypt_from_storage(&data_map, store, output_path).await?;
    ```

## In-Memory Operations (Small Files)

### Basic Encryption/Decryption

=== "Python"
    ```python
    from self_encryption import encrypt, decrypt

    # Encrypt bytes in memory
    data = b"Small data to encrypt"
    data_map, encrypted_chunks = encrypt(data)

    # Decrypt using retrieval function
    def get_chunk(name: bytes) -> bytes:
        # Retrieve chunk by name from your storage
        return chunk_data

    decrypted = decrypt(data_map, get_chunk)
    ```

=== "Rust"
    ```rust
    use self_encryption::{encrypt, decrypt};

    // Encrypt bytes in memory
    let data = b"Small data to encrypt";
    let (data_map, encrypted_chunks) = encrypt(data)?;

    // Decrypt using retrieval function
    let decrypted = decrypt(
        &data_map,
        |name| {
            // Retrieve chunk by name from your storage
            Ok(chunk_data)
        }
    )?;
    ```

## Chunk Store Implementations

### In-Memory Store

=== "Python"
    ```python
    from self_encryption import ChunkStore
    from typing import Dict, Optional

    class MemoryStore(ChunkStore):
        def __init__(self):
            self.chunks: Dict[bytes, bytes] = {}

        def put(self, name: bytes, data: bytes) -> None:
            self.chunks[name] = data
            
        def get(self, name: bytes) -> Optional[bytes]:
            return self.chunks.get(name)
    ```

=== "Rust"
    ```rust
    use std::collections::HashMap;

    struct MemoryStore {
        chunks: HashMap<Vec<u8>, Vec<u8>>,
    }

    impl ChunkStore for MemoryStore {
        fn put(&mut self, name: &[u8], data: &[u8]) -> Result<(), Error> {
            self.chunks.insert(name.to_vec(), data.to_vec());
            Ok(())
        }
        
        fn get(&self, name: &[u8]) -> Result<Vec<u8>, Error> {
            self.chunks.get(name)
                .cloned()
                .ok_or(Error::NoSuchChunk)
        }
    }
    ```

### Disk-Based Store

=== "Python"
    ```python
    from pathlib import Path
    from typing import Optional
    import os

    class DiskStore(ChunkStore):
        def __init__(self, root_dir: Path):
            self.root_dir = root_dir
            self.root_dir.mkdir(parents=True, exist_ok=True)

        def put(self, name: bytes, data: bytes) -> None:
            path = self.root_dir / name.hex()
            path.write_bytes(data)
            
        def get(self, name: bytes) -> Optional[bytes]:
            path = self.root_dir / name.hex()
            try:
                return path.read_bytes()
            except FileNotFoundError:
                return None
    ```

=== "Rust"
    ```rust
    use std::path::PathBuf;
    use std::fs;

    struct DiskStore {
        root_dir: PathBuf,
    }

    impl ChunkStore for DiskStore {
        fn put(&mut self, name: &[u8], data: &[u8]) -> Result<(), Error> {
            let path = self.root_dir.join(hex::encode(name));
            fs::write(path, data)?;
            Ok(())
        }
        
        fn get(&self, name: &[u8]) -> Result<Vec<u8>, Error> {
            let path = self.root_dir.join(hex::encode(name));
            fs::read(path).map_err(|_| Error::NoSuchChunk)
        }
    }

    impl DiskStore {
        fn new<P: Into<PathBuf>>(root: P) -> Self {
            let root_dir = root.into();
            fs::create_dir_all(&root_dir).expect("Failed to create store directory");
            Self { root_dir }
        }
    }
    ```

## Error Handling

The library provides an `Error` enum for handling various error cases:

```rust
pub enum Error {
    NoSuchChunk,
    ChunkTooSmall,
    ChunkTooLarge,
    InvalidChunkSize,
    Io(std::io::Error),
    Serialisation(Box<bincode::ErrorKind>),
    Compression(std::io::Error),
    // ... other variants
}
```

## Best Practices

1. Use streaming operations (`streaming_encrypt_from_file` and `streaming_decrypt_from_storage`) for large files
2. Use basic `encrypt`/`decrypt` functions for small in-memory data
3. Implement proper error handling for chunk store operations
4. Verify chunks using their content hash when retrieving
5. Use parallel operations when available for better performance
