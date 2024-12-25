use anyhow::{Context, Result};
use libp2p::Multiaddr;
use libp2p::PeerId;
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::Stdio;
use std::str::FromStr;
use tokio::io::AsyncBufReadExt;
use tokio::process::{Child, Command};
use tokio::sync::mpsc;
use tokio::time::Duration;

/// Get an available port by letting the OS assign one
fn get_available_port() -> Result<u16> {
    let listener = TcpListener::bind("127.0.0.1:0").context("Failed to bind to random port")?;
    Ok(listener
        .local_addr()
        .context("Failed to get local address")?
        .port())
}

/// Find the antnode binary in common locations
fn find_antnode_binary() -> Result<PathBuf> {
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

/// Error type for node operations
#[derive(Debug, thiserror::Error)]
pub enum NodeError {
    #[error("Node is already running")]
    AlreadyRunning,
    #[error("Node is not running")]
    NotRunning,
    #[error("Failed to start node: {0}")]
    StartFailure(String),
    #[error("Failed to stop node: {0}")]
    StopFailure(String),
    #[error("Binary not found: {0}")]
    BinaryNotFound(String),
    #[error("Timeout waiting for node info")]
    InfoTimeout,
    #[error("Failed to parse peer ID: {0}")]
    PeerIdParseError(String),
}

/// Represents a local node instance for testing purposes.
#[derive(Debug)]
pub struct LocalNode {
    /// The process handle for the running node
    process: Option<Child>,
    /// The RPC port the node is listening on
    rpc_port: u16,
    /// The peer ID of the node
    peer_id: Option<PeerId>,
    /// The multiaddress where the node can be reached
    multiaddr: Option<Multiaddr>,
}

impl LocalNode {
    /// Creates a new LocalNode instance with default values.
    pub fn new(rpc_port: u16) -> Self {
        Self {
            process: None,
            rpc_port,
            peer_id: None,
            multiaddr: None,
        }
    }

    /// Creates a new LocalNode instance with an automatically assigned port.
    pub fn new_with_random_port() -> Result<Self> {
        let port = get_available_port()?;
        Ok(Self::new(port))
    }

    /// Returns the RPC port this node is configured to use
    pub fn rpc_port(&self) -> u16 {
        self.rpc_port
    }

    /// Returns true if the node process is currently running
    pub fn is_running(&self) -> bool {
        self.process.is_some()
    }

    /// Returns the peer ID of the node if available
    pub fn peer_id(&self) -> Option<&PeerId> {
        self.peer_id.as_ref()
    }

    /// Returns the multiaddr of the node if available
    pub fn multiaddr(&self) -> Option<&Multiaddr> {
        self.multiaddr.as_ref()
    }

    /// Start the node process
    pub async fn start(&mut self) -> Result<(), NodeError> {
        if self.is_running() {
            return Err(NodeError::AlreadyRunning);
        }

        let binary_path =
            find_antnode_binary().map_err(|e| NodeError::BinaryNotFound(e.to_string()))?;

        println!("Starting node with binary: {:?}", binary_path);

        let mut cmd = Command::new(binary_path);
        cmd.env("EVM_NETWORK", "local")
            .env("RUST_LOG", "debug")
            .arg("--rewards-address")
            .arg("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266")
            .arg("--home-network")
            .arg("--local")
            .arg("--ip")
            .arg("127.0.0.1")
            .arg("--port")
            .arg(self.rpc_port.to_string())
            .arg("--ignore-cache")
            .arg("evm-custom")
            .arg("--data-payments-address")
            .arg("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266")
            .arg("--payment-token-address")
            .arg("0x5FbDB2315678afecb367f032d93F642f64180aa3")
            .arg("--rpc-url")
            .arg("http://localhost:8545")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        println!("Running command: {:?}", cmd);

        let mut child = cmd
            .spawn()
            .map_err(|e| NodeError::StartFailure(e.to_string()))?;

        let (tx, mut rx) = mpsc::channel(1);

        if let Some(stdout) = child.stdout.take() {
            let mut stdout_reader = tokio::io::BufReader::new(stdout).lines();
            let tx = tx.clone();

            tokio::spawn(async move {
                while let Ok(Some(line)) = stdout_reader.next_line().await {
                    println!("Node stdout: {}", line);
                    if line.contains("PeerId is ") {
                        if let Some(peer_id_str) = line.split("PeerId is ").nth(1) {
                            let _ = tx.send(peer_id_str.trim().to_string()).await;
                        }
                    }
                }
            });
        }

        if let Some(stderr) = child.stderr.take() {
            let mut stderr_reader = tokio::io::BufReader::new(stderr).lines();
            tokio::spawn(async move {
                while let Ok(Some(line)) = stderr_reader.next_line().await {
                    println!("Node stderr: {}", line);
                }
            });
        }

        match tokio::time::timeout(Duration::from_secs(5), rx.recv()).await {
            Ok(Some(peer_id_str)) => {
                let peer_id = PeerId::from_str(&peer_id_str)
                    .map_err(|e| NodeError::PeerIdParseError(e.to_string()))?;

                let multiaddr = format!(
                    "/ip4/127.0.0.1/udp/{}/quic-v1/p2p/{}",
                    self.rpc_port, peer_id
                )
                .parse()
                .map_err(|e| {
                    NodeError::StartFailure(format!("Failed to create multiaddr: {}", e))
                })?;

                self.peer_id = Some(peer_id);
                self.multiaddr = Some(multiaddr);
                self.process = Some(child);
                Ok(())
            }
            Ok(None) => Err(NodeError::StartFailure(
                "Process output channel closed".into(),
            )),
            Err(_) => {
                let _ = child.kill().await;
                Err(NodeError::InfoTimeout)
            }
        }
    }

    /// Stop the node process
    pub async fn stop(&mut self) -> Result<(), NodeError> {
        if let Some(mut child) = self.process.take() {
            let _ = child.kill().await;
            let _ = child.wait().await;
            self.peer_id = None;
            self.multiaddr = None;
            Ok(())
        } else {
            Err(NodeError::NotRunning)
        }
    }

    /// Takes ownership of the process handle if it exists
    pub fn take_process(&mut self) -> Option<Child> {
        self.process.take()
    }
}

impl Drop for LocalNode {
    fn drop(&mut self) {
        if let Some(mut child) = self.process.take() {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let _ = child.kill().await;
                let _ = child.wait().await;
            });
        }
    }
}

impl Clone for LocalNode {
    fn clone(&self) -> Self {
        Self {
            process: None,
            rpc_port: self.rpc_port,
            peer_id: self.peer_id.clone(),
            multiaddr: self.multiaddr.clone(),
        }
    }
}
