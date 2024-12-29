# Installation Guide

## Prerequisites

- Rust (latest stable)
- Python 3.8 or higher
- Node.js 16 or higher

## Language-specific Installation

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

- [Quick Start Guide](quickstart.md)
- [API Reference](../api/README.md)
- [Local Network Setup](../guides/local_network.md)
