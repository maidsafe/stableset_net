# Antnode Library

The antnode library provides the core node implementation for participating in the Autonomi network. It handles network participation, storage management, reward collection, and data validation.

## Overview

Antnode enables:

- Full network participation
- Secure data storage and retrieval
- Reward collection for network contribution
- Peer discovery and management
- Data validation and verification
- Network health monitoring

## Installation

### Rust

```toml
[dependencies]
antnode = "0.3.2"

# Optional features
antnode = { version = "0.3.2", features = ["metrics", "tracing"] }
```

### Python

```bash
pip install antnode

# With optional features
pip install antnode[metrics,tracing]
```

## Basic Usage

### Starting a Node

```rust
// Rust
use antnode::{Node, NodeConfig};

// Create basic configuration
let config = NodeConfig::builder()
    .rewards_address("0x1234...")
    .evm_network("arbitrum_sepolia")
    .build()?;

// Create and run node
let node = Node::new(config)?;
node.run().await?;
```

```python
# Python
from antnode import AntNode

# Create and run node
node = AntNode()
node.run(
    rewards_address="0x1234...",
    evm_network="arbitrum_sepolia"
)
```

## Advanced Usage

### Custom Configuration

```rust
// Rust
use antnode::{
    NodeConfig, StorageConfig,
    NetworkConfig, RewardsConfig,
};

// Configure node
let config = NodeConfig::builder()
    .rewards_address("0x1234...")
    .evm_network("arbitrum_sepolia")
    .storage(StorageConfig {
        max_capacity: 1024 * 1024 * 1024,  // 1GB
        min_free_space: 1024 * 1024 * 100,  // 100MB
        path: "/data/autonomi".into(),
    })
    .network(NetworkConfig {
        ip: "0.0.0.0".parse()?,
        port: 12000,
        initial_peers: vec![
            "/ip4/142.93.37.4/udp/40184/quic-v1/p2p/12D3KooWPC8q7QGZsmuTtCYxZ2s3FPXPZcS8LVKkayXkVFkqDEQB".parse()?,
        ],
        max_connections: 50,
        bootstrap_interval: Duration::from_secs(300),
    })
    .rewards(RewardsConfig {
        min_payout: 1_000_000,  // 1 USDC
        auto_compound: true,
        claim_interval: Duration::from_days(7),
    })
    .build()?;

// Create node with config
let node = Node::new(config)?;
```

```python
# Python
from antnode import (
    AntNode, StorageConfig,
    NetworkConfig, RewardsConfig
)

# Configure node
node = AntNode()
node.run(
    rewards_address="0x1234...",
    evm_network="arbitrum_sepolia",
    storage=StorageConfig(
        max_capacity=1024 * 1024 * 1024,  # 1GB
        min_free_space=1024 * 1024 * 100,  # 100MB
        path="/data/autonomi"
    ),
    network=NetworkConfig(
        ip="0.0.0.0",
        port=12000,
        initial_peers=[
            "/ip4/142.93.37.4/udp/40184/quic-v1/p2p/12D3KooWPC8q7QGZsmuTtCYxZ2s3FPXPZcS8LVKkayXkVFkqDEQB"
        ],
        max_connections=50,
        bootstrap_interval=300  # seconds
    ),
    rewards=RewardsConfig(
        min_payout=1_000_000,  # 1 USDC
        auto_compound=True,
        claim_interval=7 * 24 * 60 * 60  # 7 days
    )
)
```

### Storage Operations

```rust
// Rust
use antnode::storage::{Record, RecordType};

// Store data
let key = node.store_record(
    data,
    RecordType::Chunk,
    Some(Duration::from_days(30))
)?;

// Retrieve data
let record = node.get_record(&key)?;

// List records
let records = node.list_records()?;
for record in records {
    println!("Key: {}, Size: {}", record.key, record.size);
}

// Get storage stats
let stats = node.storage_stats()?;
println!("Used: {}, Available: {}", stats.used, stats.available);
```

```python
# Python
from antnode.storage import Record, RecordType

# Store data
key = node.store_record(
    data,
    record_type=RecordType.CHUNK,
    ttl=30 * 24 * 60 * 60  # 30 days
)

# Retrieve data
record = node.get_record(key)

# List records
for record in node.list_records():
    print(f"Key: {record.key}, Size: {record.size}")

# Get storage stats
stats = node.storage_stats()
print(f"Used: {stats.used}, Available: {stats.available}")
```

### Network Management

```rust
// Rust
use antnode::network::{PeerInfo, ConnectionStats};

// Get peer information
let peers = node.list_peers()?;
for peer in peers {
    println!("ID: {}, Address: {}", peer.id, peer.address);
}

// Get connection stats
let stats = node.connection_stats()?;
println!(
    "Connected: {}, Incoming: {}, Outgoing: {}",
    stats.connected, stats.incoming, stats.outgoing
);

// Add peer manually
node.add_peer("/ip4/1.2.3.4/udp/12000/quic-v1/p2p/...".parse()?)?;

// Remove peer
node.remove_peer(peer_id)?;
```

