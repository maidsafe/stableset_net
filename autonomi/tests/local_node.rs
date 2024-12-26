use anyhow::Result;
#[cfg(feature = "local")]
use autonomi::network::local::LocalNode;
use std::time::Duration;
use tokio::time::sleep;

#[cfg(feature = "local")]
#[tokio::test]
async fn test_peer_discovery() -> Result<()> {
    println!("Starting peer discovery test");

    // Create two nodes with random ports
    let port1 = portpicker::pick_unused_port().expect("No ports free");
    let port2 = portpicker::pick_unused_port().expect("No ports free");
    println!("Created nodes with ports: {} and {}", port1, port2);

    // Start node 1
    println!("Starting node 1...");
    let mut node1 = LocalNode::new(port1);
    node1.start().await?;
    let peer_id1 = node1.peer_id().await.unwrap();
    println!("Node 1 started with peer ID: {}", peer_id1);

    // Start node 2
    println!("Starting node 2...");
    let mut node2 = LocalNode::new(port2);
    node2.start().await?;
    let peer_id2 = node2.peer_id().await.unwrap();
    println!("Node 2 started with peer ID: {}", peer_id2);

    println!("Both nodes started successfully");

    // Connect the nodes
    println!("Connecting node 1 to node 2...");
    node1.connect_to(&node2).await?;
    println!("Connected node 1 to node 2");

    // Also connect node 2 to node 1 for bidirectional connection
    println!("Connecting node 2 to node 1...");
    node2.connect_to(&node1).await?;
    println!("Connected node 2 to node 1");

    println!("Waiting for peer discovery...");

    // Wait for peer discovery (with timeout)
    let mut attempts = 0;
    let max_attempts = 120; // Increased timeout to 120 seconds
    let mut success = false;

    while attempts < max_attempts {
        let node1_has_node2 = node1.has_discovered_peer(&peer_id2).await;
        let node2_has_node1 = node2.has_discovered_peer(&peer_id1).await;

        println!(
            "Discovery state - Node 1: {} peers (has Node 2: {}), Node 2: {} peers (has Node 1: {})",
            node1.discovered_peer_count().await,
            node1_has_node2,
            node2.discovered_peer_count().await,
            node2_has_node1
        );

        if node1_has_node2 && node2_has_node1 {
            success = true;
            break;
        }

        // Print node logs every 10 seconds
        if attempts % 10 == 0 {
            println!("Still waiting for peer discovery...");
        }

        sleep(Duration::from_secs(1)).await;
        attempts += 1;
    }

    // Clean up
    println!("Stopping nodes...");
    node1.stop().await?;
    node2.stop().await?;
    println!("Nodes stopped");

    if !success {
        panic!("Peer discovery failed within timeout period");
    }

    Ok(())
}

#[cfg(not(feature = "local"))]
#[test]
fn test_peer_discovery_local_feature_disabled() {
    println!("Test skipped: local feature is not enabled");
}
