# Node.js/TypeScript API Documentation

The TypeScript/Node.js implementation of Autonomi provides a modern, type-safe interface for web and server applications. It's ideal for:

- Building decentralized web applications
- Creating secure backend services
- Developing real-time applications
- Integration with modern web frameworks

## Installation

```bash
# Install with npm
npm install autonomi

# Or with specific features
npm install autonomi@latest quantum-secure compression

# For web applications
npm install autonomi@latest web
```

## Client Initialization

The client provides flexible initialization options to match your application needs:

```{.typescript .light}
import { Client, ClientConfig } from 'autonomi';

// Initialize a read-only client for browsing
const client = await Client.initReadOnly();

// Initialize with write capabilities and custom configuration
const config: ClientConfig = {
    quantumSecurity: true,
    compression: true,
    cacheSize: '1GB',
    webSocket: true
};
const client = await Client.initWithWalletAndConfig(wallet, config);

// Upgrade a read-only client to read-write
await client.upgradeToReadWrite(wallet);
```

## Core Data Types

### Chunk - Quantum-Secure Storage

Store and retrieve immutable, quantum-secure encrypted data with streaming support:

```{.typescript .light}
import { Chunk, ChunkOptions } from 'autonomi';

// Store raw data as a chunk
const data = Buffer.from('Hello, World!');
const chunk = await client.storeChunk(data);

// Store large file with streaming
const stream = createReadStream('large-file.dat');
const chunk = await client.storeChunkStream(stream, {
    compression: true,
    chunkSize: '1MB'
});

// Retrieve chunk data with streaming
const retrieved = await client.getChunk(chunk.address);
assert(Buffer.compare(data, retrieved) === 0);

// Stream large chunks
const stream = await client.getChunkStream(chunk.address);
stream.pipe(createWriteStream('output.dat'));

// Get chunk metadata including storage metrics
const metadata = await client.getChunkMetadata(chunk.address);
console.log(`Size: ${metadata.size}, Replicas: ${metadata.replicas}`);

// Store multiple chunks efficiently
const chunks = await client.storeChunks(dataList);
```

### Pointer - Mutable References

Create and manage version-tracked references with real-time updates:

```{.typescript .light}
import { Pointer, PointerOptions } from 'autonomi';

// Create a pointer with metadata
const metadata = {
    createdAt: new Date(),
    description: 'Latest application state'
};
const pointer = await client.createPointerWithMetadata(
    targetAddress,
    metadata
);

// Update pointer with version checking
await client.updatePointer(pointer.address, newTargetAddress);

// Subscribe to pointer updates
client.subscribeToPointer(pointer.address, (update) => {
    console.log(`New target: ${update.target}`);
});

// Get pointer metadata and version history
const metadata = await client.getPointerMetadata(pointer.address);
console.log(`Version: ${metadata.version}, Updates: ${metadata.updateCount}`);
```

### LinkedList - Transaction Chains

Build decentralized DAG structures with real-time synchronization:

```{.typescript .light}
import { LinkedList, LinkedListConfig } from 'autonomi';

// Create a new linked list with configuration
const config: LinkedListConfig = {
    forkDetection: true,
    historyCompression: true,
    realtime: true
};
const list = await client.createLinkedListWithConfig(config);

// Efficient batch appends
await client.appendToListBatch(list.address, items);

// Subscribe to list updates
client.subscribeToList(list.address, (update) => {
    console.log(`New item: ${update.data}`);
});

// Advanced fork detection and resolution
const forks = await client.detectForksDetailed(list.address);
if (!forks) {
    console.log('No forks detected');
} else {
    const resolved = await client.resolveForkAutomatically(forks.branches);
    console.log(`Fork resolved: ${resolved}`);
}
```

### ScratchPad - Temporary Workspace

Efficient unstructured data storage with real-time updates:

```{.typescript .light}
import { ScratchPad, ContentType, ScratchPadConfig } from 'autonomi';

// Create a scratchpad with custom configuration
const config: ScratchPadConfig = {
    compression: true,
    encryption: true,
    realtime: true
};
const pad = await client.createScratchpadWithConfig(
    ContentType.UserSettings,
    config
);

// Update with JSON data
const settings = { theme: 'dark', fontSize: 14 };
await client.updateScratchpadJson(pad.address, settings);

// Subscribe to updates
client.subscribeToScratchpad(pad.address, (update) => {
    console.log(`New data: ${update.data}`);
});
```

## File System Operations

Modern file and directory operations with streaming support:

```{.typescript .light}
import { File, Directory, FileOptions } from 'autonomi/fs';

// Store a file with custom options
const options: FileOptions = {
    compression: true,
    encryption: true,
    redundancy: 3,
    chunkSize: '1MB'
};
const file = await client.storeFileWithOptions(
    'example.txt',
    content,
    options
);

// Stream large files
const writeStream = await client.createFileWriteStream('large-file.dat');
sourceStream.pipe(writeStream);

// Create a directory with metadata
const dir = await client.createDirectoryWithMetadata(
    'docs',
    metadata
);

// Subscribe to directory changes
client.subscribeToDirectory(dir.address, (update) => {
    console.log(`Directory updated: ${update.type}`);
});
```

