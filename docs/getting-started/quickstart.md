# Quick Start Guide

This guide will help you get started with Autonomi quickly. We'll create a simple application that stores and retrieves data using linked lists.

## Choose Your Language

=== "Node.js"
    ```typescript
    import { Client, LinkedList } from '@autonomi/client';

    async function main() {
        // Initialize client
        const client = new Client();
        await client.connect();
        
        // Create a linked list
        const list = new LinkedList();
        list.append("Hello");
        list.append("World");
        
        // Store the list
        const address = await client.linkedListPut(list);
        console.log(`List stored at: ${address}`);
        
        // Retrieve the list
        const retrieved = await client.linkedListGet(address);
        console.log(retrieved.toString()); // "Hello World"
    }
    
    main().catch(console.error);
    ```

=== "Python"
    ```python
    import asyncio
    from autonomi import Client, LinkedList

    async def main():
        # Initialize client
        client = Client()
        await client.connect()
        
        # Create a linked list
        list_obj = LinkedList()
        list_obj.append("Hello")
        list_obj.append("World")
        
        # Store the list
        address = await client.linked_list_put(list_obj)
        print(f"List stored at: {address}")
        
        # Retrieve the list
        retrieved = await client.linked_list_get(address)
        print(str(retrieved))  # "Hello World"
    
    asyncio.run(main())
    ```

=== "Rust"
    ```rust
    use autonomi::{Client, LinkedList, Result};

    fn main() -> Result<()> {
        // Initialize client
        let client = Client::new()?;
        
        // Create a linked list
        let mut list = LinkedList::new();
        list.append("Hello");
        list.append("World");
        
        // Store the list
        let address = client.linked_list_put(&list)?;
        println!("List stored at: {}", address);
        
        // Retrieve the list
        let retrieved = client.linked_list_get(&address)?;
        println!("{}", retrieved);
        
        Ok(())
    }
    ```

## Working with Pointers

Pointers allow you to create references to data in the network:

=== "Node.js"
    ```typescript
    import { Client, Pointer } from '@autonomi/client';

    async function main() {
        const client = new Client();
        await client.connect();
        
        // Create a pointer
        const pointer = new Pointer();
        pointer.setTarget("example-target");
        
        // Store the pointer
        const address = await client.pointerPut(pointer);
        console.log(`Pointer stored at: ${address}`);
        
        // Retrieve the pointer
        const retrieved = await client.pointerGet(address);
        console.log(`Target: ${retrieved.getTarget()}`);
    }
    ```

=== "Python"
    ```python
    import asyncio
    from autonomi import Client, Pointer

    async def main():
        client = Client()
        await client.connect()
        
        # Create a pointer
        pointer = Pointer()
        pointer.set_target("example-target")
        
        # Store the pointer
        address = await client.pointer_put(pointer)
        print(f"Pointer stored at: {address}")
        
        # Retrieve the pointer
        retrieved = await client.pointer_get(address)
        print(f"Target: {retrieved.get_target()}")
    
    asyncio.run(main())
    ```

=== "Rust"
    ```rust
    use autonomi::{Client, Pointer, Result};

    fn main() -> Result<()> {
        let client = Client::new()?;
        
        // Create a pointer
        let mut pointer = Pointer::new();
        pointer.set_target("example-target");
        
        // Store the pointer
        let address = client.pointer_put(&pointer)?;
        println!("Pointer stored at: {}", address);
        
        // Retrieve the pointer
        let retrieved = client.pointer_get(&address)?;
        println!("Target: {}", retrieved.target());
        
        Ok(())
    }
    ```

## Next Steps

- [Installation Guide](installation.md)
- [API Reference](../api/autonomi-client/README.md)
- [Local Network Setup](../guides/local_network.md)
