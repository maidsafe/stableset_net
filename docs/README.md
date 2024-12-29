# Autonomi Documentation

<div align="center">
<h1>The Alternative Future of the Cloud</h1>
<h3>Autonomous, Secure, Perpetual Data - Powered by You</h3>
</div>

Welcome to Autonomi, a new layer of the internet built from a multitude of everyday devices. Our platform is deliberately designed to allow even the smallest computers to contribute, making it truly decentralized and accessible to all.

## Core Features

### 🌐 Autonomous & Distributed

Control remains in the hands of users rather than centralized entities. Node operators play a crucial role and are directly rewarded by the network itself.

### 🔒 Quantum Secure

Built with cutting-edge quantum security protocols, ensuring unmatched safety for your data. Our network helps fortify the world's data against present and future security threats.

### 💾 Perpetual Data

True self-sovereignty over your digital life with permanent storage that's not just an enticing pricing proposition - it's vital for unhindered access to your data.

## Getting Started

### Quick Start

```bash
# Install Autonomi client
pip install autonomi

# Initialize a read-only client
from autonomi import Client
client = Client.init_read_only()

# Store and retrieve data
pointer = await client.store_file("example.txt", b"Hello, Autonomi!")
data = await client.get_file(pointer)
```

### Running a Node

```bash
# Install node software
pip install antnode

# Start your node
from antnode import AntNode
node = AntNode()
node.run(
    rewards_address="your-address",
    evm_network="arbitrum_sepolia"
)
```

## Core Components

### 1. Fundamental Data Types

- [**Chunk**](guides/data_types.md#chunk) - Quantum-secure encrypted data blocks
- [**Pointer**](guides/data_types.md#pointer) - Mutable references with version tracking
- [**LinkedList**](guides/data_types.md#linkedlist) - Transaction chains and DAGs
- [**ScratchPad**](guides/data_types.md#scratchpad) - Temporary workspace with CRDT properties

### 2. Supporting Libraries

- [**Self-Encryption**](libraries/self_encryption.md) - Quantum-secure data encryption
- [**Antnode**](libraries/antnode.md) - Core node implementation
- [**BLS Threshold Cryptography**](libraries/blsttc.md) - High-performance cryptographic operations

## Language Support

### 🦀 Rust

```rust
use autonomi::Client;

let client = Client::init_read_only().await?;
let pointer = client.store_file("example.txt", data).await?;
```

[View Rust Documentation](api/rust/README.md)

### 🐍 Python

```python
from autonomi import Client

client = Client.init_read_only()
pointer = await client.store_file("example.txt", data)
```

[View Python Documentation](api/python/README.md)

### 📜 TypeScript/Node.js

```typescript
import { Client } from 'autonomi';

const client = await Client.initReadOnly();
const pointer = await client.storeFile("example.txt", data);
```

[View TypeScript Documentation](api/nodejs/README.md)

## Network Features

### No Complicated Setup

Get running in minutes with our streamlined setup process. Our platform is designed to be accessible to everyone, regardless of technical expertise.

### Secure by Design

- Encrypted, duplicated, and randomly distributed data
- Quantum-secure protocols
- Threshold cryptography
- Forward secrecy

### Earn While Contributing

Put your spare space to use and get paid for contributing to the network. Node operators are rewarded directly by the network for their participation.

## Documentation Sections

- [📚 Guides](guides/README.md) - Step-by-step tutorials and how-tos
- [🔧 API Reference](api/README.md) - Detailed API documentation
- [🛠️ Libraries](libraries/README.md) - Supporting library documentation
- [📖 White Paper](https://autonomi.com/whitepaper) - Technical details and architecture

## Join the Community

- [Discord](https://discord.gg/autonomi)
- [Forum](https://forum.autonomi.com)
- [X (Twitter)](https://twitter.com/autonomi)
- [Reddit](https://reddit.com/r/autonomi)
- [LinkedIn](https://linkedin.com/company/autonomi)

## Contributing

Help shape the future of the internet by contributing to Autonomi:

1. Run a node
2. Contribute code
3. Report issues
4. Improve documentation
5. Join community discussions

## Learn More

- [Local Network Setup](guides/local_network.md)
- [Client Modes Guide](guides/client_modes.md)
- [Data Types Guide](guides/data_types.md)
- [Security Guide](guides/security.md)
- [Performance Guide](guides/performance.md)

<div align="center">
<h2>Shape the Future as the New Internet Assembles Itself</h2>
</div>
