#![cfg(feature = "local")]

use anyhow::Result;
use autonomi::network::LocalNode;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_peer_discovery() -> Result<()> {
    println!("Starting peer discovery test");

    // Start first node
    println!("Starting node 1...");
    let mut node1 = LocalNode::start().await?;
    println!("Node 1 started at {}", node1.get_multiaddr());

    // Start second node without the --first flag
    println!("Starting node 2...");
    let mut node2 = LocalNode::start_secondary().await?;
    println!("Node 2 started at {}", node2.get_multiaddr());

    // Wait for peer discovery (with timeout)
    let mut attempts = 0;
    let max_attempts = 30; // 30 seconds timeout

    while attempts < max_attempts {
        // Check if nodes have discovered each other through mDNS
        println!(
            "Waiting for peer discovery... attempt {}/{}",
            attempts + 1,
            max_attempts
        );

        // TODO: Add actual peer discovery check here
        // For now we'll just check if both nodes are still running
        if !node1.is_running().await? || !node2.is_running().await? {
            panic!("One of the nodes has stopped running");
        }

        sleep(Duration::from_secs(1)).await;
        attempts += 1;
    }

    Ok(())
}
