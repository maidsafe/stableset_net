# Installation Guide

## Prerequisites

- Rust (latest stable)
- Python 3.8 or higher
- Node.js 16 or higher

## API-specific Installation

Choose the APIs you need for your project:

### Autonomi Client

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

### Ant Node

=== "Python"
    ```bash
    pip install antnode
    ```

=== "Rust"
    ```toml
    [dependencies]
    ant-node = "0.3.2"
    ```

### BLS Threshold Crypto

=== "Python"
    ```bash
    pip install blsttc
    ```

=== "Rust"
    ```toml
    [dependencies]
    blsttc = "8.0.2"
    ```

### Self Encryption

=== "Python"
    ```bash
    pip install self-encryption
    ```

=== "Rust"
    ```toml
    [dependencies]
    self_encryption = "0.28.0"
    ```

## Verifying Installation

Test your installation by running a simple client initialization:

=== "Node.js"
    ```typescript
    import { Client } from 'autonomi';

    const client = await Client.initReadOnly();
    console.log('Client initialized successfully');
    ```

=== "Python"
    ```python
    from autonomi import Client

    client = Client.init_read_only()
    print('Client initialized successfully')
    ```

=== "Rust"
    ```rust
    use autonomi::Client;

    let client = Client::new_local().await?;
    println!("Client initialized successfully");
    ```

## Next Steps

- API References:
  - [Autonomi Client](../api/autonomi-client/README.md)
  - [Ant Node](../api/ant-node/README.md)
  - [BLS Threshold Crypto](../api/blsttc/README.md)
  - [Self Encryption](../api/self-encryption/README.md)
- [Local Network Setup](../guides/local_network.md)
