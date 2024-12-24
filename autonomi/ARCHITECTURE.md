# Autonomi Client Architecture Analysis

## Current Architecture

### Overview

The Autonomi client is a Rust-based network client with support for WASM and Python bindings. It provides functionality for interacting with a decentralized network, including data operations, payments, and network connectivity.

### Core Components

1. **Client Module** (`src/client/mod.rs`)
   - Main client implementation
   - Network connectivity and bootstrapping
   - Event handling system
   - Features:
     - Bootstrap cache support
     - Local/remote network support
     - EVM network integration
     - Client event system

2. **Feature Modules**
   - `address`: Network addressing
   - `payment`: Payment functionality
   - `quote`: Quoting system
   - `data`: Data operations
   - `files`: File handling
   - `linked_list`: Data structure implementation
   - `pointer`: Pointer system
   - Optional features:
     - `external-signer`
     - `registers`
     - `vault`

3. **Cross-Platform Support**
   - WASM support via `wasm` module
   - Python bindings via `python.rs`
   - Platform-specific optimizations

### Current Client Implementation Analysis

#### Strengths

1. Modular design with clear separation of concerns
2. Flexible feature system
3. Cross-platform support
4. Built-in bootstrap cache functionality
5. Event-driven architecture

#### Limitations

1. Tight coupling between wallet and client functionality
2. No clear separation between read-only and write operations
3. Complex initialization process
4. Bootstrap process could be more robust

## Proposed Architecture

### Core Design Principles

1. **Data-Centric API Design**
   - Focus on data types and operations
   - Abstract away networking complexity
   - Python-friendly class-based design
   - Efficient streaming operations for large files

2. **Type System**

   ```rust
   // Core data types
   pub struct DataAddress(XorName);
   pub struct ChunkAddress(XorName);
   
   // Data map wrapper for simplified interface
   pub struct FileMap {
       inner: DataMap,
       original_path: PathBuf,
       size: u64,
   }
   ```

3. **Base Client Implementation**

   ```rust
   pub struct Client {
       network: Arc<NetworkLayer>,
       config: ClientConfig,
       wallet: Option<Wallet>,
   }

   impl Client {
       // Constructor for read-only client
       pub async fn new(config: ClientConfig) -> Result<Self, ClientError> {
           Ok(Self {
               network: Arc::new(NetworkLayer::new(config.clone()).await?),
               config,
               wallet: None,
           })
       }

       // Constructor with wallet
       pub async fn with_wallet(
           config: ClientConfig,
           wallet: Wallet
       ) -> Result<Self, ClientError> {
           Ok(Self {
               network: Arc::new(NetworkLayer::new(config.clone()).await?),
               config,
               wallet: Some(wallet),
           })
       }

       // Read operations - available to all clients
       pub async fn get_bytes(&self, address: DataAddress) -> Result<Vec<u8>, ClientError> {
           self.network.get_bytes(address).await
       }

       pub async fn get_file(
           &self,
           map: FileMap,
           output: PathBuf
       ) -> Result<(), ClientError> {
           let get = |name| self.network.get_chunk(name);
           streaming_decrypt_from_storage(&map.inner, &output, get)?;
           Ok(())
       }

       // Write operations - require wallet
       pub async fn store_bytes(&self, data: Vec<u8>) -> Result<DataAddress, ClientError> {
           let wallet = self.wallet.as_ref()
               .ok_or(ClientError::WalletRequired)?;
           
           // Handle payment
           let cost = self.estimate_store_cost(data.len()).await?;
           wallet.pay(cost).await?;
           
           // Store data
           self.network.store_bytes(data).await
       }

       pub async fn store_file(&self, path: PathBuf) -> Result<FileMap, ClientError> {
           let wallet = self.wallet.as_ref()
               .ok_or(ClientError::WalletRequired)?;
           
           // Handle payment
           let size = path.metadata()?.len();
           let cost = self.estimate_store_cost(size).await?;
           wallet.pay(cost).await?;
           
           // Store file
           let store = |name, data| self.network.store_chunk(name, data);
           let data_map = streaming_encrypt_from_file(&path, store)?;
           
           Ok(FileMap {
               inner: data_map,
               original_path: path.clone(),
               size,
           })
       }
   }
   ```

