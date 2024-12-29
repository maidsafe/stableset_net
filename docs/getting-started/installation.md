# Installation Guide

This guide will help you install the Autonomi client for your preferred programming language.

## Prerequisites

- Node.js 16+ (for Node.js client)
- Python 3.8+ (for Python client)
- Rust toolchain (for Rust client)
- Docker (for local network)

## Node.js Installation

```bash
# Using npm
npm install @autonomi/client

# Using yarn
yarn add @autonomi/client

# Using pnpm
pnpm add @autonomi/client
```

### TypeScript Configuration

Add these settings to your `tsconfig.json`:

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "module": "commonjs",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true
  }
}
```

## Python Installation

```bash
# Using pip
pip install autonomi-client

# Using poetry
poetry add autonomi-client
```

### Virtual Environment (recommended)

```bash
# Create virtual environment
python -m venv venv

# Activate virtual environment
source venv/bin/activate  # Unix
.\venv\Scripts\activate   # Windows

# Install package
pip install autonomi-client
```

## Rust Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
autonomi = "0.1.0"
```

Or using cargo-edit:

```bash
cargo add autonomi
```

## Docker Setup (for Local Network)

1. Install Docker:
   - [Docker Desktop for Mac](https://docs.docker.com/desktop/mac/install/)
   - [Docker Desktop for Windows](https://docs.docker.com/desktop/windows/install/)
   - [Docker Engine for Linux](https://docs.docker.com/engine/install/)

2. Pull the Autonomi image:

```bash
docker pull autonomi/node:latest
```

## Verifying Installation

### Node.js

```typescript
import { Client } from '@autonomi/client';

async function verify() {
    const client = new Client();
    await client.connect();
    console.log('Connected successfully!');
}

verify().catch(console.error);
```

### Python

```python
import asyncio
from autonomi import Client

async def verify():
    client = Client()
    await client.connect()
    print('Connected successfully!')

asyncio.run(verify())
```

### Rust

```rust
use autonomi::Client;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new()?;
    println!("Connected successfully!");
    Ok(())
}
```

## Next Steps

- [Quick Start Guide](quickstart.md)
- [Local Network Setup](../guides/local_network.md)
- [API Reference](../api/README.md)
