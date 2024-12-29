# Network Operations

This page documents the network operations available in the Ant Node API.

## Node Management

### Starting a Node

=== "Python"
    ```python
    from antnode import AntNode

    # Create and start a node
    node = AntNode()
    node.run(
        rewards_address="0x1234567890123456789012345678901234567890",
        evm_network="arbitrum_sepolia",
        ip="0.0.0.0",
        port=12000,
        initial_peers=[
            "/ip4/142.93.37.4/udp/40184/quic-v1/p2p/12D3KooWPC8q7QGZsmuTtCYxZ2s3FPXPZcS8LVKkayXkVFkqDEQB",
        ]
    )
    ```

=== "Rust"
    ```rust
    use ant_node::{Node, NodeConfig};

    // Create and start a node
    let config = NodeConfig::default();
    let mut node = Node::new(config)?;
    node.run().await?;
    ```

### Node Information

=== "Python"
    ```python
    # Get node's peer ID
    peer_id = node.peer_id()

    # Get current rewards address
    address = node.get_rewards_address()

    # Get routing table information
    kbuckets = node.get_kbuckets()
    for distance, peers in kbuckets:
        print(f"Distance {distance}: {len(peers)} peers")

    # Get all stored record addresses
    records = node.get_all_record_addresses()
    ```

=== "Rust"
    ```rust
    // Get node's peer ID
    let peer_id = node.peer_id();

    // Get current rewards address
    let address = node.rewards_address();

    // Get routing table information
    let kbuckets = node.get_kbuckets()?;
    for (distance, peers) in kbuckets {
        println!("Distance {}: {} peers", distance, peers.len());
    }

    // Get all stored record addresses
    let records = node.get_all_record_addresses()?;
    ```

## Network Events

### Event Handling

=== "Python"
    ```python
    from antnode import NodeEvent

    # Register event handlers
    @node.on(NodeEvent.CONNECTED)
    def handle_connected():
        print("Connected to network")

    @node.on(NodeEvent.CHUNK_STORED)
    def handle_chunk_stored(address):
        print(f"Chunk stored: {address}")

    @node.on(NodeEvent.REWARD_RECEIVED)
    def handle_reward(amount, address):
        print(f"Reward received: {amount} at {address}")
    ```

=== "Rust"
    ```rust
    use ant_node::{NodeEvent, NodeEventsReceiver};

    // Get event receiver
    let mut events: NodeEventsReceiver = node.event_receiver();

    // Handle events
    while let Ok(event) = events.recv().await {
        match event {
            NodeEvent::ConnectedToNetwork => println!("Connected to network"),
            NodeEvent::ChunkStored(addr) => println!("Chunk stored: {}", addr),
            NodeEvent::RewardReceived(amount, addr) => {
                println!("Reward received: {} at {}", amount, addr)
            }
            NodeEvent::ChannelClosed => break,
            NodeEvent::TerminateNode(reason) => {
                println!("Node terminated: {}", reason);
                break;
            }
        }
    }
    ```

## Peer Management

### Peer Discovery

=== "Python"
    ```python
    # Add a peer manually
    node.add_peer("/ip4/1.2.3.4/udp/12000/quic-v1/p2p/...")

    # Get connected peers
    peers = node.get_connected_peers()
    for peer in peers:
        print(f"Peer: {peer.id}, Address: {peer.address}")

    # Find peers near an address
    nearby = node.find_peers_near(target_address)
    ```

=== "Rust"
    ```rust
    // Add a peer manually
    node.add_peer("/ip4/1.2.3.4/udp/12000/quic-v1/p2p/...".parse()?)?;

    // Get connected peers
    let peers = node.get_connected_peers()?;
    for peer in peers {
        println!("Peer: {}, Address: {}", peer.id, peer.address);
    }

    // Find peers near an address
    let nearby = node.find_peers_near(&target_address).await?;
    ```

## Data Storage

### Record Management

=== "Python"
    ```python
    # Store a record
    key = "0123456789abcdef"  # Hex string
    value = b"Hello, World!"
    node.store_record(key, value, "chunk")

    # Retrieve a record
    data = node.get_record(key)

    # Delete a record
    success = node.delete_record(key)

    # Get total storage size
    size = node.get_stored_records_size()
    ```

=== "Rust"
    ```rust
    use ant_node::storage::RecordType;

    // Store a record
    let key = "0123456789abcdef";  // Hex string
    let value = b"Hello, World!";
    node.store_record(key, value, RecordType::Chunk)?;

    // Retrieve a record
    let data = node.get_record(key)?;

    // Delete a record
    let success = node.delete_record(key)?;

    // Get total storage size
    let size = node.get_stored_records_size()?;
    ```

## Network Metrics

### Performance Monitoring

=== "Python"
    ```python
    # Get network metrics
    metrics = node.get_metrics()
    print(f"Connected peers: {metrics.peer_count}")
    print(f"Records stored: {metrics.record_count}")
    print(f"Storage used: {metrics.storage_used}")
    print(f"Bandwidth in: {metrics.bandwidth_in}")
    print(f"Bandwidth out: {metrics.bandwidth_out}")

    # Get node uptime
    uptime = node.get_uptime()
    print(f"Node uptime: {uptime} seconds")
    ```

=== "Rust"
    ```rust
    // Get network metrics
    let metrics = node.get_metrics()?;
    println!("Connected peers: {}", metrics.peer_count);
    println!("Records stored: {}", metrics.record_count);
    println!("Storage used: {}", metrics.storage_used);
    println!("Bandwidth in: {}", metrics.bandwidth_in);
    println!("Bandwidth out: {}", metrics.bandwidth_out);

    // Get node uptime
    let uptime = node.get_uptime()?;
    println!("Node uptime: {} seconds", uptime);
    ```

## Best Practices

1. **Event Handling**
   - Always handle critical events
   - Implement proper error recovery
   - Log important events

2. **Peer Management**
   - Maintain healthy peer connections
   - Implement peer discovery
   - Handle peer disconnections

3. **Storage Management**
   - Monitor storage usage
   - Implement cleanup policies
   - Handle storage full conditions

4. **Network Health**
   - Monitor network metrics
   - Track bandwidth usage
   - Monitor node performance
