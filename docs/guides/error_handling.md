# Error Handling in Autonomi

This guide covers error handling best practices and patterns across all supported languages in Autonomi.

## Overview

Proper error handling is crucial for building reliable applications with Autonomi. This guide provides comprehensive coverage of error types, handling strategies, and best practices across Rust, Python, and TypeScript implementations.

## Error Types

### Common Error Categories

1. Network Errors
   - Connection failures
   - Timeout errors
   - Peer discovery issues

2. Storage Errors
   - Capacity limits
   - Chunk storage failures
   - Data retrieval errors

3. Cryptographic Errors
   - Encryption failures
   - Signature verification errors
   - Key management issues

4. Client Errors
   - Configuration errors
   - Permission denied
   - Invalid operations

## Language-Specific Implementation

### Rust Implementation

```rust
use autonomi::{Client, Error, Result};

async fn handle_storage_operation(data: Vec<u8>) -> Result<Pointer> {
    let client = Client::init().await?;
    
    match client.store(data).await {
        Ok(pointer) => Ok(pointer),
        Err(Error::StorageCapacityExceeded) => {
            // Handle capacity error
            cleanup_old_data().await?;
            client.store(data).await
        }
        Err(Error::NetworkTimeout) => {
            // Retry with exponential backoff
            retry_with_backoff(|| client.store(data)).await
        }
        Err(e) => Err(e),
    }
}

// Custom error type
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Autonomi error: {0}")]
    Autonomi(#[from] autonomi::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

### Python Implementation

```python
from autonomi import Client, AutonomiError
from autonomi.errors import StorageError, NetworkError
import asyncio
from typing import Any, Optional

async def handle_storage_operation(data: bytes) -> Optional[str]:
    try:
        client = await Client.init()
        pointer = await client.store(data)
        return pointer
    except StorageError as e:
        # Handle storage-specific errors
        logger.error(f"Storage error: {e}")
        if isinstance(e, StorageError.CapacityExceeded):
            await cleanup_old_data()
            return await client.store(data)
        raise
    except NetworkError as e:
        # Handle network-specific errors
        logger.error(f"Network error: {e}")
        return await retry_with_backoff(lambda: client.store(data))
    except AutonomiError as e:
        # Handle other Autonomi-specific errors
        logger.error(f"Autonomi error: {e}")
        raise
```

### TypeScript Implementation

```typescript
import { Client, AutonomiError, StorageError, NetworkError } from '@autonomi/client';

async function handleStorageOperation(data: Buffer): Promise<string> {
  try {
    const client = await Client.init();
    return await client.store(data);
  } catch (error) {
    if (error instanceof StorageError) {
      // Handle storage-specific errors
      console.error('Storage error:', error);
      if (error.code === 'CAPACITY_EXCEEDED') {
        await cleanupOldData();
        return await client.store(data);
      }
    } else if (error instanceof NetworkError) {
      // Handle network-specific errors
      console.error('Network error:', error);
      return await retryWithBackoff(() => client.store(data));
    } else if (error instanceof AutonomiError) {
      // Handle other Autonomi-specific errors
      console.error('Autonomi error:', error);
    }
    throw error;
  }
}
```

## Error Handling Patterns

### Retry Mechanisms

```typescript
async function retryWithBackoff<T>(
  operation: () => Promise<T>,
  maxRetries: number = 3,
  baseDelay: number = 1000
): Promise<T> {
  let lastError: Error;
  
  for (let i = 0; i < maxRetries; i++) {
    try {
      return await operation();
    } catch (error) {
      lastError = error;
      if (!isRetryableError(error)) {
        throw error;
      }
      const delay = baseDelay * Math.pow(2, i);
      await new Promise(resolve => setTimeout(resolve, delay));
    }
  }
  
  throw lastError;
}
```

### Circuit Breaker

```rust
use std::time::{Duration, Instant};

struct CircuitBreaker {
    failure_threshold: u32,
    reset_timeout: Duration,
    failure_count: u32,
    last_failure: Option<Instant>,
}

impl CircuitBreaker {
    async fn execute<F, T, E>(&mut self, operation: F) -> Result<T, E>
    where
        F: Future<Output = Result<T, E>>,
    {
        if self.is_open() {
            return Err(Error::CircuitBreakerOpen);
        }
        
        match operation.await {
            Ok(result) => {
                self.reset();
                Ok(result)
            }
            Err(e) => {
                self.record_failure();
                Err(e)
            }
        }
    }
}
```

### Error Recovery

```python
async def recover_from_error(error: AutonomiError) -> None:
    if isinstance(error, NetworkError):
        await reconnect_to_network()
    elif isinstance(error, StorageError):
        await cleanup_storage()
    elif isinstance(error, CryptoError):
        await refresh_keys()
```

## Logging and Monitoring

### Structured Logging

```rust
use tracing::{error, info, warn};

async fn process_operation() -> Result<()> {
    info!(
        operation = "store",
        size = data.len(),
        "Starting storage operation"
    );
    
    match client.store(data).await {
        Ok(pointer) => {
            info!(
                operation = "store",
                pointer = %pointer,
                "Storage operation successful"
            );
            Ok(())
        }
        Err(e) => {
            error!(
                operation = "store",
                error = %e,
                "Storage operation failed"
            );
            Err(e)
        }
    }
}
```

### Metrics Collection

```typescript
import { metrics } from '@autonomi/client';

async function trackOperation<T>(
  name: string,
  operation: () => Promise<T>
): Promise<T> {
  const timer = metrics.startTimer(`operation_${name}`);
  try {
    const result = await operation();
    metrics.incrementCounter(`operation_${name}_success`);
    return result;
  } catch (error) {
    metrics.incrementCounter(`operation_${name}_error`);
    throw error;
  } finally {
    timer.end();
  }
}
```

## Best Practices

1. Always use typed errors
2. Implement proper error recovery
3. Use structured logging
4. Implement retry mechanisms
5. Monitor error rates
6. Provide clear error messages

## Error Prevention

### Input Validation

```rust
use validator::Validate;

#[derive(Debug, Validate)]
struct StorageRequest {
    #[validate(length(min = 1, max = 1000000))]
    data: Vec<u8>,
    #[validate(range(min = 1, max = 100))]
    chunk_size: usize,
}
```

### Defensive Programming

```python
from typing import Optional

def get_storage_config(config: dict) -> StorageConfig:
    return StorageConfig(
        chunk_size=config.get('chunk_size', DEFAULT_CHUNK_SIZE),
        encryption=config.get('encryption', True),
        compression=config.get('compression', False)
    )
```

## Testing Error Handling

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    #[test]
    async fn test_storage_error_handling() {
        let client = MockClient::new()
            .with_error(StorageError::CapacityExceeded);
            
        let result = handle_storage_operation(data).await;
        assert!(matches!(result, Err(Error::StorageCapacityExceeded)));
    }
}
```

### Integration Tests

```typescript
describe('Error Handling', () => {
  it('should handle network errors', async () => {
    const client = new Client();
    mockNetworkFailure();
    
    await expect(client.store(data))
      .rejects
      .toThrow(NetworkError);
  });
});
```

## Resources

- [API Reference](../api/README.md)
- [Web Development Guide](web_development.md)
- [Quantum Security Guide](quantum_security.md)

## Support

For error handling support:

- Technical support: [support@autonomi.com](mailto:support@autonomi.com)
- Documentation: [https://docs.autonomi.com/errors](https://docs.autonomi.com/errors)
- Issue tracker: [https://github.com/dirvine/autonomi/issues](https://github.com/dirvine/autonomi/issues)
