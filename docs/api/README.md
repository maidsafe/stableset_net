# API Documentation

## Getting Started

The Autonomi API provides a flexible interface for interacting with the network. You can start with a read-only client for browsing and reading data, then optionally upgrade to write capabilities when needed. See the [Client Modes Guide](../guides/client_modes.md) for details.

## Core Concepts

- **Client Modes**: Choose between read-only and read-write access
- **Data Storage**: Store and retrieve data on the network
- **Linked Lists**: Create and manage linked data structures
- **Pointers**: Reference and update data locations
- **Vaults**: Secure data storage with encryption
- **Payments**: Handle storage payments using EVM wallets

## Language Support

- [Rust API](rust/README.md)
- [Node.js API](nodejs/README.md)
- [Python API](python/README.md)

## Common Use Cases

1. **Read-Only Access**
   - Browse network data
   - Retrieve files and content
   - Query linked lists and pointers

2. **Write Operations**
   - Store public and private data
   - Create and update data structures
   - Manage user data in vaults

3. **Payment Handling**
   - Get storage quotes
   - Make payments for write operations
   - Manage wallet balances

## Best Practices

1. Start with a read-only client for browsing
2. Upgrade to read-write mode only when needed
3. Handle errors appropriately
4. Follow security guidelines for wallet management
5. Use appropriate payment options for write operations

## Further Reading

- [Local Network Setup](../guides/local_network.md)
- [Client Modes Guide](../guides/client_modes.md)
- [Data Storage Guide](../guides/data_storage.md)
