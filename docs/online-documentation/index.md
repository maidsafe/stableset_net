# Autonomi Documentation

Welcome to the Autonomi documentation! This guide will help you get started with using the Autonomi network client.

## What is Autonomi?

Autonomi is a decentralized network client that provides:

- Distributed data storage and retrieval
- EVM network integration
- Secure pointer management
- Linked list data structures

## Quick Links

- [Installation Guide](getting-started/installation.md)
- [Quick Start Guide](getting-started/quickstart.md)
- API References:
  - [Autonomi Client](api/autonomi-client/README.md) - Core client library for network operations
  - [Ant Node](api/ant-node/README.md) - Node implementation for network participation
  - [BLS Threshold Crypto](api/blsttc/README.md) - Threshold cryptography implementation
  - [Self Encryption](api/self-encryption/README.md) - Content-based encryption library
- [Local Network Setup](guides/local_network.md)

## Language Support

Autonomi provides client libraries for multiple languages:

=== "Node.js"
    ```typescript
    import { Client } from 'autonomi';

    const client = new Client();
    await client.connect();
    ```

=== "Python"
    ```python
    from autonomi import Client

    client = Client()
    await client.connect()
    ```

=== "Rust"
    ```rust
    use autonomi::Client;

    let client = Client::new()?;
    ```

## Building from Source

=== "Python (using Maturin & uv)"
    ```bash
    # Install build dependencies
    curl -LsSf <https://astral.sh/uv/install.sh> | sh
    uv pip install maturin

    # Clone the repository
    git clone https://github.com/dirvine/autonomi.git
    cd autonomi

    # Create and activate virtual environment
    uv venv
    source .venv/bin/activate  # Unix
    # or
    .venv\Scripts\activate     # Windows

    # Build and install the package
    cd python
    maturin develop
    
    # Install dependencies
    uv pip install -r requirements.txt
    ```

=== "Node.js"
    ```bash
    # Install build dependencies
    npm install -g node-gyp

    # Clone the repository
    git clone https://github.com/dirvine/autonomi.git
    cd autonomi

    # Build the Node.js bindings
    cd nodejs
    npm install
    npm run build

    # Link for local development
    npm link
    ```

=== "Rust"
    ```bash
    # Clone the repository
    git clone <https://github.com/dirvine/autonomi.git>
    cd autonomi

    # Build the project
    cargo build --release

    # Run tests
    cargo test --all-features

    # Install locally
    cargo install --path .
    ```

## Contributing

We welcome contributions! Here's how you can help:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Submit a pull request

For more details, see our [Contributing Guide](https://github.com/dirvine/autonomi/blob/main/CONTRIBUTING.md).

## Getting Help

- [GitHub Issues](https://github.com/dirvine/autonomi/issues)
- API References:
  - [Autonomi Client](api/autonomi-client/README.md)
  - [Ant Node](api/ant-node/README.md)
  - [BLS Threshold Crypto](api/blsttc/README.md)
  - [Self Encryption](api/self-encryption/README.md)
- [Testing Guide](guides/testing_guide.md)