## Error Handling

Comprehensive error handling with TypeScript support:

```{.typescript .light}
import {
    ChunkError,
    PointerError,
    ListError,
    ScratchPadError
} from 'autonomi/errors';

// Handle chunk operations with detailed errors
try {
    const data = await client.getChunk(address);
    processData(data);
} catch (error) {
    if (error instanceof ChunkError.NotFound) {
        console.log(`Chunk not found: ${error.address}`);
        handleMissing();
    } else if (error instanceof ChunkError.NetworkError) {
        console.log(`Network error: ${error.message}`);
        handleNetworkError(error);
    } else if (error instanceof ChunkError.ValidationError) {
        console.log(`Validation failed: expected ${error.expected}, got ${error.actual}`);
        handleValidationError();
    } else {
        handleOtherError(error);
    }
}

// Handle pointer updates with version conflicts
try {
    await client.updatePointer(address, newTarget);
    console.log('Update successful');
} catch (error) {
    if (error instanceof PointerError.VersionConflict) {
        console.log(`Version conflict: current ${error.current}, attempted ${error.attempted}`);
        handleConflict();
    } else {
        handleOtherError(error);
    }
}
```

## Advanced Usage

### Web Integration

```{.typescript .light}
import { WebClient, WebClientConfig } from 'autonomi/web';

// Create a web-optimized client
const config: WebClientConfig = {
    webSocket: true,
    compression: true,
    cacheSize: '100MB'
};
const client = await WebClient.init(config);

// Subscribe to real-time updates
client.subscribe('updates', (update) => {
    updateUI(update);
});

// Handle offline mode
client.onOffline(() => {
    enableOfflineMode();
});

// Sync when back online
client.onOnline(async () => {
    await client.sync();
});
```

### Custom Types with TypeScript

You can define custom types using TypeScript interfaces:

```{.typescript .light}
interface UserProfile {
  name: string;
  age: number;
  preferences: {
    theme: 'light' | 'dark';
    notifications: boolean;
  };
}

// Use the type with Autonomi
const profile: UserProfile = {
  name: "Alice",
  age: 30,
  preferences: {
    theme: "light",
    notifications: true
  }
};

await client.store(profile);
```

### Quantum-Secure Encryption

```{.typescript .light}
import {
    encryptQuantumSecure,
    decryptQuantumSecure,
    generateKey
} from 'autonomi/crypto';

// Generate quantum-secure keys
const key = await generateQuantumSecureKey();

// Encrypt data with quantum security
const encrypted = await encryptQuantumSecure(data, key);
const pad = await client.createScratchpad(ContentType.Encrypted);
await client.updateScratchpad(pad.address, encrypted);

// Decrypt with quantum security
const encrypted = await client.getScratchpad(pad.address);
const decrypted = await decryptQuantumSecure(encrypted, key);
```

## Performance Optimization

### Connection Pooling

```{.typescript .light}
import { Pool, PoolConfig } from 'autonomi/pool';

// Create a connection pool
const pool = new Pool({
    minConnections: 5,
    maxConnections: 20,
    idleTimeout: 30000
});

// Get a client from the pool
const client = await pool.get();
try {
    await processData(client);
} finally {
    await pool.release(client);
}
```

### Batch Operations

```{.typescript .light}
// Batch chunk storage
const chunks = await client.storeChunksBatch(dataList);

// Batch pointer updates
const updates = [
    new PointerUpdate(addr1, target1),
    new PointerUpdate(addr2, target2)
];
await client.updatePointersBatch(updates);
```

## Best Practices

1. **Web Integration**
   - Use WebSocket for real-time updates
   - Implement offline support
   - Handle connection state
   - Cache frequently accessed data

2. **Error Handling**
   - Use TypeScript for type safety
   - Implement retry logic
   - Handle version conflicts
   - Validate data integrity

3. **Security**
   - Enable quantum security
   - Use encryption for sensitive data
   - Implement access control
   - Validate all inputs

4. **Resource Management**
   - Use connection pools
   - Clean up resources
   - Monitor memory usage
   - Handle backpressure

## TypeScript Types

The API is fully typed for better IDE support and code quality:

```{.typescript .light}
import { Address, Data, Metadata } from 'autonomi/types';

interface Client {
    storeChunk(data: Buffer): Promise<Address>;
    getChunk(address: Address): Promise<Buffer>;
    createPointer(target: Address): Promise<Pointer>;
    updatePointer(address: Address, target: Address): Promise<void>;
}
```

## Further Reading

- [Web Development Guide](../../guides/web_development.md)
- [Quantum Security Guide](../../guides/quantum_security.md)
- [Error Handling Guide](../../guides/error_handling.md)
- [API Reference](https://autonomi.dev/api)
- [Examples Repository](https://github.com/autonomi/examples)
