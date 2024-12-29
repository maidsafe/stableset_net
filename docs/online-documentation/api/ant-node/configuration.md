# Node Configuration

This page documents the configuration options for running an Ant Node.

## Configuration Options

### Network Configuration

=== "Python"
    ```python
    from antnode import NodeConfig

    config = NodeConfig(
        # Network settings
        ip="0.0.0.0",              # IP address to listen on
        port=12000,                # Port to listen on
        evm_network="arbitrum_sepolia",  # EVM network to use
        rewards_address="0x...",   # EVM wallet address for rewards
        
        # Node settings
        local=False,               # Run in local mode
        home_network=False,        # Node is behind NAT
        root_dir=None,             # Custom root directory
        
        # Network peers
        initial_peers=[            # Bootstrap peers
            "/ip4/142.93.37.4/udp/40184/quic-v1/p2p/12D3KooWPC8q7QGZsmuTtCYxZ2s3FPXPZcS8LVKkayXkVFkqDEQB",
        ]
    )
    ```

=== "Rust"
    ```rust
    use ant_node::{NodeConfig, RewardsAddress};
    use std::path::PathBuf;

    let config = NodeConfig::builder()
        // Network settings
        .ip("0.0.0.0")
        .port(12000)
        .evm_network("arbitrum_sepolia")
        .rewards_address(RewardsAddress::new("0x..."))
        
        // Node settings
        .local(false)
        .home_network(false)
        .root_dir(Some(PathBuf::from("/path/to/data")))
        
        // Network peers
        .initial_peers(vec![
            "/ip4/142.93.37.4/udp/40184/quic-v1/p2p/12D3KooWPC8q7QGZsmuTtCYxZ2s3FPXPZcS8LVKkayXkVFkqDEQB"
                .parse()
                .unwrap()
        ])
        .build()?;
    ```

### Storage Configuration

=== "Python"
    ```python
    from antnode import StorageConfig

    storage_config = StorageConfig(
        max_size=1024 * 1024 * 1024,  # 1GB max storage
        min_free_space=1024 * 1024,    # 1MB min free space
        cleanup_interval=3600,          # Cleanup every hour
        backup_enabled=True,
        backup_interval=86400,          # Daily backups
        backup_path="/path/to/backups"
    )

    config.storage = storage_config
    ```

=== "Rust"
    ```rust
    use ant_node::StorageConfig;
    use std::path::PathBuf;

    let storage_config = StorageConfig::builder()
        .max_size(1024 * 1024 * 1024)  // 1GB max storage
        .min_free_space(1024 * 1024)    // 1MB min free space
        .cleanup_interval(3600)          // Cleanup every hour
        .backup_enabled(true)
        .backup_interval(86400)          // Daily backups
        .backup_path(PathBuf::from("/path/to/backups"))
        .build()?;

    config.storage = storage_config;
    ```

### Network Types

The `evm_network` parameter can be one of:

- `arbitrum_sepolia` - Test network
- `arbitrum_one` - Main network

### Directory Structure

The node uses the following directory structure:

```
root_dir/
├── data/           # Stored data chunks
├── logs/           # Node logs
├── peers/          # Peer information
└── metadata/       # Node metadata
```

## Environment Variables

The node configuration can also be set using environment variables:

```bash
# Network settings
export ANT_NODE_IP="0.0.0.0"
export ANT_NODE_PORT="12000"
export ANT_NODE_EVM_NETWORK="arbitrum_sepolia"
export ANT_NODE_REWARDS_ADDRESS="0x..."

# Node settings
export ANT_NODE_LOCAL="false"
export ANT_NODE_HOME_NETWORK="false"
export ANT_NODE_ROOT_DIR="/path/to/data"

# Storage settings
export ANT_NODE_MAX_STORAGE="1073741824"  # 1GB
export ANT_NODE_MIN_FREE_SPACE="1048576"  # 1MB
export ANT_NODE_CLEANUP_INTERVAL="3600"
```

## Configuration File

You can also provide configuration through a YAML file:

```yaml
# config.yaml
network:
  ip: "0.0.0.0"
  port: 12000
  evm_network: "arbitrum_sepolia"
  rewards_address: "0x..."
  initial_peers:
    - "/ip4/142.93.37.4/udp/40184/quic-v1/p2p/12D3KooWPC8q7QGZsmuTtCYxZ2s3FPXPZcS8LVKkayXkVFkqDEQB"

node:
  local: false
  home_network: false
  root_dir: "/path/to/data"

storage:
  max_size: 1073741824  # 1GB
  min_free_space: 1048576  # 1MB
  cleanup_interval: 3600
  backup:
    enabled: true
    interval: 86400
    path: "/path/to/backups"
```

Load the configuration file:

=== "Python"
    ```python
    from antnode import load_config

    config = load_config("config.yaml")
    node = AntNode(config)
    ```

=== "Rust"
    ```rust
    use ant_node::config::load_config;

    let config = load_config("config.yaml")?;
    let node = Node::new(config)?;
    ```

## Best Practices

1. **Network Settings**
   - Use a static IP if possible
   - Open required ports in firewall
   - Configure proper rewards address

2. **Storage Management**
   - Set appropriate storage limits
   - Enable regular backups
   - Monitor free space

3. **Security**
   - Run node with minimal privileges
   - Secure rewards address private key
   - Regular security updates

4. **Monitoring**
   - Enable logging
   - Monitor node health
   - Set up alerts