4. **Network Layer**

   ```rust
   struct NetworkLayer {
       bootstrap_cache: BootstrapCache,
       connection_manager: ConnectionManager,
   }

   impl NetworkLayer {
       async fn store_chunk(&self, name: XorName, data: Bytes) -> Result<(), StoreError> {
           // Internal implementation
       }
       
       async fn get_chunk(&self, name: XorName) -> Result<Bytes, FetchError> {
           // Internal implementation
       }
   }
   ```

### Wallet Integration

1. **Wallet Types**

   ```rust
   pub struct Wallet {
       keypair: Keypair,
       network: Arc<NetworkLayer>,
       balance: Arc<RwLock<Amount>>,
   }

   // Different ways to create a wallet
   impl Wallet {
       // Create new wallet with generated keypair
       pub async fn new() -> Result<Self, WalletError> {
           let keypair = Keypair::generate_ed25519();
           Self::from_keypair(keypair).await
       }

       // Create from existing secret key
       pub async fn from_secret_key(secret: &[u8]) -> Result<Self, WalletError> {
           let keypair = Keypair::from_secret_bytes(secret)?;
           Self::from_keypair(keypair).await
       }

       // Create from mnemonic phrase
       pub async fn from_mnemonic(phrase: &str) -> Result<Self, WalletError> {
           let keypair = generate_keypair_from_mnemonic(phrase)?;
           Self::from_keypair(keypair).await
       }

       // Get testnet tokens for development
       pub async fn get_test_tokens(&mut self) -> Result<Amount, WalletError> {
           if !self.network.is_testnet() {
               return Err(WalletError::TestnetOnly);
           }
           self.network.request_test_tokens(self.address()).await
       }
   }
   ```

2. **Automatic Wallet Creation**

   ```rust
   impl Client {
       // Create client with new wallet
       pub async fn with_new_wallet(
           config: ClientConfig,
       ) -> Result<(Self, String), ClientError> {
           let wallet = Wallet::new().await?;
           
           // Save mnemonic for user
           let mnemonic = wallet.keypair.to_mnemonic()?;
           
           // If testnet, get initial tokens
           if config.network_type == NetworkType::TestNet {
               wallet.get_test_tokens().await?;
           }
           
           Ok((
               Self::with_wallet(config, wallet).await?,
               mnemonic
           ))
       }

       // Create client with wallet, getting test tokens if needed
       pub async fn ensure_funded_wallet(
           config: ClientConfig,
           wallet: Option<Wallet>
       ) -> Result<Self, ClientError> {
           let wallet = match wallet {
               Some(w) => w,
               None => {
                   let mut w = Wallet::new().await?;
                   if config.network_type == NetworkType::TestNet {
                       w.get_test_tokens().await?;
                   }
                   w
               }
           };
           
           Self::with_wallet(config, wallet).await
       }
   }
   ```

3. **Python Wallet Integration**

   ```python
   class Wallet:
       @classmethod
       def new(cls) -> 'Wallet':
           """Create a new wallet with generated keypair"""
           return cls._create_new()
       
       @classmethod
       def from_secret_key(cls, secret: bytes) -> 'Wallet':
           """Create wallet from existing secret key"""
           return cls._from_secret(secret)
       
       @classmethod
       def from_mnemonic(cls, phrase: str) -> 'Wallet':
           """Create wallet from mnemonic phrase"""
           return cls._from_phrase(phrase)
       
       async def get_test_tokens(self) -> int:
           """Get testnet tokens (testnet only)"""
           return await self._request_tokens()

   class Client:
       @classmethod
       async def with_new_wallet(cls, config: Optional[Dict] = None) -> Tuple['Client', str]:
           """Create client with new wallet, returns (client, mnemonic)"""
           wallet = await Wallet.new()
           if config and config.get('network_type') == 'testnet':
               await wallet.get_test_tokens()
           return cls(wallet=wallet), wallet.mnemonic

       @classmethod
       async def ensure_funded_wallet(
           cls,
           wallet: Optional[Wallet] = None,
           config: Optional[Dict] = None
       ) -> 'Client':
           """Create client with wallet, creating new one if needed"""
           if not wallet:
               wallet = await Wallet.new()
               if config and config.get('network_type') == 'testnet':
                   await wallet.get_test_tokens()
           return cls(wallet=wallet)
   ```

### Wallet Usage Examples

