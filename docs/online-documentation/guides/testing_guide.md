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
import { Client, GraphEntry } from '@autonomi/client';

describe('GraphEntry Operations', () => {
  let client: Client;

  beforeEach(() => {
    client = new Client();
  });

  test('should store and retrieve graph', async () => {
    const list = new GraphEntry();
    list.append("test data");
    
    const address = await client.GraphEntryPut(list);
    const retrieved = await client.GraphEntryGet(address);
    
    expect(retrieved.toString()).toBe("test data");
  });
});
```

### Python Example

```python
import pytest
from autonomi import Client, GraphEntry

@pytest.mark.asyncio
async def test_graph_entry_operations():
    client = Client()
    
    # Create and store list
    entry_obj = GraphEntry()
    entry_obj.append("test data")
    
    address = await client.graph_entry_put(entry_obj)
    retrieved = await client.graph_entry_get(address)
    
    assert str(retrieved) == "test data"
```

### Rust Example

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_entry_operations() {
        let client = Client::new();
        
        let mut entry = GraphEntry::new();
        entry.append("test data");

        let address = client.graph_entry_put(&entry).unwrap();
        let retrieved = client.graph_entry_get(&address).unwrap();
        
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
