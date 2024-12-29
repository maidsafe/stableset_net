# Node.js API Reference

## Installation

```bash
npm install @autonomi/client
```

## Core Classes

### Client

The main interface for interacting with the Autonomi network.

```typescript
class Client {
  constructor(config?: ClientConfig);
  
  // Linked List Operations
  async linkedListPut(list: LinkedList): Promise<LinkedListAddress>;
  async linkedListGet(address: LinkedListAddress): Promise<LinkedList>;
  
  // Pointer Operations
  async pointerPut(pointer: Pointer): Promise<PointerAddress>;
  async pointerGet(address: PointerAddress): Promise<Pointer>;
  
  // Network Operations
  async connect(): Promise<void>;
  async disconnect(): Promise<void>;
}
```

### LinkedList

Represents a linked list data structure.

```typescript
class LinkedList {
  constructor();
  
  append(data: any): void;
  prepend(data: any): void;
  remove(index: number): void;
  get(index: number): any;
  toString(): string;
}
```

### Pointer

Represents a pointer in the network.

```typescript
class Pointer {
  constructor();
  
  setTarget(target: string): void;
  getTarget(): string;
  isValid(): boolean;
}
```

### Scratchpad

Represents a mutable storage location with versioning and encryption.

```typescript
interface ScratchpadConfig {
  contentType: number;
  data: Uint8Array;
  secretKey: Uint8Array;
}

class Scratchpad {
  constructor(config: ScratchpadConfig);
  
  // Get the network address
  getAddress(): string;
  
  // Get the current version counter
  getCounter(): number;
  
  // Update the data and sign with secret key
  update(data: Uint8Array, secretKey: Uint8Array): void;
  
  // Verify the signature
  verify(): boolean;
  
  // Decrypt the data using the secret key
  decrypt(secretKey: Uint8Array): Uint8Array;
}
```

### Self-Encryption

Utilities for data encryption and chunking.

```typescript
interface EncryptionResult {
  dataMap: DataMap;
  chunks: Chunk[];
}

interface DataMap {
  chunks: ChunkInfo[];
  totalSize: number;
}

interface ChunkInfo {
  hash: string;
  size: number;
  offset: number;
}

class SelfEncryption {
  static async encrypt(data: Uint8Array): Promise<EncryptionResult>;
  static async decrypt(dataMap: DataMap, chunks: Chunk[]): Promise<Uint8Array>;
  static async packDataMap(dataMap: DataMap): Promise<Chunk>;
}

### Files and Directories

Utilities for managing files and directories in the network.

```typescript
interface FileMetadata {
  name: string;
  size: number;
  created: Date;
  modified: Date;
  contentType: string;
}

interface DirectoryEntry {
  name: string;
  type: 'file' | 'directory';
  metadata?: FileMetadata;
}

class File {
  constructor(name: string, data?: Uint8Array);
  
  // Get file metadata
  getMetadata(): FileMetadata;
  
  // Read file contents
  async read(): Promise<Uint8Array>;
  
  // Write file contents
  async write(data: Uint8Array): Promise<void>;
  
  // Update file metadata
  async updateMetadata(metadata: Partial<FileMetadata>): Promise<void>;
}

class Directory {
  constructor(name: string);
  
  // List directory contents
  async list(): Promise<DirectoryEntry[]>;
  
  // Create a new file
  async createFile(name: string, data?: Uint8Array): Promise<File>;
  
  // Create a new subdirectory
  async createDirectory(name: string): Promise<Directory>;
  
  // Get a file or directory by name
  async get(name: string): Promise<File | Directory>;
  
  // Delete a file or directory
  async delete(name: string): Promise<void>;
}
```

### Archive

Utilities for creating and managing archives.

```typescript
interface ArchiveOptions {
  compression?: 'none' | 'gzip' | 'bzip2';
  encryption?: {
    algorithm: 'aes-256-gcm';
    key: Uint8Array;
  };
}

interface ArchiveEntry {
  name: string;
  size: number;
  compressed: boolean;
  encrypted: boolean;
}

class Archive {
  constructor(options?: ArchiveOptions);
  
  // Add a file or directory to the archive
  async add(path: string, source: File | Directory): Promise<void>;
  
  // Extract files from the archive
  async extract(destination: Directory, pattern?: string): Promise<void>;
  
  // List archive contents
  async list(): Promise<ArchiveEntry[]>;
  
  // Verify archive integrity
  async verify(): Promise<boolean>;
}

### Vault

Secure storage for sensitive data.

```typescript
interface VaultConfig {
  secretKey: Uint8Array;
  algorithm?: 'aes-256-gcm' | 'xchacha20-poly1305';
  iterations?: number;
}

interface VaultEntry {
  key: string;
  created: Date;
  modified: Date;
  tags?: string[];
}

class Vault {
  constructor(config: VaultConfig);
  
  // Store encrypted data
  async put(key: string, data: Uint8Array, tags?: string[]): Promise<void>;
  
  // Retrieve and decrypt data
  async get(key: string): Promise<Uint8Array>;
  
  // List vault contents
  async list(tag?: string): Promise<VaultEntry[]>;
  
  // Delete data
  async delete(key: string): Promise<void>;
  
  // Rotate encryption key
  async rotateKey(newKey: Uint8Array): Promise<void>;
}

## Types

```typescript
interface ClientConfig {
  networkUrl?: string;
  timeout?: number;
  retries?: number;
}

type LinkedListAddress = string;
type PointerAddress = string;
```

## Error Handling

```typescript
class AutonomiError extends Error {
  constructor(message: string, code: string);
  
  readonly code: string;
  readonly message: string;
}
```

## Examples

### Basic Usage

```typescript
import { Client, LinkedList } from '@autonomi/client';

async function example() {
  const client = new Client();
  
  // Create and store a linked list
  const list = new LinkedList();
  list.append("Hello");
  list.append("World");
  
  const address = await client.linkedListPut(list);
  console.log(`List stored at: ${address}`);
  
  // Retrieve the list
  const retrieved = await client.linkedListGet(address);
  console.log(retrieved.toString()); // "Hello World"
}
```

### Error Handling

```typescript
try {
  const client = new Client();
  await client.connect();
} catch (error) {
  if (error instanceof AutonomiError) {
    console.error(`Error code: ${error.code}`);
    console.error(`Message: ${error.message}`);
  }
}
```

## Best Practices

1. Always use TypeScript for better type safety
2. Handle errors appropriately
3. Use async/await for all asynchronous operations
4. Properly dispose of resources
5. Follow the provided examples for memory management
