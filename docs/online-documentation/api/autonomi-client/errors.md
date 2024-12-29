# Error Handling Reference

This page documents the error types and handling patterns for the Autonomi Client API.

## Error Types

### ChunkError

Errors related to chunk operations.

=== "Node.js"
    ```typescript
    class ChunkError extends Error {
        static NotFound: typeof ChunkError;
        static InvalidSize: typeof ChunkError;
        static NetworkError: typeof ChunkError;
        static StorageFull: typeof ChunkError;
    }

    try {
        const chunk = await client.getChunk(address);
    } catch (error) {
        if (error instanceof ChunkError.NotFound) {
            // Handle missing chunk
        } else if (error instanceof ChunkError.NetworkError) {
            // Handle network issues
        }
    }
    ```

=== "Python"
    ```python
    class ChunkError(Exception):
        class NotFound(ChunkError): pass
        class InvalidSize(ChunkError): pass
        class NetworkError(ChunkError): pass
        class StorageFull(ChunkError): pass

    try:
        chunk = client.get_chunk(address)
    except ChunkError.NotFound:
        # Handle missing chunk
        pass
    except ChunkError.NetworkError as e:
        # Handle network issues
        pass
    ```

=== "Rust"
    ```rust
    pub enum ChunkError {
        NotFound,
        InvalidSize,
        NetworkError(NetworkError),
        StorageFull,
    }

    match client.get_chunk(address).await {
        Ok(chunk) => { /* Process chunk */ }
        Err(ChunkError::NotFound) => { /* Handle missing chunk */ }
        Err(ChunkError::NetworkError(e)) => { /* Handle network issues */ }
        Err(e) => { /* Handle other errors */ }
    }
    ```

### PointerError

Errors related to pointer operations.

=== "Node.js"
    ```typescript
    class PointerError extends Error {
        static NotFound: typeof PointerError;
        static VersionConflict: typeof PointerError;
        static InvalidTarget: typeof PointerError;
    }

    try {
        await client.updatePointer(address, newTarget);
    } catch (error) {
        if (error instanceof PointerError.VersionConflict) {
            // Handle version conflict
        }
    }
    ```

=== "Python"
    ```python
    class PointerError(Exception):
        class NotFound(PointerError): pass
        class VersionConflict(PointerError): pass
        class InvalidTarget(PointerError): pass

    try:
        client.update_pointer(address, new_target)
    except PointerError.VersionConflict:
        # Handle version conflict
        pass
    ```

=== "Rust"
    ```rust
    pub enum PointerError {
        NotFound,
        VersionConflict,
        InvalidTarget,
    }

    match client.update_pointer(address, new_target).await {
        Ok(_) => { /* Success */ }
        Err(PointerError::VersionConflict) => { /* Handle conflict */ }
        Err(e) => { /* Handle other errors */ }
    }
    ```

### ListError

Errors related to linked list operations.

=== "Node.js"
    ```typescript
    class ListError extends Error {
        static NotFound: typeof ListError;
        static InvalidIndex: typeof ListError;
        static ForkDetected: typeof ListError;
    }

    try {
        const item = await client.getListItem(address, index);
    } catch (error) {
        if (error instanceof ListError.InvalidIndex) {
            // Handle invalid index
        }
    }
    ```

=== "Python"
    ```python
    class ListError(Exception):
        class NotFound(ListError): pass
        class InvalidIndex(ListError): pass
        class ForkDetected(ListError): pass

    try:
        item = client.get_list_item(address, index)
    except ListError.InvalidIndex:
        # Handle invalid index
        pass
    ```

=== "Rust"
    ```rust
    pub enum ListError {
        NotFound,
        InvalidIndex,
        ForkDetected,
    }

    match client.get_list_item(address, index).await {
        Ok(item) => { /* Process item */ }
        Err(ListError::InvalidIndex) => { /* Handle invalid index */ }
        Err(e) => { /* Handle other errors */ }
    }
    ```

## Error Handling Patterns

### Retry Logic

For transient errors like network issues:

=== "Node.js"
    ```typescript
    async function withRetry<T>(
        operation: () => Promise<T>,
        maxRetries = 3,
        delay = 1000
    ): Promise<T> {
        let lastError: Error;
        for (let i = 0; i < maxRetries; i++) {
            try {
                return await operation();
            } catch (error) {
                if (error instanceof ChunkError.NetworkError) {
                    lastError = error;
                    await new Promise(resolve => setTimeout(resolve, delay));
                    continue;
                }
                throw error;
            }
        }
        throw lastError;
    }

    // Usage
    const chunk = await withRetry(() => client.getChunk(address));
    ```

=== "Python"
    ```python
    async def with_retry(operation, max_retries=3, delay=1.0):
        last_error = None
        for i in range(max_retries):
            try:
                return await operation()
            except ChunkError.NetworkError as e:
                last_error = e
                await asyncio.sleep(delay)
                continue
            except Exception as e:
                raise e
        raise last_error

    # Usage
    chunk = await with_retry(lambda: client.get_chunk(address))
    ```

=== "Rust"
    ```rust
    async fn with_retry<F, T, E>(
        operation: F,
        max_retries: u32,
        delay: Duration
    ) -> Result<T, E>
    where
        F: Fn() -> Future<Output = Result<T, E>>,
        E: From<ChunkError>,
    {
        let mut last_error = None;
        for_ in 0..max_retries {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if let Some(ChunkError::NetworkError(_)) = e.downcast_ref() {
                        last_error = Some(e);
                        tokio::time::sleep(delay).await;
                        continue;
                    }
                    return Err(e);
                }
            }
        }
        Err(last_error.unwrap())
    }

    // Usage
    let chunk = with_retry(|| client.get_chunk(address), 3, Duration::from_secs(1)).await?;
    ```

### Error Recovery

For handling version conflicts in pointers:

=== "Node.js"
    ```typescript
    async function updatePointerSafely(
        client: Client,
        address: Address,
        newTarget: Address
    ): Promise<void> {
        while (true) {
            try {
                await client.updatePointer(address, newTarget);
                break;
            } catch (error) {
                if (error instanceof PointerError.VersionConflict) {
                    const current = await client.resolvePointer(address);
                    if (current.equals(newTarget)) break;
                    continue;
                }
                throw error;
            }
        }
    }
    ```

=== "Python"
    ```python
    async def update_pointer_safely(client, address, new_target):
        while True:
            try:
                await client.update_pointer(address, new_target)
                break
            except PointerError.VersionConflict:
                current = await client.resolve_pointer(address)
                if current == new_target:
                    break
                continue
            except Exception as e:
                raise e
    ```

=== "Rust"
    ```rust
    async fn update_pointer_safely(
        client: &Client,
        address: Address,
        new_target: Address
    ) -> Result<()> {
        loop {
            match client.update_pointer(address, new_target).await {
                Ok(_) => break Ok(()),
                Err(PointerError::VersionConflict) => {
                    let current = client.resolve_pointer(address).await?;
                    if current == new_target {
                        break Ok(());
                    }
                    continue;
                }
                Err(e) => break Err(e),
            }
        }
    }
    ```
