use anyhow::Result;
use autonomi::network::LocalNode;
use std::time::Duration;
use tokio::time::{sleep, timeout};

#[tokio::test]
async fn test_local_node_basic_functionality() -> Result<()> {
    println!("Starting basic functionality test");

    // Test node creation with random port
    let mut node = LocalNode::new_with_random_port()?;
    println!("Created node with random port: {}", node.rpc_port());

    // Verify initial state
    assert!(node.rpc_port() > 0);
    assert!(!node.is_running());
    assert!(node.peer_id().is_none());
    assert!(node.multiaddr().is_none());
    println!("Initial state verified");

    // Start the node with timeout
    println!("Starting node...");
    match timeout(Duration::from_secs(5), node.start()).await {
        Ok(result) => {
            result?;
            println!("Node started successfully");
        }
        Err(_) => {
            // Force cleanup if timeout
            if let Some(mut process) = node.take_process() {
                let _ = process.kill().await;
                let _ = process.wait().await;
            }
            anyhow::bail!("Timeout while starting node");
        }
    }

    // Quick verification of running state
    assert!(node.is_running());
    assert!(node.peer_id().is_some());
    assert!(node.multiaddr().is_some());
    println!("Post-start assertions passed");

    // Verify multiaddr contains peer ID and port
    let multiaddr = node.multiaddr().unwrap().to_string();
    println!("Node multiaddr: {}", multiaddr);
    assert!(multiaddr.contains(&node.peer_id().unwrap().to_string()));
    assert!(multiaddr.contains(&node.rpc_port().to_string()));
    println!("Multiaddr verification passed");

    // Brief stabilization period
    println!("Brief stabilization wait...");
    sleep(Duration::from_millis(500)).await;

    // Stop the node with timeout
    println!("Stopping node...");
    match timeout(Duration::from_secs(5), node.stop()).await {
        Ok(result) => {
            result?;
            println!("Node stopped successfully");
        }
        Err(_) => {
            // Force cleanup if timeout
            if let Some(mut process) = node.take_process() {
                let _ = process.kill().await;
                let _ = process.wait().await;
            }
            println!("Had to force cleanup due to timeout");
        }
    }

    // Final state verification
    assert!(!node.is_running());
    assert!(node.peer_id().is_none());
    assert!(node.multiaddr().is_none());
    println!("Post-stop assertions passed");

    println!("Test completed successfully");
    Ok(())
}