```python
# Python
from antnode.network import PeerInfo, ConnectionStats

# Get peer information
for peer in node.list_peers():
    print(f"ID: {peer.id}, Address: {peer.address}")

# Get connection stats
stats = node.connection_stats()
print(
    f"Connected: {stats.connected}, "
    f"Incoming: {stats.incoming}, "
    f"Outgoing: {stats.outgoing}"
)

# Add peer manually
node.add_peer("/ip4/1.2.3.4/udp/12000/quic-v1/p2p/...")

# Remove peer
node.remove_peer(peer_id)
```

### Rewards Management

```rust
// Rust
use antnode::rewards::{RewardStats, ClaimInfo};

// Get reward statistics
let stats = node.reward_stats()?;
println!(
    "Earned: {}, Claimed: {}, Available: {}",
    stats.earned, stats.claimed, stats.available
);

// Claim rewards
let claim = node.claim_rewards()?;
println!("Claimed {} tokens", claim.amount);

// Get claim history
let claims = node.list_claims()?;
for claim in claims {
    println!(
        "Amount: {}, Time: {}, TX: {}",
        claim.amount, claim.timestamp, claim.tx_hash
    );
}
```

```python
# Python
from antnode.rewards import RewardStats, ClaimInfo

# Get reward statistics
stats = node.reward_stats()
print(
    f"Earned: {stats.earned}, "
    f"Claimed: {stats.claimed}, "
    f"Available: {stats.available}"
)

# Claim rewards
claim = node.claim_rewards()
print(f"Claimed {claim.amount} tokens")

# Get claim history
for claim in node.list_claims():
    print(
        f"Amount: {claim.amount}, "
        f"Time: {claim.timestamp}, "
        f"TX: {claim.tx_hash}"
    )
```

## Monitoring and Metrics

### Prometheus Metrics

```rust
// Rust
use antnode::metrics::{MetricsConfig, PrometheusExporter};

// Configure metrics
let config = NodeConfig::builder()
    .metrics(MetricsConfig {
        enabled: true,
        port: 9100,
        path: "/metrics".into(),
    })
    .build()?;

// Access metrics programmatically
let metrics = node.get_metrics()?;
println!("Storage Used: {}", metrics.storage.used);
println!("Peers Connected: {}", metrics.network.peers);
```

```python
# Python
from antnode.metrics import MetricsConfig, PrometheusExporter

# Configure metrics
node.run(
    metrics=MetricsConfig(
        enabled=True,
        port=9100,
        path="/metrics"
    )
)

# Access metrics programmatically
metrics = node.get_metrics()
print(f"Storage Used: {metrics.storage.used}")
print(f"Peers Connected: {metrics.network.peers}")
```

### Logging and Tracing

```rust
// Rust
use antnode::tracing::{TracingConfig, LogLevel};

// Configure logging
let config = NodeConfig::builder()
    .tracing(TracingConfig {
        level: LogLevel::Debug,
        file: Some("/var/log/antnode.log".into()),
        json: true,
    })
    .build()?;

// Access logs programmatically
let logs = node.get_logs()?;
for log in logs {
    println!("{}: {}", log.timestamp, log.message);
}
```

```python
# Python
from antnode.tracing import TracingConfig, LogLevel

# Configure logging
node.run(
    tracing=TracingConfig(
        level=LogLevel.DEBUG,
        file="/var/log/antnode.log",
        json=True
    )
)

# Access logs programmatically
for log in node.get_logs():
    print(f"{log.timestamp}: {log.message}")
```

## Error Handling

```rust
// Rust
use antnode::error::{Error, Result};

match node.run().await {
    Ok(_) => {
        // Node running successfully
    }
    Err(Error::Storage(e)) => {
        // Handle storage errors
    }
    Err(Error::Network(e)) => {
        // Handle network errors
    }
    Err(Error::Rewards(e)) => {
        // Handle rewards errors
    }
    Err(e) => {
        // Handle other errors
    }
}
```

```python
# Python
from antnode.error import (
    Error, StorageError,
    NetworkError, RewardsError
)

try:
    node.run()
except StorageError as e:
    # Handle storage errors
except NetworkError as e:
    # Handle network errors
except RewardsError as e:
    # Handle rewards errors
except Error as e:
    # Handle other errors
```

## Best Practices

1. **Node Configuration**
   - Set appropriate storage limits
   - Configure reliable initial peers
   - Use secure rewards address
   - Enable metrics and logging

2. **Storage Management**
   - Monitor storage usage
   - Clean up expired records
   - Validate data integrity
   - Backup important data

3. **Network Optimization**
   - Maintain good peer connections
   - Monitor network health
   - Handle network errors
   - Use appropriate timeouts

4. **Rewards Management**
   - Set up automatic claiming
   - Monitor reward accumulation
   - Keep private keys secure
   - Track claim history

## API Reference

See the complete [API Reference](https://docs.rs/antnode) for detailed documentation of all types and functions.
