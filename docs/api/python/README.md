# Python API Reference

## Installation

```bash
pip install autonomi-client
```

## Core Classes

### Client

The main interface for interacting with the Autonomi network.

```python
from typing import Optional, Dict, Any
from autonomi import LinkedList, Pointer, LinkedListAddress, PointerAddress

class Client:
    def __init__(self, config: Optional[Dict[str, Any]] = None) -> None:
        """Initialize a new Autonomi client.
        
        Args:
            config: Optional configuration dictionary
        """
        pass
        
    async def linked_list_put(self, list_obj: LinkedList) -> LinkedListAddress:
        """Store a linked list in the network.
        
        Args:
            list_obj: The linked list to store
            
        Returns:
            The address where the list is stored
        """
        pass
        
    async def linked_list_get(self, address: LinkedListAddress) -> LinkedList:
        """Retrieve a linked list from the network.
        
        Args:
            address: The address of the list to retrieve
            
        Returns:
            The retrieved linked list
        """
        pass
        
    async def pointer_put(self, pointer: Pointer) -> PointerAddress:
        """Store a pointer in the network.
        
        Args:
            pointer: The pointer to store
            
        Returns:
            The address where the pointer is stored
        """
        pass
        
    async def pointer_get(self, address: PointerAddress) -> Pointer:
        """Retrieve a pointer from the network.
        
        Args:
            address: The address of the pointer to retrieve
            
        Returns:
            The retrieved pointer
        """
        pass
```

### LinkedList

Represents a linked list data structure.

```python
from typing import Any

class LinkedList:
    def __init__(self) -> None:
        """Initialize a new linked list."""
        pass
        
    def append(self, data: Any) -> None:
        """Append data to the list.
        
        Args:
            data: The data to append
        """
        pass
        
    def prepend(self, data: Any) -> None:
        """Prepend data to the list.
        
        Args:
            data: The data to prepend
        """
        pass
        
    def remove(self, index: int) -> None:
        """Remove an item at the specified index.
        
        Args:
            index: The index to remove
        """
        pass
        
    def get(self, index: int) -> Any:
        """Get an item at the specified index.
        
        Args:
            index: The index to retrieve
            
        Returns:
            The item at the specified index
        """
        pass
```

### Pointer

Represents a pointer in the network.

```python
class Pointer:
    def __init__(self) -> None:
        """Initialize a new pointer."""
        pass
        
    def set_target(self, target: str) -> None:
        """Set the target of the pointer.
        
        Args:
            target: The target to set
        """
        pass
        
    def get_target(self) -> str:
        """Get the target of the pointer.
        
        Returns:
            The current target
        """
        pass
        
    def is_valid(self) -> bool:
        """Check if the pointer is valid.
        
        Returns:
            True if valid, False otherwise
        """
        pass
```

### Scratchpad

Represents a mutable storage location with versioning and encryption.

```python
from dataclasses import dataclass
from typing import Optional, Union

@dataclass
class ScratchpadConfig:
    content_type: int
    data: bytes
    secret_key: bytes

class Scratchpad:
    def __init__(self, config: ScratchpadConfig) -> None:
        """Initialize a new scratchpad.
        
        Args:
            config: Configuration for the scratchpad
        """
        pass
        
    def get_address(self) -> str:
        """Get the network address of the scratchpad.
        
        Returns:
            The network address
        """
        pass
        
    def get_counter(self) -> int:
        """Get the current version counter.
        
        Returns:
            The current counter value
        """
        pass
        
    def update(self, data: bytes, secret_key: bytes) -> None:
        """Update the data and sign with secret key.
        
        Args:
            data: New data to store
            secret_key: Key for signing
        """
        pass
        
    def verify(self) -> bool:
        """Verify the signature.
        
        Returns:
            True if signature is valid
        """
        pass
        
    def decrypt(self, secret_key: bytes) -> bytes:
        """Decrypt the data using the secret key.
        
        Args:
            secret_key: Key for decryption
            
        Returns:
            The decrypted data
        """
        pass

### Self-Encryption

Utilities for data encryption and chunking.

```python
from typing import List, NamedTuple

class ChunkInfo(NamedTuple):
    hash: str
    size: int
    offset: int

class DataMap(NamedTuple):
    chunks: List[ChunkInfo]
    total_size: int

class EncryptionResult(NamedTuple):
    data_map: DataMap
    chunks: List[bytes]

class SelfEncryption:
    @staticmethod
    async def encrypt(data: bytes) -> EncryptionResult:
        """Encrypt and chunk the data.
        
        Args:
            data: Data to encrypt
            
        Returns:
            Encryption result containing data map and chunks
        """
        pass
        
    @staticmethod
    async def decrypt(data_map: DataMap, chunks: List[bytes]) -> bytes:
        """Decrypt and reassemble the data.
        
        Args:
            data_map: Map of chunks
            chunks: List of encrypted chunks
            
        Returns:
            The decrypted data
        """
        pass
        
    @staticmethod
    async def pack_data_map(data_map: DataMap) -> bytes:
        """Pack a data map into a chunk.
        
        Args:
            data_map: Map to pack
            
        Returns:
            The packed chunk
        """
        pass
```

### Files and Directories

Utilities for managing files and directories in the network.

```python
from dataclasses import dataclass
from datetime import datetime
from typing import Optional, Union, List

@dataclass
class FileMetadata:
    name: str
    size: int
    created: datetime
    modified: datetime
    content_type: str

@dataclass
class DirectoryEntry:
    name: str
    type: str  # 'file' or 'directory'
    metadata: Optional[FileMetadata] = None

