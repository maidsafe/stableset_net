use anyhow::{Context, Result};
use autonomi::{Client, ClientConfig};
use dirs_next;
use std::net::{IpAddr, TcpListener};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use tokio::time::sleep;
use serial_test::serial;
use ant_networking::find_local_ip;

// Get the local IP for testing
lazy_static::lazy_static! {
    static ref LOCAL_IP: IpAddr = find_local_ip().expect("Failed to find local IP");
}

#[derive(Debug)]
struct NodeOutput {
    peer_id: Option<String>,
    #[allow(dead_code)]
    listeners: Vec<String>,
}

async fn process_output(
    stdout: tokio::process::ChildStdout,
    stderr: tokio::process::ChildStderr,
    _port: u16,
    node_output: Arc<Mutex<NodeOutput>>,
    prefix: String,
) {
    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);

    let mut stdout_lines = stdout_reader.lines();
    let mut stderr_lines = stderr_reader.lines();

    loop {
        tokio::select! {
            Ok(Some(line)) = stdout_lines.next_line() => {
                println!("[{}] stdout: {}", prefix, line);
                if line.contains("PeerId is ") {
                    if let Some(peer_id) = line.split("PeerId is ").nth(1) {
                        let mut output = node_output.lock().await;
                        output.peer_id = Some(peer_id.trim().to_string());
                    }
                }
            }
            Ok(Some(line)) = stderr_lines.next_line() => {
                println!("[{}] stderr: {}", prefix, line);
            }
            else => break,
        }
    }
}

fn get_available_port() -> Result<u16> {
    let listener = TcpListener::bind((*LOCAL_IP, 0))?;
    Ok(listener.local_addr()?.port())
}

async fn cleanup_nodes() -> Result<()> {
    // Kill any existing nodes
    let _ = Command::new("pkill")
        .arg("-f")
        .arg("antnode")
        .output()
        .await?;

    // Remove node data directories
    let data_dir = dirs_next::data_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not get data directory"))?
        .join("autonomi")
        .join("node");
    if data_dir.exists() {
        std::fs::remove_dir_all(&data_dir)?;
    }

    // Allow more time for cleanup
    sleep(Duration::from_secs(5)).await;

    Ok(())
}

async fn start_local_node(
    is_first: bool,
    peer_addr: Option<&str>,
    port: Option<u16>,
) -> Result<(Child, u16, Arc<Mutex<NodeOutput>>)> {
    let port = port.unwrap_or_else(|| get_available_port().unwrap());
    println!("Setting up node to listen on {}:{}", *LOCAL_IP, port);

    let mut cmd = Command::new("../target/debug/antnode");
    cmd.arg("--rewards-address")
        .arg("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266")
        .arg("--home-network")
        .arg("--local")
        .arg("--ip")
        .arg(LOCAL_IP.to_string())
        .arg("--port")
        .arg(port.to_string())
        .arg("--ignore-cache");

    if is_first {
        cmd.arg("--first");
    }

    if let Some(addr) = peer_addr {
        println!("Connecting to peer: {}", addr);
        cmd.arg("--peer").arg(addr);
    }

    cmd.arg("evm-custom")
        .arg("--data-payments-address")
        .arg("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266")
        .arg("--payment-token-address")
        .arg("0x5FbDB2315678afecb367f032d93F642f64180aa3")
        .arg("--rpc-url")
        .arg("http://localhost:8545");

    println!("Starting node with command: {:?}", cmd);

    let node_output = Arc::new(Mutex::new(NodeOutput {
        peer_id: None,
        listeners: Vec::new(),
    }));
    let node_output_clone = node_output.clone();

    let mut child = cmd
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .context("Failed to start node")?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    tokio::spawn(process_output(
        stdout,
        stderr,
        port,
        node_output_clone,
        format!("Node {}", port),
    ));

    Ok((child, port, node_output))
}

async fn wait_for_node_ready(port: u16, node_output: Arc<Mutex<NodeOutput>>) -> Result<()> {
    let mut retries = 0;
    let max_retries = 30;
    let retry_delay = Duration::from_secs(2);

    while retries < max_retries {
        let output = node_output.lock().await;
        if let Some(peer_id) = &output.peer_id {
            println!(
                "Node is ready on port {} with peer_id Some(\"{}\")",
                port, peer_id
            );
            return Ok(());
        }
        drop(output);

        println!(
            "Waiting for node on port {} to be ready (attempt {}/{})",
            port,
            retries + 1,
            max_retries
        );
        sleep(retry_delay).await;
        retries += 1;
    }

    anyhow::bail!("Node failed to start and provide peer ID")
}

