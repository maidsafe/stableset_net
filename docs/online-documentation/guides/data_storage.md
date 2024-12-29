# Data Storage Guide

This guide explains how Autonomi handles data storage, including self-encryption and scratchpad features.

## Self-Encryption

Self-encryption is a core feature that provides secure data storage by splitting and encrypting data into chunks.

### How It Works

1. Data is split into chunks
2. Each chunk is encrypted
3. A data map is created to track the chunks
4. Additional encryption layers are added for larger files

### Usage Examples

=== "Node.js"
    ```typescript
    import { Client } from '@autonomi/client';

    async function storeEncryptedData(data: Uint8Array) {
        const client = new Client();
        
        // Data is automatically self-encrypted when stored
        const address = await client.data_put_public(data);
        console.log(`Data stored at: ${address}`);
        
        // Retrieve and decrypt data
        const retrieved = await client.data_get_public(address);
        console.log('Data retrieved successfully');
    }
    ```

=== "Python"
    ```python
    from autonomi import Client

    async def store_encrypted_data(data: bytes):
        client = Client()
        
        # Data is automatically self-encrypted when stored
        address = await client.data_put_public(data)
        print(f"Data stored at: {address}")
        
        # Retrieve and decrypt data
        retrieved = await client.data_get_public(address)
        print("Data retrieved successfully")
    ```

=== "Rust"
    ```rust
    use autonomi::{Client, Bytes, Result};

    async fn store_encrypted_data(data: Bytes) -> Result<()> {
        let client = Client::new()?;
        
        // Data is automatically self-encrypted when stored
        let address = client.data_put_public(data).await?;
        println!("Data stored at: {}", address);
        
        // Retrieve and decrypt data
        let retrieved = client.data_get_public(&address).await?;
        println!("Data retrieved successfully");
        
        Ok(())
    }
    ```

## Scratchpad

Scratchpad provides a mutable storage location for encrypted data with versioning support.

### Features

- Mutable data storage
- Version tracking with monotonic counter
- Owner-based access control
- Data encryption
- Signature verification

### Usage Examples

=== "Node.js"
    ```typescript
    import { Client, Scratchpad } from '@autonomi/client';

    async function useScratchpad() {
        const client = new Client();
        const secretKey = await client.generate_secret_key();
        
        // Create or get existing scratchpad
        const [scratchpad, isNew] = await client.get_or_create_scratchpad(
            secretKey,
            42 // content type
        );
        
        // Update scratchpad data
        const data = new TextEncoder().encode('Hello World');
        await client.update_scratchpad(scratchpad, data, secretKey);
        
        // Read scratchpad data
        const retrieved = await client.get_scratchpad(scratchpad.address);
        const decrypted = await client.decrypt_scratchpad(retrieved, secretKey);
        console.log(new TextDecoder().decode(decrypted));
    }
    ```

=== "Python"
    ```python
    from autonomi import Client, Scratchpad

    async def use_scratchpad():
        client = Client()
        secret_key = client.generate_secret_key()
        
        # Create or get existing scratchpad
        scratchpad, is_new = await client.get_or_create_scratchpad(
            secret_key,
            42  # content type
        )
        
        # Update scratchpad data
        data = b"Hello World"
        await client.update_scratchpad(scratchpad, data, secret_key)
        
        # Read scratchpad data
        retrieved = await client.get_scratchpad(scratchpad.address)
        decrypted = await client.decrypt_scratchpad(retrieved, secret_key)
        print(decrypted.decode())
    ```

=== "Rust"
    ```rust
    use autonomi::{Client, Scratchpad, SecretKey, Bytes, Result};

    async fn use_scratchpad() -> Result<()> {
        let client = Client::new()?;
        let secret_key = SecretKey::random();
        
        // Create or get existing scratchpad
        let (mut scratchpad, is_new) = client
            .get_or_create_scratchpad(&secret_key, 42)
            .await?;
        
        // Update scratchpad data
        let data = Bytes::from("Hello World");
        scratchpad.update_and_sign(data, &secret_key);
        
        // Store updated scratchpad
        client.put_scratchpad(&scratchpad).await?;
        
        // Read scratchpad data
        let retrieved = client.get_scratchpad(scratchpad.address()).await?;
        let decrypted = retrieved.decrypt_data(&secret_key)?;
        println!("Data: {}", String::from_utf8_lossy(&decrypted));
        
        Ok(())
    }
    ```

### Best Practices

1. **Version Management**
   - Always check the counter before updates
   - Handle version conflicts appropriately
   - Use monotonic counters for ordering

2. **Security**
   - Keep secret keys secure
   - Verify signatures before trusting data
   - Always encrypt sensitive data

3. **Error Handling**
   - Handle decryption failures gracefully
   - Implement proper retry logic for network operations
   - Validate data before storage

4. **Performance**
   - Cache frequently accessed data
   - Batch updates when possible
   - Monitor storage size

## Implementation Details

### Self-Encryption Process

1. **Data Splitting**

   ```rust
   // Internal process when storing data
   let (data_map, chunks) = self_encryption::encrypt(data)?;
   let (data_map_chunk, additional_chunks) = pack_data_map(data_map)?;
   ```

2. **Chunk Management**
   - Each chunk is stored separately
   - Chunks are encrypted individually
   - Data maps track chunk locations

### Scratchpad Structure

```rust
pub struct Scratchpad {
    // Network address
    address: ScratchpadAddress,
    // Data type identifier
    data_encoding: u64,
    // Encrypted content
    encrypted_data: Bytes,
    // Version counter
    counter: u64,
    // Owner's signature
    signature: Option<Signature>,
}
```

## Advanced Topics

### Custom Data Types

You can use scratchpads to store any custom data type by implementing proper serialization:

```rust
#[derive(Serialize, Deserialize)]
struct CustomData {
    field1: String,
    field2: u64,
}

// Serialize before storing
let custom_data = CustomData {
    field1: "test".into(),
    field2: 42,
};
let bytes = serde_json::to_vec(&custom_data)?;
scratchpad.update_and_sign(Bytes::from(bytes), &secret_key);
```

### Batch Operations

For better performance when dealing with multiple data items:

```rust
async fn batch_store(items: Vec<Bytes>) -> Result<Vec<ChunkAddress>> {
    let mut addresses = Vec::new();
    for item in items {
        let (data_map_chunk, chunks) = encrypt(item)?;
        // Store chunks in parallel
        futures::future::join_all(chunks.iter().map(|c| store_chunk(c))).await;
        addresses.push(data_map_chunk.address());
    }
    Ok(addresses)
}
```