class File:
    def __init__(self, name: str, data: Optional[bytes] = None) -> None:
        """Initialize a new file.
        
        Args:
            name: Name of the file
            data: Optional initial data
        """
        pass
        
    def get_metadata(self) -> FileMetadata:
        """Get file metadata.
        
        Returns:
            File metadata
        """
        pass
        
    async def read(self) -> bytes:
        """Read file contents.
        
        Returns:
            File contents as bytes
        """
        pass
        
    async def write(self, data: bytes) -> None:
        """Write file contents.
        
        Args:
            data: Data to write
        """
        pass
        
    async def update_metadata(self, metadata: FileMetadata) -> None:
        """Update file metadata.
        
        Args:
            metadata: New metadata
        """
        pass

class Directory:
    def __init__(self, name: str) -> None:
        """Initialize a new directory.
        
        Args:
            name: Name of the directory
        """
        pass
        
    async def list(self) -> List[DirectoryEntry]:
        """List directory contents.
        
        Returns:
            List of directory entries
        """
        pass
        
    async def create_file(self, name: str, data: Optional[bytes] = None) -> File:
        """Create a new file.
        
        Args:
            name: Name of the file
            data: Optional initial data
            
        Returns:
            The created file
        """
        pass
        
    async def create_directory(self, name: str) -> 'Directory':
        """Create a new subdirectory.
        
        Args:
            name: Name of the directory
            
        Returns:
            The created directory
        """
        pass
        
    async def get(self, name: str) -> Union[File, 'Directory']:
        """Get a file or directory by name.
        
        Args:
            name: Name to look up
            
        Returns:
            File or Directory object
        """
        pass
        
    async def delete(self, name: str) -> None:
        """Delete a file or directory.
        
        Args:
            name: Name to delete
        """
        pass

### Archive

Utilities for creating and managing archives.

```python
from dataclasses import dataclass
from typing import Optional, List

@dataclass
class ArchiveOptions:
    compression: str = 'none'  # 'none', 'gzip', or 'bzip2'
    encryption: Optional[dict] = None  # {'algorithm': 'aes-256-gcm', 'key': bytes}

@dataclass
class ArchiveEntry:
    name: str
    size: int
    compressed: bool
    encrypted: bool

class Archive:
    def __init__(self, options: Optional[ArchiveOptions] = None) -> None:
        """Initialize a new archive.
        
        Args:
            options: Optional archive configuration
        """
        pass
        
    async def add(self, path: str, source: Union[File, Directory]) -> None:
        """Add a file or directory to the archive.
        
        Args:
            path: Path within the archive
            source: Source file or directory
        """
        pass
        
    async def extract(self, destination: Directory, pattern: Optional[str] = None) -> None:
        """Extract files from the archive.
        
        Args:
            destination: Destination directory
            pattern: Optional glob pattern
        """
        pass
        
    async def list(self) -> List[ArchiveEntry]:
        """List archive contents.
        
        Returns:
            List of archive entries
        """
        pass
        
    async def verify(self) -> bool:
        """Verify archive integrity.
        
        Returns:
            True if archive is valid
        """
        pass

### Vault

Secure storage for sensitive data.

```python
from dataclasses import dataclass
from datetime import datetime
from typing import Optional, List

@dataclass
class VaultConfig:
    secret_key: bytes
    algorithm: str = 'aes-256-gcm'  # or 'xchacha20-poly1305'
    iterations: int = 100000

@dataclass
class VaultEntry:
    key: str
    created: datetime
    modified: datetime
    tags: Optional[List[str]] = None

class Vault:
    def __init__(self, config: VaultConfig) -> None:
        """Initialize a new vault.
        
        Args:
            config: Vault configuration
        """
        pass
        
    async def put(self, key: str, data: bytes, tags: Optional[List[str]] = None) -> None:
        """Store encrypted data.
        
        Args:
            key: Key to store under
            data: Data to encrypt and store
            tags: Optional tags
        """
        pass
        
    async def get(self, key: str) -> bytes:
        """Retrieve and decrypt data.
        
        Args:
            key: Key to retrieve
            
        Returns:
            Decrypted data
        """
        pass
        
    async def list(self, tag: Optional[str] = None) -> List[VaultEntry]:
        """List vault contents.
        
        Args:
            tag: Optional tag to filter by
            
        Returns:
            List of vault entries
        """
        pass
        
    async def delete(self, key: str) -> None:
        """Delete data.
        
        Args:
            key: Key to delete
        """
        pass
        
    async def rotate_key(self, new_key: bytes) -> None:
        """Rotate encryption key.
        
        Args:
            new_key: New encryption key
        """
        pass
```

## Examples

### Basic Usage

```python
import asyncio
from autonomi import Client, LinkedList

async def example():
    client = Client()
    
    # Create and store a linked list
    list_obj = LinkedList()
    list_obj.append("Hello")
    list_obj.append("World")
    
    address = await client.linked_list_put(list_obj)
    print(f"List stored at: {address}")
    
    # Retrieve the list
    retrieved = await client.linked_list_get(address)
    print(str(retrieved))  # "Hello World"

# Run the example
asyncio.run(example())
```

### Error Handling

```python
from autonomi import Client, AutonomiError

async def example():
    try:
        client = Client()
        await client.connect()
    except AutonomiError as e:
        print(f"Error code: {e.code}")
        print(f"Message: {e.message}")
```

## Best Practices

1. Use type hints for better code quality
2. Handle errors appropriately using try/except
3. Use async/await for all asynchronous operations
4. Follow the provided examples for proper resource management
5. Use context managers when appropriate
