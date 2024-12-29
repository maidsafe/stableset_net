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
