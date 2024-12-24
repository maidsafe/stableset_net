# Autonomi Implementation Schedule (2-Day Sprint)

## Day 1: Core Implementation (Morning)

### Hour 0-1: Project Setup

```bash
# Project structure
cargo new autonomi
cd autonomi
# Add dependencies to Cargo.toml
# Set up basic directory structure
```

### Hour 1-2: Network Layer

```rust
// Implement core networking with mdns
// Focus on local testing first
impl Client {
    pub async fn new_local() -> Result<Self> {
        // Initialize with mdns discovery
        let config = ClientConfig {
            network_type: NetworkType::Local,
            ..Default::default()
        };
        Self::new(config).await
    }
}
```

### Hour 2-4: Core Client & Data Operations

```rust
// Implement basic client with self_encryption
impl Client {
    pub async fn store_bytes(&self, data: Vec<u8>) -> Result<DataAddress>;
    pub async fn get_bytes(&self, address: DataAddress) -> Result<Vec<u8>>;
    pub async fn store_file(&self, path: PathBuf) -> Result<FileMap>;
    pub async fn get_file(&self, map: FileMap, output: PathBuf) -> Result<()>;
}
```

## Day 1: Integration (Afternoon)

### Hour 4-6: Local Network Testing

```rust
// Implement local network management
pub struct LocalNetwork {
    nodes: Vec<LocalNode>,
}

impl LocalNetwork {
    pub async fn new(node_count: usize) -> Result<Self>;
}

// Basic test
#[tokio::test]
async fn test_local_network() {
    let (client, network) = Client::local_test(3).await?;
    // Test basic operations
}
```

### Hour 6-8: Wallet Integration

```rust
// Basic wallet implementation
impl Client {
    pub async fn with_wallet(config: ClientConfig, wallet: Wallet) -> Result<Self>;
    pub async fn ensure_funded_wallet(config: ClientConfig) -> Result<Self>;
}
```

## Day 2: Polish and Python (Morning)

### Hour 0-2: Python Bindings

```python
# Basic Python API
class Client:
    @classmethod
    async def new_local(cls) -> 'Client': ...
    async def store_bytes(self, data: bytes) -> str: ...
    async def get_bytes(self, address: str) -> bytes: ...
```

### Hour 2-4: Testing & Documentation

- Write essential tests
- Document core APIs
- Create basic examples

## Day 2: Finalization (Afternoon)

### Hour 4-6: Integration Testing

- Test complete workflows
- Fix any issues found
- Performance testing

### Hour 6-8: Final Polish

- Documentation cleanup
- Example applications
- Final testing

## Critical Path Features

1. **Must Have**
   - Local network with mdns
   - Basic data operations
   - File streaming
   - Python bindings

2. **Should Have**
   - Wallet integration
   - Basic error handling
   - Simple examples

3. **Nice to Have**
   - Advanced error handling
   - Performance optimizations
   - Extended documentation

## Testing Priorities

1. **Critical Tests**

   ```rust
   #[tokio::test]
   async fn test_local_network_basics() {
       let client = Client::new_local().await?;
       let data = b"test data";
       let addr = client.store_bytes(data.to_vec()).await?;
       let retrieved = client.get_bytes(addr).await?;
       assert_eq!(data, &retrieved[..]);
   }
   ```

2. **Core Functionality**

   ```rust
   #[tokio::test]
   async fn test_file_operations() {
       let client = Client::new_local().await?;
       let file_map = client.store_file("test.txt").await?;
       client.get_file(file_map, "retrieved.txt").await?;
   }
   ```

## Implementation Order

### Day 1 Morning Checklist

- [ ] Project setup
- [ ] Network layer with mdns
- [ ] Basic client operations
- [ ] Self-encryption integration

### Day 1 Afternoon Checklist

- [ ] Local network testing
- [ ] Wallet integration
- [ ] Basic error handling
- [ ] Core tests

### Day 2 Morning Checklist

- [ ] Python bindings
- [ ] Documentation
- [ ] Examples
- [ ] Integration tests

### Day 2 Afternoon Checklist

- [ ] Performance testing
- [ ] Bug fixes
- [ ] Final documentation
- [ ] Release preparation

## Development Guidelines

1. **Fast Development**
   - Use existing code where possible
   - Minimize custom implementations
   - Focus on core functionality first

2. **Testing Strategy**
   - Test as you go
   - Focus on critical paths
   - Integration tests over unit tests

3. **Documentation**
   - Document while coding
   - Focus on API examples
   - Keep README updated

## Emergency Fallbacks

1. **Network Issues**
   - Default to local testing
   - Skip complex network scenarios
   - Focus on basic connectivity

2. **Feature Cuts**
   - Skip advanced error handling
   - Minimal wallet features
   - Basic Python bindings only

3. **Time Management**
   - Core features first
   - Skip non-essential optimizations
   - Minimal but functional documentation
