# Testing Guide

This guide covers testing strategies for Autonomi applications across different languages and environments.

## Test Environment Setup

### Node.js

```bash
npm install --save-dev jest @types/jest ts-jest
```

### Python

```bash
pip install pytest pytest-asyncio
```

### Rust

```bash
cargo install cargo-test
```

## Writing Tests

### Node.js Example

```typescript
import { Client, LinkedList } from '@autonomi/client';

describe('LinkedList Operations', () => {
  let client: Client;

  beforeEach(() => {
    client = new Client();
  });

  test('should store and retrieve linked list', async () => {
    const list = new LinkedList();
    list.append("test data");
    
    const address = await client.linkedListPut(list);
    const retrieved = await client.linkedListGet(address);
    
    expect(retrieved.toString()).toBe("test data");
  });
});
```

### Python Example

```python
import pytest
from autonomi import Client, LinkedList

@pytest.mark.asyncio
async def test_linked_list_operations():
    client = Client()
    
    # Create and store list
    list_obj = LinkedList()
    list_obj.append("test data")
    
    address = await client.linked_list_put(list_obj)
    retrieved = await client.linked_list_get(address)
    
    assert str(retrieved) == "test data"
```

### Rust Example

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linked_list_operations() {
        let client = Client::new();
        
        let mut list = LinkedList::new();
        list.append("test data");
        
        let address = client.linked_list_put(&list).unwrap();
        let retrieved = client.linked_list_get(&address).unwrap();
        
        assert_eq!(retrieved.to_string(), "test data");
    }
}
```

## Test Categories

1. Unit Tests
2. Integration Tests
3. Network Tests
4. EVM Integration Tests

## CI/CD Integration

- GitHub Actions configuration
- Test automation
- Coverage reporting

## Best Practices

- Test isolation
- Mock network calls
- Error scenarios
- Performance testing