1. **Rust Examples**

   ```rust
   // Create new client with wallet
   let (client, mnemonic) = Client::with_new_wallet(config).await?;
   println!("Save your mnemonic: {}", mnemonic);

   // Create client ensuring funded wallet
   let client = Client::ensure_funded_wallet(config, None).await?;

   // Restore wallet from mnemonic
   let wallet = Wallet::from_mnemonic(saved_mnemonic).await?;
   let client = Client::with_wallet(config, wallet).await?;
   ```

2. **Python Examples**

   ```python
   # Create new client with wallet
   client, mnemonic = await Client.with_new_wallet()
   print(f"Save your mnemonic: {mnemonic}")

   # Create client ensuring funded wallet
   client = await Client.ensure_funded_wallet()

   # Restore wallet from mnemonic
   wallet = await Wallet.from_mnemonic(saved_mnemonic)
   client = Client(wallet=wallet)
   ```

### Wallet Security Considerations

1. **Mnemonic Handling**

   ```rust
   impl Wallet {
       // Secure mnemonic generation
       fn generate_mnemonic() -> Result<String, WalletError> {
           let entropy = generate_secure_entropy()?;
           bip39::Mnemonic::from_entropy(&entropy)
               .map(|m| m.to_string())
               .map_err(WalletError::from)
       }

       // Validate mnemonic
       fn validate_mnemonic(phrase: &str) -> Result<(), WalletError> {
           bip39::Mnemonic::validate(phrase, bip39::Language::English)
               .map_err(WalletError::from)
       }
   }
   ```

2. **Key Storage**

   ```rust
   impl Client {
       // Export encrypted wallet
       pub async fn export_wallet(
           &self,
           password: &str
       ) -> Result<Vec<u8>, WalletError> {
           let wallet = self.wallet.as_ref()
               .ok_or(WalletError::NoWallet)?;
           wallet.export_encrypted(password).await
       }

       // Import encrypted wallet
       pub async fn import_wallet(
           encrypted: &[u8],
           password: &str
       ) -> Result<Self, WalletError> {
           let wallet = Wallet::import_encrypted(encrypted, password).await?;
           Self::with_wallet(ClientConfig::default(), wallet).await
       }
   }
   ```

### Python Bindings

The Rust class-based design maps directly to Python:

```python
class Client:
    """Base client for network operations"""
    
    @classmethod
    def new(cls, config: Optional[Dict] = None) -> 'Client':
        """Create a read-only client"""
        return cls(config=config)
    
    @classmethod
    def with_wallet(cls, wallet: Wallet, config: Optional[Dict] = None) -> 'Client':
        """Create a client with write capabilities"""
        return cls(wallet=wallet, config=config)
    
    def get_bytes(self, address: str) -> bytes:
        """Read data from the network"""
        pass
        
    def get_file(self, file_map: FileMap, output_path: str) -> None:
        """Download a file from the network"""
        pass
    
    def store_bytes(self, data: bytes) -> str:
        """Store data on the network (requires wallet)"""
        if not self.wallet:
            raise ValueError("Wallet required for write operations")
        pass
    
    def store_file(self, path: str) -> FileMap:
        """Store a file on the network (requires wallet)"""
        if not self.wallet:
            raise ValueError("Wallet required for write operations")
        pass
```

### Usage Examples

1. **Rust Usage**

   ```rust
   // Read-only client
   let client = Client::new(ClientConfig::default()).await?;
   let data = client.get_bytes(address).await?;

   // Client with write capability
   let client = Client::with_wallet(config, wallet).await?;
   let address = client.store_bytes(data).await?;
   ```

2. **Python Usage**

   ```python
   # Read-only client
   client = Client.new()
   data = client.get_bytes("safe://example")

   # Client with write capability
   client = Client.with_wallet(wallet)
   address = client.store_bytes(b"Hello World")
   ```

### Implementation Structure

1. **Core Modules**

   ```
   src/
   ├── data/
   │   ├── types.rs     # Core data types
   │   ├── operations.rs # Data operations
   │   └── metadata.rs   # Metadata handling
   ├── client/
   │   ├── read.rs      # ReadOnlyClient implementation
   │   ├── full.rs      # FullClient implementation
   │   └── network.rs   # Network abstraction (internal)
   └── wallet/
       ├── types.rs     # Wallet types
       └── operations.rs # Payment operations
   ```

