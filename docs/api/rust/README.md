# Rust API Reference

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
autonomi = "0.1.0"
```

## Core Types

### Client

The main interface for interacting with the Autonomi network.

```rust
use autonomi::{Client, LinkedList, Pointer, Result};

pub struct Client {
    // ... implementation details ...
}

impl Client {
    /// Create a new client with default configuration
    pub fn new() -> Result<Self>;
    
    /// Create a new client with custom configuration
    pub fn with_config(config: ClientConfig) -> Result<Self>;
    
    /// Store a linked list in the network
    pub fn linked_list_put(&self, list: &LinkedList) -> Result<LinkedListAddress>;
    
    /// Retrieve a linked list from the network
    pub fn linked_list_get(&self, address: &LinkedListAddress) -> Result<LinkedList>;
    
    /// Store a pointer in the network
    pub fn pointer_put(&self, pointer: &Pointer) -> Result<PointerAddress>;
    
    /// Retrieve a pointer from the network
    pub fn pointer_get(&self, address: &PointerAddress) -> Result<Pointer>;
}
```

### LinkedList

Represents a linked list data structure.

```rust
pub struct LinkedList {
    // ... implementation details ...
}

impl LinkedList {
    /// Create a new empty linked list
    pub fn new() -> Self;
    
    /// Append data to the list
    pub fn append<T: Into<Vec<u8>>>(&mut self, data: T);
    
    /// Prepend data to the list
    pub fn prepend<T: Into<Vec<u8>>>(&mut self, data: T);
    
    /// Remove an item at the specified index
    pub fn remove(&mut self, index: usize) -> Result<()>;
    
    /// Get an item at the specified index
    pub fn get(&self, index: usize) -> Option<&[u8]>;
}
```

### Pointer

Represents a pointer in the network.

```rust
pub struct Pointer {
    // ... implementation details ...
}

impl Pointer {
    /// Create a new pointer
    pub fn new() -> Self;
    
    /// Set the target of the pointer
    pub fn set_target<T: Into<String>>(&mut self, target: T);
    
    /// Get the target of the pointer
    pub fn target(&self) -> &str;
    
    /// Check if the pointer is valid
    pub fn is_valid(&self) -> bool;
}
```

### Scratchpad

Represents a mutable storage location with versioning and encryption.

```rust
pub struct ScratchpadConfig {
    pub content_type: u64,
    pub data: Vec<u8>,
    pub secret_key: SecretKey,
}

pub struct Scratchpad {
    // ... implementation details ...
}

impl Scratchpad {
    /// Create a new scratchpad with the given configuration
    pub fn new(config: ScratchpadConfig) -> Result<Self>;
    
    /// Get the network address of the scratchpad
    pub fn address(&self) -> &str;
    
    /// Get the current version counter
    pub fn counter(&self) -> u64;
    
    /// Update the data and sign with secret key
    pub fn update_and_sign(&mut self, data: Vec<u8>, secret_key: &SecretKey) -> Result<()>;
    
    /// Verify the signature
    pub fn verify(&self) -> bool;
    
    /// Decrypt the data using the secret key
    pub fn decrypt(&self, secret_key: &SecretKey) -> Result<Vec<u8>>;
}

### Self-Encryption

Utilities for data encryption and chunking.

```rust
pub struct ChunkInfo {
    pub hash: String,
    pub size: usize,
    pub offset: usize,
}

pub struct DataMap {
    pub chunks: Vec<ChunkInfo>,
    pub total_size: usize,
}

pub struct EncryptionResult {
    pub data_map: DataMap,
    pub chunks: Vec<Vec<u8>>,
}

pub mod self_encryption {
    use super::*;
    
    /// Encrypt and chunk the data
    pub fn encrypt(data: &[u8]) -> Result<EncryptionResult>;
    
    /// Decrypt and reassemble the data
    pub fn decrypt(data_map: &DataMap, chunks: &[Vec<u8>]) -> Result<Vec<u8>>;
    
    /// Pack a data map into a chunk
    pub fn pack_data_map(data_map: &DataMap) -> Result<Vec<u8>>;
}

### Files and Directories

Utilities for managing files and directories in the network.

```rust
use chrono::{DateTime, Utc};

pub struct FileMetadata {
    pub name: String,
    pub size: u64,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    pub content_type: String,
}

pub enum DirectoryEntry {
    File {
        name: String,
        metadata: FileMetadata,
    },
    Directory {
        name: String,
    },
}

pub struct File {
    // ... implementation details ...
}

impl File {
    /// Create a new file with optional initial data
    pub fn new(name: &str, data: Option<Vec<u8>>) -> Result<Self>;
    
    /// Get file metadata
    pub fn metadata(&self) -> &FileMetadata;
    
    /// Read file contents
    pub async fn read(&self) -> Result<Vec<u8>>;
    
