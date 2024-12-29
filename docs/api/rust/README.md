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
