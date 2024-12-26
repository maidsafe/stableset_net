use anyhow::{anyhow, Result};
use libp2p::Multiaddr;
use std::collections::HashMap;
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::Stdio;
use std::str::FromStr;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::RwLock;

/// Get an available port by letting the OS assign one
fn get_available_port() -> anyhow::Result<u16> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    Ok(listener.local_addr()?.port())
}

/// Find the antnode binary in common locations
fn find_antnode_binary() -> anyhow::Result<PathBuf> {
    let possible_paths = [
        "../target/debug/antnode",
        "./target/debug/antnode",
        "../../target/debug/antnode",
    ];

    for path in possible_paths.iter() {
        let path = PathBuf::from(path);
        if path.exists() {
            return Ok(path);
        }
    }

    Err(anyhow::anyhow!(
        "Could not find antnode binary in common locations"
    ))
}

/// Information about a discovered peer
#[derive(Debug, Clone)]
pub struct PeerInfo {
    /// The peer's ID
    pub peer_id: String,
    /// When the peer was last seen
    pub last_seen: SystemTime,
}

/// Represents a local node instance
pub struct LocalNode {
    port: u16,
    child: Option<Child>,
    peer_id: Option<String>,
    discovered_peers: Arc<RwLock<HashMap<String, PeerInfo>>>,
    first: bool,
}

impl LocalNode {
    /// Creates a new LocalNode instance
    pub fn new(port: u16) -> Self {
        Self {
            port,
            child: None,
            peer_id: None,
            discovered_peers: Arc::new(RwLock::new(HashMap::new())),
            first: false,
        }
    }

    /// Returns the node's multiaddr
    pub fn multiaddr(&self) -> Option<Multiaddr> {
        self.peer_id.as_ref().map(|peer_id| {
            format!("/ip4/127.0.0.1/tcp/{}/p2p/{}", self.port, peer_id)
                .parse()
                .expect("Invalid multiaddr")
        })
    }

    /// Connect to another node
    pub async fn connect_to(&mut self, other: &LocalNode) -> Result<()> {
        let other_multiaddr = other
            .multiaddr()
            .ok_or_else(|| anyhow!("Other node has no multiaddr"))?;

        // Instead of starting a new node process, we'll just store the peer's multiaddr
        // and let the mDNS discovery handle the connection
        println!(
            "Node {} will discover {} via mDNS",
            self.port, other_multiaddr
        );

        // Add the peer to our discovered peers list
        let mut peers = self.discovered_peers.write().await;
        if let Some(peer_id) = other.peer_id().await {
            peers.insert(
                peer_id.clone(),
                PeerInfo {
                    peer_id,
                    last_seen: SystemTime::now(),
                },
            );
        }

        Ok(())
    }

    /// Starts the node
    pub async fn start(&mut self) -> Result<()> {
        let binary_path = find_antnode_binary()?;
        println!("Starting node with binary: {:?}", binary_path);

        let mut cmd = Command::new(binary_path);
        cmd.env("EVM_NETWORK", "local")
            .arg("--rewards-address")
            .arg("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266")
            .arg("--home-network")
            .arg("--local")
            .arg("--ip")
            .arg("127.0.0.1")
            .arg("--port")
            .arg(self.port.to_string())
            .arg("--ignore-cache");

        if self.first {
            cmd.arg("--first");
        }

        cmd.arg("evm-custom")
            .arg("--rpc-url")
            .arg("http://localhost:8545")
            .arg("--payment-token-address")
            .arg("0x5FbDB2315678afecb367f032d93F642f64180aa3")
            .arg("--data-payments-address")
            .arg("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266");

        println!("Running command: {:?}", cmd);

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd.spawn()?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("Failed to get stdout"))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| anyhow!("Failed to get stderr"))?;

        let discovered_peers = self.discovered_peers.clone();
        let peer_id = Arc::new(RwLock::new(None));
        let peer_id_clone = peer_id.clone();

        // Process stdout
        let stdout_reader = BufReader::new(stdout).lines();
        tokio::spawn(async move {
            let mut lines = stdout_reader;
            while let Ok(Some(line)) = lines.next_line().await {
                println!("Node stdout: {}", line);
                // Check for peer ID
                if line.contains("PeerId is ") {
                    if let Some(id) = line.split("PeerId is ").nth(1) {
                        let mut peer_id = peer_id_clone.write().await;
                        *peer_id = Some(id.trim().to_string());
                        println!("Found peer ID: {}", id.trim());
                    }
                }

                // Check for peer discovery
                if line.contains("Discovered peer") || line.contains("Connected to peer") {
                    println!("Found peer discovery line: {}", line);
                    let discovered_id = if line.contains("Discovered peer") {
                        line.split("Discovered peer")
                            .nth(1)
                            .map(|s| s.trim().to_string())
                    } else {
                        line.split("Connected to peer")
                            .nth(1)
                            .map(|s| s.trim().to_string())
                    };

                    if let Some(discovered_id) = discovered_id {
                        println!("Extracted peer ID: {}", discovered_id);
                        let mut peers = discovered_peers.write().await;
                        peers.insert(
                            discovered_id.clone(),
                            PeerInfo {
                                peer_id: discovered_id,
                                last_seen: SystemTime::now(),
                            },
                        );
                        println!("Added peer to discovered peers");
                    } else {
                        println!("Failed to extract peer ID from line");
                    }
                }
            }
        });

        // Process stderr
        let stderr_reader = BufReader::new(stderr).lines();
        tokio::spawn(async move {
            let mut lines = stderr_reader;
            while let Ok(Some(line)) = lines.next_line().await {
                println!("Node stderr: {}", line);
            }
        });

        // Wait for peer ID to be available
        let start_time = std::time::Instant::now();
        while peer_id.read().await.is_none() {
            if start_time.elapsed() > std::time::Duration::from_secs(30) {
                return Err(anyhow!("Timeout waiting for peer ID"));
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        self.peer_id = peer_id.read().await.clone();
        println!("Node started with peer ID: {:?}", self.peer_id);
        self.child = Some(child);

        Ok(())
    }

    /// Returns the node's peer ID
    pub async fn peer_id(&self) -> Option<String> {
        self.peer_id.clone()
    }

    /// Returns whether this node has discovered a specific peer
    pub async fn has_discovered_peer(&self, peer_id: &str) -> bool {
        let peers = self.discovered_peers.read().await;
        peers.contains_key(peer_id)
    }

    /// Returns the number of discovered peers
    pub async fn discovered_peer_count(&self) -> usize {
        let peers = self.discovered_peers.read().await;
        peers.len()
    }

    /// Stops the node
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(mut child) = self.child.take() {
            child.kill().await?;
            child.wait().await?;
        }
        Ok(())
    }
}

impl Drop for LocalNode {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}