    /// Write file contents
    pub async fn write(&mut self, data: Vec<u8>) -> Result<()>;
    
    /// Update file metadata
    pub async fn update_metadata(&mut self, metadata: FileMetadata) -> Result<()>;
}

pub struct Directory {
    // ... implementation details ...
}

impl Directory {
    /// Create a new directory
    pub fn new(name: &str) -> Result<Self>;
    
    /// List directory contents
    pub async fn list(&self) -> Result<Vec<DirectoryEntry>>;
    
    /// Create a new file
    pub async fn create_file(&mut self, name: &str, data: Option<Vec<u8>>) -> Result<File>;
    
    /// Create a new subdirectory
    pub async fn create_directory(&mut self, name: &str) -> Result<Directory>;
    
    /// Get a file or directory by name
    pub async fn get(&self, name: &str) -> Result<DirectoryEntry>;
    
    /// Delete a file or directory
    pub async fn delete(&mut self, name: &str) -> Result<()>;
}

### Archive

Utilities for creating and managing archives.

```rust
pub enum Compression {
    None,
    Gzip,
    Bzip2,
}

pub struct EncryptionConfig {
    pub algorithm: String,  // "aes-256-gcm"
    pub key: Vec<u8>,
}

pub struct ArchiveOptions {
    pub compression: Compression,
    pub encryption: Option<EncryptionConfig>,
}

pub struct ArchiveEntry {
    pub name: String,
    pub size: u64,
    pub compressed: bool,
    pub encrypted: bool,
}

pub struct Archive {
    // ... implementation details ...
}

impl Archive {
    /// Create a new archive with optional configuration
    pub fn new(options: Option<ArchiveOptions>) -> Result<Self>;
    
    /// Add a file or directory to the archive
    pub async fn add<P: AsRef<str>>(&mut self, path: P, source: &(impl AsRef<File> + AsRef<Directory>)) -> Result<()>;
    
    /// Extract files from the archive
    pub async fn extract(&self, destination: &mut Directory, pattern: Option<&str>) -> Result<()>;
    
    /// List archive contents
    pub async fn list(&self) -> Result<Vec<ArchiveEntry>>;
    
    /// Verify archive integrity
    pub async fn verify(&self) -> Result<bool>;
}

### Vault

Secure storage for sensitive data.

```rust
pub struct VaultConfig {
    pub secret_key: Vec<u8>,
    pub algorithm: String,  // "aes-256-gcm" or "xchacha20-poly1305"
    pub iterations: u32,
}

pub struct VaultEntry {
    pub key: String,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    pub tags: Option<Vec<String>>,
}

pub struct Vault {
    // ... implementation details ...
}

impl Vault {
    /// Create a new vault with the given configuration
    pub fn new(config: VaultConfig) -> Result<Self>;
    
    /// Store encrypted data
    pub async fn put(&mut self, key: &str, data: Vec<u8>, tags: Option<Vec<String>>) -> Result<()>;
    
    /// Retrieve and decrypt data
    pub async fn get(&self, key: &str) -> Result<Vec<u8>>;
    
    /// List vault contents
    pub async fn list(&self, tag: Option<&str>) -> Result<Vec<VaultEntry>>;
    
    /// Delete data
    pub async fn delete(&mut self, key: &str) -> Result<()>;
    
    /// Rotate encryption key
    pub async fn rotate_key(&mut self, new_key: Vec<u8>) -> Result<()>;
}

## Error Handling

```rust
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    /// Network-related errors
    Network(String),
    /// Storage-related errors
    Storage(String),
    /// Invalid data format
    InvalidData(String),
    /// Other errors
    Other(String),
}

impl std::error::Error for Error {}
```

## Examples

### Basic Usage

```rust
use autonomi::{Client, LinkedList, Result};

fn main() -> Result<()> {
    // Create a new client
    let client = Client::new()?;
    
    // Create and store a linked list
    let mut list = LinkedList::new();
    list.append("Hello");
    list.append("World");
    
    let address = client.linked_list_put(&list)?;
    println!("List stored at: {}", address);
    
    // Retrieve the list
    let retrieved = client.linked_list_get(&address)?;
    println!("{}", retrieved);
    
    Ok(())
}
```

### Error Handling

```rust
use autonomi::{Client, Error};

fn main() {
    match Client::new() {
        Ok(client) => {
            // Use the client
            println!("Client created successfully");
        }
        Err(Error::Network(msg)) => {
            eprintln!("Network error: {}", msg);
        }
        Err(Error::Storage(msg)) => {
            eprintln!("Storage error: {}", msg);
        }
        Err(e) => {
            eprintln!("Other error: {}", e);
        }
    }
}
```

## Best Practices

1. Use proper error handling with `Result` types
2. Implement proper resource cleanup with `Drop` trait
3. Use strong typing and avoid unwrap/expect
4. Follow Rust's ownership and borrowing rules
5. Use async/await for network operations when available