2. **Python Bindings**

   ```python
   # Example Python API
   class DataClient:
       def get_data(self, address: str) -> bytes: ...
       def list_data(self, prefix: Optional[str] = None) -> List[str]: ...
       
   class FullClient(DataClient):
       def store_data(self, data: bytes) -> str: ...
       def delete_data(self, address: str) -> None: ...
   ```

### Network Abstraction

1. **Internal Network Layer**

   ```rust
   // Hidden from public API
   mod network {
       pub(crate) struct NetworkLayer {
           bootstrap_cache: BootstrapCache,
           connection_manager: ConnectionManager,
       }
       
       impl NetworkLayer {
           pub(crate) async fn execute_operation(
               &self, 
               operation: DataOperation
           ) -> Result<NetworkResponse, NetworkError> {
               // Handle all network complexity internally
           }
       }
   }
   ```

2. **Bootstrap Handling**

   ```rust
   // Public configuration only exposes necessary options
   pub struct ClientConfig {
       network_type: NetworkType,
       custom_peers: Option<Vec<String>>,
       timeout: Duration,
   }
   
   #[derive(Debug, Clone)]
   pub enum NetworkType {
       Local,
       TestNet,
       MainNet,
   }
   ```

### Client Implementation

1. **Read-Only Client**

   ```rust
   pub struct ReadOnlyClient {
       storage: NetworkStorage,
       config: ClientConfig,
   }
   
   impl ReadOnlyClient {
       pub async fn new(config: ClientConfig) -> Result<Self, ClientError> {
           let network = NetworkLayer::new(config.clone()).await?;
           Ok(Self {
               storage: NetworkStorage { network: Arc::new(network) },
               config,
           })
       }
   }
   
   impl DataClient for ReadOnlyClient {
       // Implement through StorageInterface
   }
   ```

2. **Full Client**

   ```rust
   pub struct FullClient {
       inner: ReadOnlyClient,
       wallet: Option<Wallet>,
   }
   
   impl FullClient {
       pub async fn with_wallet(
           config: ClientConfig,
           wallet: Wallet
       ) -> Result<Self, ClientError> {
           // Initialize with wallet
       }
   }
   
   impl WriteableDataClient for FullClient {
       // Implement write operations
   }
   ```

### Error Handling

```rust
#[derive(Debug, Error)]
pub enum ClientError {
    #[error("Data not found: {0}")]
    NotFound(DataAddress),
    #[error("Insufficient funds for operation")]
    InsufficientFunds,
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),
    #[error("Invalid data: {0}")]
    InvalidData(String),
}
```

## Migration Strategy

1. **Phase 1: Core Data Types**
   - Implement new data type system
   - Create DataClient trait
   - Build basic read operations

2. **Phase 2: Network Abstraction**
   - Implement internal network layer
   - Move existing network code behind abstraction
   - Create simplified configuration

3. **Phase 3: Write Operations**
   - Implement WriteableDataClient
   - Integrate wallet functionality
   - Add payment operations

4. **Phase 4: Python Bindings**
   - Create Python-friendly wrappers
   - Implement type conversions
   - Add Python-specific documentation

## Next Steps

1. Create new data type definitions
2. Implement DataClient trait
3. Build network abstraction layer
4. Create initial Python binding prototypes

## Implementation Benefits

1. **Simplified Data Handling**
   - Always uses streaming operations for files
   - Guaranteed 3-chunk data maps
   - No memory-based encryption/decryption for large files
   - No data map squashing required

2. **Efficient Resource Usage**
   - Streaming operations minimize memory usage
   - Direct file-to-network and network-to-file transfers
   - Constant memory overhead regardless of file size

3. **Clear API Boundaries**
   - Separate interfaces for storage and client operations
   - Simple integration with self_encryption library
   - Clean separation between file and byte operations

## API Documentation

### Quick Start

```rust
// Initialize a read-only client
let client = ReadOnlyClient::new(ClientConfig::default()).await?;

// Read data from the network
let data = client.get_bytes(address).await?;

// Initialize a client with wallet for write operations
let wallet = Wallet::from_secret_key(secret_key);
let client = FullClient::with_wallet(ClientConfig::default(), wallet).await?;

// Store data on the network (automatically handles payment)
let address = client.store_bytes(data).await?;
```

### Python Quick Start