async fn get_node_multiaddr(port: u16, peer_id: &str) -> String {
    format!("/ip4/{}/udp/{}/quic-v1/p2p/{}", *LOCAL_IP, port, peer_id)
}

#[tokio::test]
#[serial]
async fn test_local_client_operations() -> Result<()> {
    println!("\nStarting test_local_client_operations");
    cleanup_nodes().await?;

    // Start first node
    let (_node1, port1, node_output1) = start_local_node(true, None, None).await?;
    wait_for_node_ready(port1, node_output1.clone()).await?;

    let peer_id1 = {
        let output = node_output1.lock().await;
        output.peer_id.clone().expect("Peer ID should be set")
    };
    let first_node_addr = get_node_multiaddr(port1, &peer_id1).await;
    println!("First node address: {}", first_node_addr);

    // Allow time for the first node to be fully ready
    println!("Waiting for first node to be fully ready...");
    sleep(Duration::from_secs(30)).await;

    // Start second node with first node as peer
    let (_node2, port2, node_output2) = start_local_node(false, Some(&first_node_addr), None).await?;
    wait_for_node_ready(port2, node_output2.clone()).await?;

    let peer_id2 = {
        let output = node_output2.lock().await;
        output.peer_id.clone().expect("Peer ID should be set")
    };
    println!("Second node peer ID: {}", peer_id2);

    // Initialize client in local mode with bootstrap peer
    println!("Initializing client...");
    let config = ClientConfig {
        local: true,
        peers: Some(vec![first_node_addr.parse()?]),
    };
    let client = Client::init_with_config(config).await?;
    println!("Client initialized");

    // Allow time for peer discovery
    println!("Waiting for peer discovery...");
    for i in 1..=30 {
        sleep(Duration::from_secs(10)).await;
        let info = client.network_info().await?;
        println!(
            "Check {}/30: Connected to {} peers",
            i,
            info.connected_peers.len()
        );
        if !info.connected_peers.is_empty() {
            println!("Connected peers:");
            for peer in &info.connected_peers {
                println!("  - {}", peer);
            }
            println!("Successfully connected to peers");
            return Ok(());
        }
        println!("No peers connected yet, waiting...");
    }

    anyhow::bail!("Failed to connect to any peers after multiple attempts")
}

#[tokio::test]
#[serial]
async fn test_local_client_with_peers() -> Result<()> {
    println!("\nStarting test_local_client_with_peers");
    cleanup_nodes().await?;

    // Start first node
    let (_node1, port1, node_output1) = start_local_node(true, None, None).await?;
    wait_for_node_ready(port1, node_output1.clone()).await?;

    let peer_id1 = {
        let output = node_output1.lock().await;
        output.peer_id.clone().expect("Peer ID should be set")
    };
    let first_node_addr = get_node_multiaddr(port1, &peer_id1).await;
    println!("First node address: {}", first_node_addr);

    // Allow time for the first node to be fully ready
    println!("Waiting for first node to be fully ready...");
    sleep(Duration::from_secs(30)).await;

    // Create first client
    println!("Initializing first client...");
    let config1 = ClientConfig {
        local: true,
        peers: Some(vec![first_node_addr.parse()?]),
    };
    let client1 = Client::init_with_config(config1).await?;
    println!("First client initialized");

    // Wait for first client to connect
    println!("Waiting for first client to connect...");
    for i in 1..=10 {
        sleep(Duration::from_secs(10)).await;
        let info = client1.network_info().await?;
        println!(
            "Check {}: First client connected to {} peers",
            i,
            info.connected_peers.len()
        );
        if !info.connected_peers.is_empty() {
            break;
        }
        if i == 10 {
            anyhow::bail!("Failed to connect to any peers after multiple attempts");
        }
    }

    // Create second client
    println!("Initializing second client...");
    let config2 = ClientConfig {
        local: true,
        peers: Some(vec![first_node_addr.parse()?]),
    };
    let client2 = Client::init_with_config(config2).await?;
    println!("Second client initialized");

    // Wait for second client to connect
    println!("Waiting for second client to connect...");
    for i in 1..=10 {
        sleep(Duration::from_secs(10)).await;
        let info = client2.network_info().await?;
        println!(
            "Check {}: Second client connected to {} peers",
            i,
            info.connected_peers.len()
        );
        if !info.connected_peers.is_empty() {
            println!("Successfully connected to peers");
            return Ok(());
        }
    }

    anyhow::bail!("Failed to connect to any peers after multiple attempts")
}
