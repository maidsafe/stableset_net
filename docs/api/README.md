# Autonomi API Reference

Autonomi is a decentralized storage and computation platform that enables developers to build secure, scalable applications. Our API provides a consistent interface across multiple programming languages.

<div class="language-selector">
<select onchange="switchLanguage(this.value)">
  <option value="rust">Rust</option>
  <option value="python">Python</option>
  <option value="typescript">TypeScript/Node.js</option>
</select>
</div>

<div class="language-content" id="rust-content">

## Rust API

### Installation

```toml
[dependencies]
autonomi = "0.1.0"

# With optional features
autonomi = { version = "0.1.0", features = ["fs", "registers", "vault"] }
```

Available features:

- `fs`: File system operations (up/download files and directories)
- `registers`: Register datatype operations
- `vault`: Vault datatype operations
- `local`: Local peer discovery using mDNS (for development)

### Client Initialization

```{.rust .light}
use autonomi::Client;

// Initialize with default configuration
let client = Client::init().await?;

// Initialize with custom configuration
let config = ClientConfig {
    local: true,  // For local development
    peers: Some(vec!["/ip4/127.0.0.1/tcp/5000".parse()?]),
};
let client = Client::init_with_config(config).await?;

// Initialize with wallet for write access
let client = Client::init_with_wallet(wallet).await?;
```

### Data Operations

```{.rust .light}
// Store and retrieve data
let data = b"Hello, World!";
let data_addr = client.data_put_public(Bytes::from(data), (&wallet).into()).await?;
let retrieved = client.data_get_public(data_addr).await?;

// File operations (with 'fs' feature)
let dir_addr = client.dir_and_archive_upload_public("files/to/upload".into(), &wallet).await?;
client.dir_download_public(dir_addr, "files/downloaded".into()).await?;
```

### Error Handling

```{.rust .light}
use autonomi::PutError;

match client.data_put_public(data, payment).await {
    Ok(addr) => println!("Stored at: {}", addr),
    Err(PutError::InsufficientBalance) => println!("Need more funds"),
    Err(e) => println!("Error: {}", e),
}
```

</div>

<div class="language-content" id="python-content" style="display: none;">

## Python API

### Installation

```bash
pip install autonomi
```

### Client Initialization

```{.python .light}
from autonomi import Client

# Initialize read-only client
client = Client.init_read_only()

# Initialize with wallet for write access
client = Client.init_with_wallet(wallet)
```

### Data Operations

```{.python .light}
# Store and retrieve data
data = b"Hello, World!"
data_addr = client.data_put_public(data, payment)
retrieved = client.data_get_public(data_addr)

# File operations
dir_addr = client.dir_and_archive_upload_public("files/to/upload", wallet)
client.dir_download_public(dir_addr, "files/downloaded")
```

### Error Handling

```{.python .light}
try:
    addr = client.data_put_public(data, payment)
except InsufficientBalanceError:
    print("Need more funds")
except NetworkError as e:
    print(f"Network error: {e}")
```

</div>

<div class="language-content" id="typescript-content" style="display: none;">

## TypeScript/Node.js API

### Installation

```bash
npm install autonomi
```

### Client Initialization

```{.typescript .light}
import { Client } from 'autonomi';

// Initialize client
const client = await Client.connect({
    local: true,  // For local development
    peers: ["/ip4/127.0.0.1/tcp/5000"]
});
```

### Data Operations

```{.typescript .light}
// Store and retrieve data
const data = Buffer.from("Hello, World!");
const dataAddr = await client.dataPutPublic(data, payment);
const retrieved = await client.dataGetPublic(dataAddr);

// Linked List operations
const list = await client.linkedListGet(address);
await client.linkedListPut(options, payment);

// Pointer operations
const pointer = await client.pointerGet(address);
```

### Error Handling

```{.typescript .light}
try {
    const addr = await client.dataPutPublic(data, payment);
} catch (error) {
    if (error instanceof InsufficientBalanceError) {
        console.log("Need more funds");
    } else {
        console.error("Error:", error);
    }
}
```

</div>

<script>
function switchLanguage(lang) {
    document.querySelectorAll('.language-content').forEach(el => {
        el.style.display = 'none';
    });
    document.getElementById(lang + '-content').style.display = 'block';
}
</script>

<style>
.language-selector {
    margin: 20px 0;
}
.language-selector select {
    padding: 8px 16px;
    font-size: 16px;
    border: 1px solid #ddd;
    border-radius: 4px;
    background-color: white;
}
</style>

## Core Components

### Data Storage

Store and retrieve data with content-addressed chunks. Data is split into chunks using self-encryption, yielding a 'data map' for reconstruction.

### Registers

Keep small values pointing to data with update history. Supports concurrent updates with eventual consistency.

### File System Operations

Upload and download files and directories with the `fs` feature enabled.

## Best Practices

1. **Error Handling**
   - Implement proper error handling for network operations
   - Handle insufficient balance errors for write operations
   - Use retry logic for network operations

2. **Performance**
   - Use streaming for large files
   - Enable appropriate features for your use case
   - Handle resource cleanup properly

3. **Development**
   - Use local mode for development and testing
   - Monitor network connectivity
   - Implement proper error recovery

## Further Reading

- [Getting Started Guide](../guides/getting_started.md)
- [Error Handling Guide](../guides/error_handling.md)
- [Examples Repository](https://github.com/autonomi/examples)