```python
from autonomi import ReadOnlyClient, FullClient, Wallet

# Initialize read-only client
client = ReadOnlyClient()

# Read data
data = client.get_bytes("safe://example_address")

# Initialize client with wallet
wallet = Wallet.from_secret_key("your_secret_key")
client = FullClient(wallet=wallet)

# Store data (handles payment automatically)
address = client.store_bytes(b"Hello, World!")
```

### Common Operations

1. **File Operations**

   ```rust
   // Store a file
   let file_map = client.store_file("path/to/file.txt").await?;
   
   // Retrieve a file
   client.get_file(file_map, "path/to/output.txt").await?;
   ```

2. **Byte Operations**

   ```rust
   // Store bytes
   let address = client.store_bytes(data).await?;
   
   // Retrieve bytes
   let data = client.get_bytes(address).await?;
   ```

3. **Wallet Operations**

   ```rust
   // Check balance
   let balance = client.wallet()?.balance().await?;
   
   // Get cost estimate for operation
   let cost = client.estimate_store_cost(data.len()).await?;
   ```

### Python API Examples

1. **File Handling**

   ```python
   # Store a file
   file_map = client.store_file("path/to/file.txt")
   
   # Save file_map for later retrieval
   file_map_json = file_map.to_json()
   
   # Later, retrieve the file
   file_map = FileMap.from_json(file_map_json)
   client.get_file(file_map, "path/to/output.txt")
   ```

2. **Data Operations**

   ```python
   # Store data
   address = client.store_bytes(b"Hello World")
   
   # Retrieve data
   data = client.get_bytes(address)
   ```

3. **Wallet Management**

   ```python
   # Check balance
   balance = client.wallet.balance
   
   # Get operation cost estimate
   cost = client.estimate_store_cost(len(data))
   ```

### Configuration

1. **Network Selection**

   ```rust
   // Connect to mainnet
   let config = ClientConfig {
       network_type: NetworkType::MainNet,
       ..Default::default()
   };
   
   // Connect to local network
   let config = ClientConfig {
       network_type: NetworkType::Local,
       ..Default::default()
   };
   ```

2. **Custom Peers**

   ```rust
   // Connect using specific peers
   let config = ClientConfig {
       custom_peers: Some(vec!["peer1_address".to_string()]),
       ..Default::default()
   };
   ```

### Error Handling

```rust
match client.store_bytes(data).await {
    Ok(address) => println!("Stored at: {}", address),
    Err(ClientError::InsufficientFunds) => println!("Need more funds!"),
    Err(ClientError::Network(e)) => println!("Network error: {}", e),
    Err(e) => println!("Other error: {}", e),
}
```

### Best Practices

1. **Resource Management**
   - Use streaming operations for files over 1MB
   - Close clients when done to free network resources
   - Handle wallet errors appropriately

2. **Error Handling**
   - Always check for InsufficientFunds before write operations
   - Implement proper retry logic for network operations
   - Cache FileMap objects for important data

3. **Performance**
   - Reuse client instances when possible
   - Use byte operations for small data
   - Batch operations when practical

## Local Network Testing

### Local Network Setup

1. **Node Configuration with MDNS**

   ```rust
   pub struct LocalNode {
       process: Child,
       rpc_port: u16,
       peer_id: PeerId,
       multiaddr: Multiaddr,
   }

   impl LocalNode {
       pub async fn start() -> Result<Self, NodeError> {
           // Find available port
           let rpc_port = get_available_port()?;
           
           // Start ant-node with local flag for mdns discovery
           let process = Command::new("ant-node")
               .arg("--local")  // Enable mdns for local discovery
               .arg("--rpc-port")
               .arg(rpc_port.to_string())
               .arg("--log-level")
               .arg("debug")  // Helpful for seeing mdns activity
               .spawn()?;
           
           // Wait for node to start and get peer info
           let peer_info = wait_for_node_ready(rpc_port).await?;
           
           Ok(Self {
               process,
               rpc_port,
               peer_id: peer_info.peer_id,
               multiaddr: peer_info.multiaddr,
           })
       }
   }
   ```

