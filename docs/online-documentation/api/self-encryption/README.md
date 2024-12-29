# Self Encryption API Reference

Self Encryption is a novel encryption design that encrypts data using its own content as the encryption key. This provides both encryption and content-based deduplication.

## Installation

=== "Python"
    ```bash
    # Install using uv (recommended)
    curl -LsSf <https://astral.sh/uv/install.sh> | sh
    uv pip install self-encryption

    # Or using pip
    pip install self-encryption
    ```

=== "Rust"
    ```toml
    # Add to Cargo.toml
    [dependencies]
    self_encryption = "0.28.0"
    ```

## Basic Usage

=== "Python"
    ```python
    from self_encryption import SelfEncryptor, Storage

    # Create a storage backend
    class MemoryStorage(Storage):
        def __init__(self):
            self.data = {}

        def get(self, name):
            return self.data.get(name)

        def put(self, name, data):
            self.data[name] = data
            return name

        def delete(self, name):
            del self.data[name]

    # Create a self encryptor
    storage = MemoryStorage()
    encryptor = SelfEncryptor(storage)

    # Encrypt data
    data = b"Hello, World!"
    data_map = encryptor.write(data)

    # Decrypt data
    decryptor = SelfEncryptor(storage, data_map)
    decrypted = decryptor.read()
    assert data == decrypted
    ```

=== "Rust"
    ```rust
    use self_encryption::{SelfEncryptor, Storage};
    use std::collections::HashMap;

    // Create a storage backend
    #[derive(Default)]
    struct MemoryStorage {
        data: HashMap<Vec<u8>, Vec<u8>>,
    }

    impl Storage for MemoryStorage {
        fn get(&self, name: &[u8]) -> Result<Vec<u8>> {
            self.data.get(name).cloned()
                .ok_or_else(|| Error::NoSuchChunk)
        }

        fn put(&mut self, name: Vec<u8>, data: Vec<u8>) -> Result<Vec<u8>> {
            self.data.insert(name.clone(), data);
            Ok(name)
        }

        fn delete(&mut self, name: &[u8]) -> Result<()> {
            self.data.remove(name);
            Ok(())
        }
    }

    // Create a self encryptor
    let storage = MemoryStorage::default();
    let encryptor = SelfEncryptor::new(storage, DataMap::None)?;

    // Encrypt data
    let data = b"Hello, World!";
    encryptor.write(data)?;
    let data_map = encryptor.close()?;

    // Decrypt data
    let decryptor = SelfEncryptor::new(storage, data_map)?;
    let decrypted = decryptor.read(0, data.len() as u64)?;
    assert_eq!(data[..], decrypted[..]);
    ```

## Advanced Features

### Parallel Processing

=== "Python"
    ```python
    from self_encryption import parallel_encrypt, parallel_decrypt

    # Encrypt data in parallel
    data = b"Large data to encrypt..."
    data_map = parallel_encrypt(storage, data, num_threads=4)

    # Decrypt data in parallel
    decrypted = parallel_decrypt(storage, data_map, num_threads=4)
    ```

=== "Rust"
    ```rust
    use self_encryption::parallel::{encrypt, decrypt};
    use rayon::prelude::*;

    // Encrypt data in parallel
    let data = b"Large data to encrypt...";
    let data_map = encrypt(storage, data)?;

    // Decrypt data in parallel
    let decrypted = decrypt(storage, &data_map)?;
    ```

### Streaming Interface

=== "Python"
    ```python
    # Write data in chunks
    encryptor = SelfEncryptor(storage)
    for chunk in chunks:
        encryptor.write_chunk(chunk)
    data_map = encryptor.close()

    # Read data in chunks
    decryptor = SelfEncryptor(storage, data_map)
    while chunk := decryptor.read_chunk():
        process_chunk(chunk)
    ```

=== "Rust"
    ```rust
    // Write data in chunks
    let mut encryptor = SelfEncryptor::new(storage, DataMap::None)?;
    for chunk in chunks {
        encryptor.write_chunk(chunk)?;
    }
    let data_map = encryptor.close()?;

    // Read data in chunks
    let mut decryptor = SelfEncryptor::new(storage, data_map)?;
    while let Some(chunk) = decryptor.read_chunk()? {
        process_chunk(chunk);
    }
    ```

### Custom Storage Backends

=== "Python"
    ```python
    from self_encryption import Storage
    from typing import Optional

    class CustomStorage(Storage):
        def get(self, name: bytes) -> Optional[bytes]:
            # Implement retrieval logic
            pass

        def put(self, name: bytes, data: bytes) -> bytes:
            # Implement storage logic
            return name

        def delete(self, name: bytes) -> None:
            # Implement deletion logic
            pass
    ```

=== "Rust"
    ```rust
    use self_encryption::{Storage, Error, Result};

    struct CustomStorage;

    impl Storage for CustomStorage {
        fn get(&self, name: &[u8]) -> Result<Vec<u8>> {
            // Implement retrieval logic
            unimplemented!()
        }

        fn put(&mut self, name: Vec<u8>, data: Vec<u8>) -> Result<Vec<u8>> {
            // Implement storage logic
            unimplemented!()
        }

        fn delete(&mut self, name: &[u8]) -> Result<()> {
            // Implement deletion logic
            unimplemented!()
        }
    }
    ```

## Error Handling

=== "Python"
    ```python
    from self_encryption import SelfEncryptionError

    try:
        data = encryptor.read()
    except SelfEncryptionError as e:
        if isinstance(e, ChunkNotFound):
            print("Missing data chunk")
        elif isinstance(e, InvalidDataMap):
            print("Invalid data map")
        else:
            print(f"Other error: {e}")
    ```

=== "Rust"
    ```rust
    use self_encryption::Error;

    match encryptor.read(0, size) {
        Ok(data) => process_data(data),
        Err(Error::ChunkNotFound) => println!("Missing data chunk"),
        Err(Error::InvalidDataMap) => println!("Invalid data map"),
        Err(e) => println!("Other error: {}", e),
    }
    ```

## Best Practices

1. **Data Handling**
   - Use appropriate chunk sizes
   - Handle large files efficiently
   - Implement proper cleanup

2. **Storage Management**
   - Implement robust storage backends
   - Handle storage errors gracefully
   - Clean up unused chunks

3. **Performance**
   - Use parallel processing for large files
   - Implement efficient storage backends
   - Cache frequently accessed data

4. **Security**
   - Secure storage of data maps
   - Implement proper access control
   - Regular backup of critical data

## Common Use Cases

1. **File Storage**
   - Secure file storage
   - Content-based deduplication
   - Efficient large file handling

2. **Backup Systems**
   - Incremental backups
   - Deduplication
   - Secure storage

3. **Content Distribution**
   - Distributed content delivery
   - Content verification
   - Bandwidth optimization