2. **Local Network Manager with MDNS**

   ```rust
   pub struct LocalNetwork {
       nodes: Vec<LocalNode>,
   }

   impl LocalNetwork {
       pub async fn new(node_count: usize) -> Result<Self, NodeError> {
           let mut nodes = Vec::with_capacity(node_count);
           
           // Start nodes - they will discover each other via mdns
           for _ in 0..node_count {
               nodes.push(LocalNode::start().await?);
           }
           
           // Wait for mdns discovery and network stabilization
           tokio::time::sleep(Duration::from_secs(5)).await;
           
           // Verify nodes have discovered each other
           Self::verify_node_connectivity(&nodes).await?;
           
           Ok(Self { nodes })
       }

       async fn verify_node_connectivity(nodes: &[LocalNode]) -> Result<(), NodeError> {
           // Check each node's peer count through RPC
           for node in nodes {
               let peers = node.get_connected_peers().await?;
               if peers.len() < nodes.len() - 1 {
                   return Err(NodeError::InsufficientConnectivity {
                       expected: nodes.len() - 1,
                       actual: peers.len(),
                   });
               }
           }
           Ok(())
       }
   }
   ```

### Client Integration with Local Network

1. **Local Client Setup**

   ```rust
   impl Client {
       // Create client connected to local network using mdns
       pub async fn local_test(node_count: usize) -> Result<(Self, LocalNetwork), ClientError> {
           // Start local network
           let network = LocalNetwork::new(node_count).await?;
           
           // Create client config with local flag for mdns
           let config = ClientConfig {
               network_type: NetworkType::Local,  // Enables mdns in client
               ..Default::default()
           };
           
           // Create client - it will discover nodes via mdns
           let client = Self::new(config).await?;
           
           Ok((client, network))
       }
   }
   ```

### Usage Examples

1. **Local Development Testing**

   ```rust
   #[tokio::test]
   async fn test_local_network() -> Result<(), Box<dyn Error>> {
       // Start client and local network with mdns discovery
       let (mut client, network) = Client::local_test(3).await?;
       
       // Create test wallet for write operations
       let wallet = Wallet::new().await?;
       client.set_wallet(Some(wallet));
       
       // Store and retrieve data using local network
       let test_data = b"Hello, local network!";
       let address = client.store_bytes(test_data.to_vec()).await?;
       let retrieved = client.get_bytes(address).await?;
       assert_eq!(retrieved, test_data);
       
       Ok(())
   }
   ```

2. **Python Local Testing**

   ```python
   async def test_local_network():
       # Start local network with mdns discovery
       client, network = await Client.local_test(node_count=3)
       
       try:
           # Create wallet for testing
           wallet = await Wallet.new()
           client.wallet = wallet
           
           # Test data operations
           address = await client.store_bytes(b"Hello, local network!")
           data = await client.get_bytes(address)
           assert data == b"Hello, local network!"
           
       finally:
           await network.stop()
   ```

### Local Development Configuration

1. **Node Options for Local Testing**

   ```rust
   pub struct LocalNodeConfig {
       rpc_port: Option<u16>,
       data_dir: Option<PathBuf>,
       log_level: LogLevel,
       mdns_enabled: bool,  // Always true for local testing
   }

   impl Default for LocalNodeConfig {
       fn default() -> Self {
           Self {
               rpc_port: None,  // Automatically assign
               data_dir: None,  // Use temporary directory
               log_level: LogLevel::Debug,  // More verbose for local testing
               mdns_enabled: true,
           }
       }
   }
   ```

2. **Client Configuration for Local Testing**

   ```rust
   impl Client {
       pub async fn new_local() -> Result<Self, ClientError> {
           let config = ClientConfig {
               network_type: NetworkType::Local,
               log_level: LogLevel::Debug,
               ..Default::default()
           };
           Self::new(config).await
       }
   }
   ```

### Best Practices for Local Testing

1. **MDNS Usage**
   - Always use `--local` flag for local development
   - Allow sufficient time for MDNS discovery
   - Monitor MDNS logs for connectivity issues
   - Test with different network sizes

2. **Network Verification**
   - Verify node discovery through MDNS
   - Check peer connections before testing
   - Monitor network stability
   - Handle node disconnections gracefully

3. **Development Workflow**

   ```rust
   // Example development workflow
   async fn development_workflow() -> Result<(), Error> {
       // 1. Start local network with mdns
       let (client, network) = Client::local_test(3).await?;
       
       // 2. Verify network health
       network.verify_connectivity().await?;
       
       // 3. Run development tests
       run_tests(client).await?;
       
       // 4. Clean up
       network.stop().await?;
       Ok(())
   }
   ```
